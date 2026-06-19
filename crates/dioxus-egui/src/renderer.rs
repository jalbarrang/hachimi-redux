//! A minimal Dioxus renderer that targets egui_taffy.
//!
//! Dioxus diffs a `VirtualDom` into a stream of `Mutation`s (a stack machine).
//! We apply them to a retained arena of nodes, then each egui frame we *walk*
//! that arena and emit immediate `egui_taffy` calls. egui owns the input loop,
//! so when a button is clicked we record its `ElementId` and the app delivers a
//! real `onclick` event back into the VirtualDom via `runtime().handle_event` —
//! so component `onclick` closures run exactly like on any other Dioxus renderer.

use std::cell::RefCell;

use dioxus::dioxus_core::{AttributeValue, ElementId, Template, TemplateAttribute, TemplateNode, WriteMutations};

/// A one-shot imperative egui draw closure for a `native="egui"` island.
type NativeDrawFn = Box<dyn FnMut(&mut egui::Ui)>;

thread_local! {
    /// When set, the next `native="egui"` container walks this closure instead of children.
    pub(crate) static NATIVE_EGUI_DRAW: RefCell<Option<NativeDrawFn>> = const { RefCell::new(None) };
}

/// Register a one-shot egui draw closure for the next `native="egui"` Dioxus node.
pub fn set_native_draw(f: impl FnMut(&mut egui::Ui) + 'static) {
    NATIVE_EGUI_DRAW.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(f));
    });
}

use crate::style::container_style;
use egui::{Color32, CornerRadius, RichText, Stroke, StrokeKind};
use egui_taffy::{Tui, TuiBuilderLogic};

/// An event recorded during a frame walk, to be delivered back to the VDOM by
/// the caller via `runtime().handle_event`. egui owns the input loop, so the
/// renderer can only *observe* clicks and value changes during the walk; the app
/// turns these into real Dioxus events (see `lib.rs`).
#[derive(Debug, Clone, PartialEq)]
pub enum DomEvent {
    /// A `click` on an element with an `onclick` listener.
    Click(ElementId),
    /// A value change on a form widget. `name` is the html event name
    /// (`"input"` or `"change"`); `value` is the widget's new value as the
    /// string an `oninput`/`onchange` closure reads via `event.value()`
    /// (`"true"`/`"false"` for a checkbox).
    Form {
        id: ElementId,
        name: &'static str,
        value: String,
    },
}

/// Which value listeners a node carries. Mirrors how `onclick` is tracked, but
/// for the form events the value widgets deliver.
#[derive(Debug, Default, Clone, Copy)]
struct Listeners {
    click: bool,
    input: bool,
    change: bool,
}

/// Inherited text styling, threaded down the walk so a parent element's
/// `"color"`/`"font-size"`/`"weight"` attributes style the text inside it (like
/// CSS inheritance). Children override only the fields they set.
#[derive(Debug, Default, Clone, Copy)]
struct TextCtx {
    color: Option<Color32>,
    size: Option<f32>,
    strong: bool,
}

impl TextCtx {
    /// Layer an element's text attributes over the inherited context.
    fn inherit(self, attrs: &[(String, String)]) -> TextCtx {
        TextCtx {
            color: parse_color(DioxusEgui::attr(attrs, "color")).or(self.color),
            size: DioxusEgui::attr(attrs, "font-size")
                .and_then(|v| v.parse::<f32>().ok())
                .or(self.size),
            strong: matches!(DioxusEgui::attr(attrs, "weight"), Some("bold" | "strong")) || self.strong,
        }
    }

    /// Apply this context to some text, producing a styled `RichText`.
    fn rich(self, text: impl Into<String>) -> RichText {
        let mut rt = RichText::new(text.into());
        if let Some(c) = self.color {
            rt = rt.color(c);
        }
        if let Some(s) = self.size {
            rt = rt.size(s);
        }
        if self.strong {
            rt = rt.strong();
        }
        rt
    }
}

/// Background/border decoration parsed from an element's visual attributes.
#[derive(Debug, Default, Clone, Copy)]
struct Deco {
    bg: Option<Color32>,
    border: Option<Color32>,
    border_width: f32,
    radius: u8,
}

impl Deco {
    /// Parse `"bg"`/`"border"`/`"border-width"`/`"radius"`. Returns `None` when
    /// there is nothing to paint (no fill and no border), so plain layout
    /// containers skip the extra background pass.
    fn parse(attrs: &[(String, String)]) -> Option<Deco> {
        let bg = parse_color(DioxusEgui::attr(attrs, "bg"));
        let border = parse_color(DioxusEgui::attr(attrs, "border"));
        if bg.is_none() && border.is_none() {
            return None;
        }
        Some(Deco {
            bg,
            border,
            border_width: DioxusEgui::attr_f32(attrs, "border-width").unwrap_or(1.0),
            radius: DioxusEgui::attr(attrs, "radius")
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(0),
        })
    }

    fn paint(&self, ui: &egui::Ui, rect: egui::Rect) {
        let cr = CornerRadius::same(self.radius);
        if let Some(bg) = self.bg {
            ui.painter().rect_filled(rect, cr, bg);
        }
        if let Some(border) = self.border {
            ui.painter()
                .rect_stroke(rect, cr, Stroke::new(self.border_width, border), StrokeKind::Inside);
        }
    }
}

/// Parse a `#rrggbb` or `#rrggbbaa` color string. `None` (missing attr) and
/// malformed values yield `None`.
fn parse_color(s: Option<&str>) -> Option<Color32> {
    let hex = s?.trim().strip_prefix('#')?;
    let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
    match hex.len() {
        6 => Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?)),
        8 => Some(Color32::from_rgba_unmultiplied(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
        _ => None,
    }
}

/// Tint a fill for a button's interaction state: lift toward white on hover,
/// darken when pressed. A transparent (alpha 0) base stays transparent at rest
/// but shows a faint hover overlay, matching shadcn's ghost button.
fn button_fill(base: Color32, response: &egui::Response) -> Color32 {
    if response.is_pointer_button_down_on() {
        blend(base, Color32::BLACK, 0.15)
    } else if response.hovered() {
        if base.a() == 0 {
            Color32::from_white_alpha(18)
        } else {
            blend(base, Color32::WHITE, 0.10)
        }
    } else {
        base
    }
}

/// Mix `a` toward `b` by `t` (0..=1), preserving `a`'s alpha.
fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()), a.a())
}

#[derive(Debug)]
enum NodeKind {
    Element {
        tag: &'static str,
        attrs: Vec<(String, String)>,
        children: Vec<usize>,
        /// The node's ElementId (set when it gets a listener) and which
        /// listeners it has — needed to deliver events back to the VDOM.
        eid: Option<usize>,
        listeners: Listeners,
    },
    Text(String),
    Placeholder,
}

#[derive(Debug)]
struct Node {
    parent: Option<usize>,
    kind: NodeKind,
}

fn empty_element(tag: &'static str) -> NodeKind {
    NodeKind::Element {
        tag,
        attrs: Vec::new(),
        children: Vec::new(),
        eid: None,
        listeners: Listeners::default(),
    }
}

pub struct DioxusEgui {
    arena: Vec<Option<Node>>,
    element_map: Vec<Option<usize>>,
    stack: Vec<usize>,
    root: usize,
}

impl Default for DioxusEgui {
    fn default() -> Self {
        Self::new()
    }
}

impl DioxusEgui {
    pub fn new() -> Self {
        let mut me = Self {
            arena: Vec::new(),
            element_map: Vec::new(),
            stack: Vec::new(),
            root: 0,
        };
        let root = me.alloc(Node {
            parent: None,
            kind: empty_element("root"),
        });
        me.root = root;
        me.set_element(ElementId(0), root);
        me.stack.push(root);
        me
    }

    fn alloc(&mut self, n: Node) -> usize {
        self.arena.push(Some(n));
        self.arena.len() - 1
    }

    fn set_element(&mut self, id: ElementId, idx: usize) {
        if id.0 >= self.element_map.len() {
            self.element_map.resize(id.0 + 1, None);
        }
        self.element_map[id.0] = Some(idx);
    }

    fn node_of(&self, id: ElementId) -> usize {
        self.element_map[id.0].expect("unknown ElementId")
    }

    fn node_ref(&self, idx: usize) -> &Node {
        self.arena[idx].as_ref().expect("arena slot occupied")
    }

    fn node_mut(&mut self, idx: usize) -> &mut Node {
        self.arena[idx].as_mut().expect("arena slot occupied")
    }

    fn children_mut(&mut self, idx: usize) -> &mut Vec<usize> {
        match &mut self.node_mut(idx).kind {
            NodeKind::Element { children, .. } => children,
            _ => panic!("node {idx} is not an element"),
        }
    }

    /// Descend from `start` following child indices in `path`.
    fn navigate(&self, start: usize, path: &[u8]) -> usize {
        let mut cur = start;
        for &p in path {
            cur = match &self.node_ref(cur).kind {
                NodeKind::Element { children, .. } => children[p as usize],
                _ => panic!("cannot descend into non-element"),
            };
        }
        cur
    }

    fn build_template_node(&mut self, tn: &TemplateNode, parent: Option<usize>) -> usize {
        match tn {
            TemplateNode::Element {
                tag, attrs, children, ..
            } => {
                let mut a = Vec::new();
                for attr in *attrs {
                    if let TemplateAttribute::Static { name, value, .. } = attr {
                        a.push((name.to_string(), value.to_string()));
                    }
                }
                let idx = self.alloc(Node {
                    parent,
                    kind: NodeKind::Element {
                        tag,
                        attrs: a,
                        children: Vec::new(),
                        eid: None,
                        listeners: Listeners::default(),
                    },
                });
                for c in *children {
                    let ci = self.build_template_node(c, Some(idx));
                    self.children_mut(idx).push(ci);
                }
                idx
            }
            TemplateNode::Text { text } => self.alloc(Node {
                parent,
                kind: NodeKind::Text(text.to_string()),
            }),
            TemplateNode::Dynamic { .. } => self.alloc(Node {
                parent,
                kind: NodeKind::Placeholder,
            }),
        }
    }

    fn pop_top(&mut self, m: usize) -> Vec<usize> {
        let at = self.stack.len() - m;
        self.stack.split_off(at)
    }

    fn replace_in_parent(&mut self, target: usize, new: &[usize]) {
        let parent = self.node_ref(target).parent.expect("no parent");
        let pos = {
            let ch = self.children_mut(parent);
            ch.iter().position(|&c| c == target).expect("target not child")
        };
        for &n in new {
            self.node_mut(n).parent = Some(parent);
        }
        self.children_mut(parent).splice(pos..=pos, new.iter().copied());
    }

    // ---- frame walk -> egui_taffy ----

    /// Walk the tree, emit egui_taffy, and collect any events (button clicks,
    /// widget value changes) observed this frame, to be delivered back to the
    /// VDOM by the caller.
    pub fn render(&self, tui: &mut Tui, events: &mut Vec<DomEvent>) {
        let kids = self.child_ids(self.root);
        for c in kids {
            self.walk(c, tui, TextCtx::default(), events);
        }
    }

    fn child_ids(&self, idx: usize) -> Vec<usize> {
        match &self.node_ref(idx).kind {
            NodeKind::Element { children, .. } => children.clone(),
            _ => Vec::new(),
        }
    }

    fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
        attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }

    fn attr_f32(attrs: &[(String, String)], name: &str) -> Option<f32> {
        Self::attr(attrs, name).and_then(|v| v.parse::<f32>().ok())
    }

    fn attr_f64(attrs: &[(String, String)], name: &str) -> Option<f64> {
        Self::attr(attrs, name).and_then(|v| v.parse::<f64>().ok())
    }

    fn attr_bool(attrs: &[(String, String)], name: &str) -> bool {
        Self::attr(attrs, name) == Some("true")
    }

    /// Emit a value-change event to every value listener present on a node.
    /// `oninput` fires on every change, `onchange` on commit; we deliver to
    /// whichever the component declared (matching how the closures are written).
    fn push_value(eid: Option<usize>, listeners: &Listeners, value: String, out: &mut Vec<DomEvent>) {
        let Some(e) = eid else { return };
        if listeners.input {
            out.push(DomEvent::Form {
                id: ElementId(e),
                name: "input",
                value: value.clone(),
            });
        }
        if listeners.change {
            out.push(DomEvent::Form {
                id: ElementId(e),
                name: "change",
                value,
            });
        }
    }

    fn walk(&self, idx: usize, tui: &mut Tui, ctx: TextCtx, events: &mut Vec<DomEvent>) {
        let node = self.node_ref(idx);
        match &node.kind {
            NodeKind::Text(s) => {
                tui.label(ctx.rich(s.clone()));
            }
            NodeKind::Placeholder => {}
            NodeKind::Element {
                tag,
                attrs,
                children,
                eid,
                listeners,
            } => match *tag {
                "button" => self.walk_button(attrs, children, eid, listeners, ctx, tui, events),
                "input" => self.walk_input(attrs, eid, listeners, tui, events),
                "select" => self.walk_select(attrs, children, eid, listeners, ctx, tui, events),
                "textarea" => self.walk_textarea(attrs, eid, listeners, tui, events),
                "img" => self.walk_img(attrs, tui),
                _ => self.walk_container(attrs, children, ctx, tui, events),
            },
        }
    }

    /// Render a layout container. Supports `scroll="y"` / `scroll="x"` on divs.
    fn walk_container(
        &self,
        attrs: &[(String, String)],
        children: &[usize],
        ctx: TextCtx,
        tui: &mut Tui,
        events: &mut Vec<DomEvent>,
    ) {
        let ctx = ctx.inherit(attrs);
        let style = container_style(attrs);
        if Self::attr(attrs, "native") == Some("egui") {
            tui.style(style).ui(|ui| {
                NATIVE_EGUI_DRAW.with(|slot| {
                    if let Some(mut draw) = slot.borrow_mut().take() {
                        draw(ui);
                    }
                });
            });
            return;
        }
        let draw = |me: &Self, tui: &mut Tui, events: &mut Vec<DomEvent>| {
            for &c in children {
                me.walk(c, tui, ctx, events);
            }
        };
        let paint_children = |tui: &mut Tui, events: &mut Vec<DomEvent>| draw(self, tui, events);

        match Deco::parse(attrs) {
            Some(deco) => {
                tui.style(style).add_with_background_ui(
                    move |ui, container| deco.paint(ui, container.full_container()),
                    |tui, _| paint_children(tui, events),
                );
            }
            None => {
                tui.style(style).add(|tui| paint_children(tui, events));
            }
        }
    }

    /// Render a button. A themed button (any `bg`/`border`/`radius`/`color`
    /// attribute) paints its own variant-colored face with hover/press states;
    /// a bare button falls back to egui's default button look. Either way a
    /// click is delivered to the VDOM if the element has an `onclick` listener.
    #[allow(clippy::too_many_arguments)]
    fn walk_button(
        &self,
        attrs: &[(String, String)],
        children: &[usize],
        eid: &Option<usize>,
        listeners: &Listeners,
        ctx: TextCtx,
        tui: &mut Tui,
        events: &mut Vec<DomEvent>,
    ) {
        let ctx = ctx.inherit(attrs);
        let themed = Deco::parse(attrs).is_some() || Self::attr(attrs, "color").is_some();
        let clicked = if themed {
            let deco = Deco::parse(attrs).unwrap_or_default();
            let style = container_style(attrs);
            let ret = tui.style(style).add_with_background_ui(
                move |ui, container| {
                    let rect = container.full_container();
                    let resp = ui.interact(rect, ui.id().with("honse-btn"), egui::Sense::click());
                    let painted = Deco {
                        bg: Some(button_fill(deco.bg.unwrap_or(Color32::TRANSPARENT), &resp)),
                        ..deco
                    };
                    painted.paint(ui, rect);
                    resp
                },
                |tui, _| {
                    for &c in children {
                        self.walk(c, tui, ctx, events);
                    }
                },
            );
            ret.background.clicked()
        } else {
            tui.button(|tui| {
                for &c in children {
                    self.walk(c, tui, ctx, events);
                }
            })
            .clicked()
        };
        if clicked && listeners.click {
            if let Some(e) = eid {
                events.push(DomEvent::Click(ElementId(*e)));
            }
        }
    }

    /// Render a value widget (`input`), keyed off its `type` attribute, and
    /// deliver the new value back as a form event when the user changes it.
    fn walk_input(
        &self,
        attrs: &[(String, String)],
        eid: &Option<usize>,
        listeners: &Listeners,
        tui: &mut Tui,
        events: &mut Vec<DomEvent>,
    ) {
        match Self::attr(attrs, "type") {
            Some("checkbox") => {
                let was = Self::attr_bool(attrs, "checked");
                let mut now = was;
                tui.ui(|ui| ui.checkbox(&mut now, ""));
                if now != was {
                    Self::push_value(*eid, listeners, now.to_string(), events);
                }
            }
            Some("range") => {
                let min = Self::attr_f64(attrs, "min").unwrap_or(0.0);
                let max = Self::attr_f64(attrs, "max").unwrap_or(100.0);
                let was = Self::attr_f64(attrs, "value").unwrap_or(min);
                let mut now = was;
                let resp = tui.ui(|ui| ui.add(egui::Slider::new(&mut now, min..=max)));
                if resp.changed() {
                    Self::push_value(*eid, listeners, now.to_string(), events);
                }
            }
            _ => {
                let was = Self::attr(attrs, "value").unwrap_or("").to_string();
                let mut now = was.clone();
                let resp = tui.ui(|ui| ui.text_edit_singleline(&mut now));
                if resp.changed() && now != was {
                    Self::push_value(*eid, listeners, now, events);
                }
            }
        }
    }

    /// Render a `<select>` combo box from `<option value="…">label</option>` children.
    #[allow(clippy::too_many_arguments)]
    fn walk_select(
        &self,
        attrs: &[(String, String)],
        children: &[usize],
        eid: &Option<usize>,
        listeners: &Listeners,
        ctx: TextCtx,
        tui: &mut Tui,
        events: &mut Vec<DomEvent>,
    ) {
        let mut options: Vec<(String, String)> = Vec::new();
        for &c in children {
            if let NodeKind::Element {
                tag: "option",
                attrs,
                children,
                ..
            } = &self.node_ref(c).kind
            {
                let value = Self::attr(attrs, "value")
                    .or_else(|| Self::attr(attrs, "label"))
                    .unwrap_or("")
                    .to_string();
                let label = children
                    .iter()
                    .filter_map(|&cc| match &self.node_ref(cc).kind {
                        NodeKind::Text(s) => Some(s.clone()),
                        _ => None,
                    })
                    .next()
                    .unwrap_or_else(|| value.clone());
                options.push((value, label));
            }
        }

        let current = Self::attr(attrs, "value").unwrap_or("").to_string();
        let mut selected = current.clone();
        let mut changed = false;
        tui.ui(|ui| {
            egui::ComboBox::from_id_salt(ui.id().with("dioxus-select"))
                .selected_text(
                    options
                        .iter()
                        .find(|(v, _)| v == &selected)
                        .map(|(_, l)| l.as_str())
                        .unwrap_or(&selected),
                )
                .show_ui(ui, |ui| {
                    for (value, label) in &options {
                        if ui.selectable_value(&mut selected, value.clone(), label).clicked() {
                            changed = true;
                        }
                    }
                });
        });
        if changed && selected != current {
            Self::push_value(*eid, listeners, selected, events);
        }
        let _ = ctx; // options don't use text ctx today
    }

    /// Render a multiline `<textarea>`.
    fn walk_textarea(
        &self,
        attrs: &[(String, String)],
        eid: &Option<usize>,
        listeners: &Listeners,
        tui: &mut Tui,
        events: &mut Vec<DomEvent>,
    ) {
        let was = Self::attr(attrs, "value").unwrap_or("").to_string();
        let mut now = was.clone();
        let rows = Self::attr(attrs, "rows")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(3);
        let resp = tui.ui(|ui| {
            ui.add(
                egui::TextEdit::multiline(&mut now)
                    .desired_rows(rows)
                    .desired_width(f32::INFINITY),
            )
        });
        if resp.changed() && now != was {
            Self::push_value(*eid, listeners, now, events);
        }
    }

    /// Render an `<img src="…" width="…" height="…">` via egui's image widget.
    fn walk_img(&self, attrs: &[(String, String)], tui: &mut Tui) {
        let src = Self::attr(attrs, "src").unwrap_or("");
        let w = Self::attr(attrs, "width").and_then(|v| v.parse::<f32>().ok());
        let h = Self::attr(attrs, "height").and_then(|v| v.parse::<f32>().ok());
        tui.ui(|ui| {
            let mut img = egui::Image::new(src);
            if let (Some(w), Some(h)) = (w, h) {
                img = img.fit_to_exact_size(egui::vec2(w, h));
            }
            ui.add(img);
        });
    }

    /// Flatten the retained tree to text for headless assertions.
    #[cfg(any(test, feature = "introspect"))]
    pub fn dump(&self) -> String {
        let mut out = String::new();
        self.dump_node(self.root, &mut out);
        out
    }

    #[cfg(any(test, feature = "introspect"))]
    fn dump_node(&self, idx: usize, out: &mut String) {
        match &self.node_ref(idx).kind {
            NodeKind::Text(s) => {
                out.push_str(s);
                out.push('\n');
            }
            NodeKind::Placeholder => {}
            NodeKind::Element {
                tag: "input", attrs, ..
            } => {
                let kind = Self::attr(attrs, "type").unwrap_or("text");
                let v = match kind {
                    "checkbox" => Self::attr(attrs, "checked").unwrap_or("false"),
                    _ => Self::attr(attrs, "value").unwrap_or(""),
                };
                out.push_str(&format!("[input type={kind} value={v}]\n"));
            }
            NodeKind::Element {
                tag: "select", attrs, ..
            } => {
                let v = Self::attr(attrs, "value").unwrap_or("");
                out.push_str(&format!("[select value={v}]\n"));
            }
            NodeKind::Element {
                tag: "textarea", attrs, ..
            } => {
                let v = Self::attr(attrs, "value").unwrap_or("");
                out.push_str(&format!("[textarea value={v}]\n"));
            }
            NodeKind::Element { children, .. } => {
                for &c in children {
                    self.dump_node(c, out);
                }
            }
        }
    }

    /// Buttons in the tree as (ElementId, label) — used by headless tests to
    /// "click" a button by its visible text.
    #[cfg(any(test, feature = "introspect"))]
    pub fn buttons(&self) -> Vec<(ElementId, String)> {
        let mut out = Vec::new();
        for n in self.arena.iter().flatten() {
            if let NodeKind::Element {
                tag: "button",
                eid: Some(e),
                children,
                ..
            } = &n.kind
            {
                let label = children
                    .iter()
                    .filter_map(|&c| match &self.node_ref(c).kind {
                        NodeKind::Text(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .collect::<String>();
                out.push((ElementId(*e), label));
            }
        }
        out
    }

    /// Value widgets in the tree as (ElementId, type) — used by headless tests
    /// to deliver a value event to a widget without an egui input loop.
    #[cfg(any(test, feature = "introspect"))]
    pub fn inputs(&self) -> Vec<(ElementId, String)> {
        let mut out = Vec::new();
        for n in self.arena.iter().flatten() {
            if let NodeKind::Element {
                tag: "input",
                eid: Some(e),
                attrs,
                ..
            } = &n.kind
            {
                let kind = Self::attr(attrs, "type").unwrap_or("text").to_string();
                out.push((ElementId(*e), kind));
            }
        }
        out
    }
}

impl WriteMutations for DioxusEgui {
    fn append_children(&mut self, id: ElementId, m: usize) {
        let parent = self.node_of(id);
        let kids = self.pop_top(m);
        for &k in &kids {
            self.node_mut(k).parent = Some(parent);
        }
        self.children_mut(parent).extend(kids);
    }

    fn assign_node_id(&mut self, path: &'static [u8], id: ElementId) {
        let base = *self.stack.last().expect("empty stack");
        let target = self.navigate(base, path);
        self.set_element(id, target);
    }

    fn create_placeholder(&mut self, id: ElementId) {
        let idx = self.alloc(Node {
            parent: None,
            kind: NodeKind::Placeholder,
        });
        self.set_element(id, idx);
        self.stack.push(idx);
    }

    fn create_text_node(&mut self, value: &str, id: ElementId) {
        let idx = self.alloc(Node {
            parent: None,
            kind: NodeKind::Text(value.to_string()),
        });
        self.set_element(id, idx);
        self.stack.push(idx);
    }

    fn load_template(&mut self, template: Template, index: usize, id: ElementId) {
        let root = self.build_template_node(&template.roots[index], None);
        self.set_element(id, root);
        self.stack.push(root);
    }

    fn replace_node_with(&mut self, id: ElementId, m: usize) {
        let target = self.node_of(id);
        let new = self.pop_top(m);
        self.replace_in_parent(target, &new);
    }

    fn replace_placeholder_with_nodes(&mut self, path: &'static [u8], m: usize) {
        let new = self.pop_top(m);
        let base = *self.stack.last().expect("empty stack");
        let target = self.navigate(base, path);
        self.replace_in_parent(target, &new);
    }

    fn insert_nodes_after(&mut self, id: ElementId, m: usize) {
        let target = self.node_of(id);
        let parent = self.node_ref(target).parent.expect("no parent");
        let new = self.pop_top(m);
        for &n in &new {
            self.node_mut(n).parent = Some(parent);
        }
        let pos = self
            .children_mut(parent)
            .iter()
            .position(|&c| c == target)
            .expect("target not child")
            + 1;
        self.children_mut(parent).splice(pos..pos, new);
    }

    fn insert_nodes_before(&mut self, id: ElementId, m: usize) {
        let target = self.node_of(id);
        let parent = self.node_ref(target).parent.expect("no parent");
        let new = self.pop_top(m);
        for &n in &new {
            self.node_mut(n).parent = Some(parent);
        }
        let pos = self
            .children_mut(parent)
            .iter()
            .position(|&c| c == target)
            .expect("target not child");
        self.children_mut(parent).splice(pos..pos, new);
    }

    fn set_attribute(&mut self, name: &'static str, _ns: Option<&'static str>, value: &AttributeValue, id: ElementId) {
        let idx = self.node_of(id);
        let v = match value {
            AttributeValue::Text(t) => Some(t.clone()),
            AttributeValue::Float(f) => Some(f.to_string()),
            AttributeValue::Int(i) => Some(i.to_string()),
            AttributeValue::Bool(b) => Some(b.to_string()),
            _ => None,
        };
        if let NodeKind::Element { attrs, .. } = &mut self.node_mut(idx).kind {
            attrs.retain(|(n, _)| n != name);
            if let Some(v) = v {
                attrs.push((name.to_string(), v));
            }
        }
    }

    fn set_node_text(&mut self, value: &str, id: ElementId) {
        let idx = self.node_of(id);
        if let NodeKind::Text(t) = &mut self.node_mut(idx).kind {
            *t = value.to_string();
        }
    }

    fn create_event_listener(&mut self, name: &'static str, id: ElementId) {
        let idx = self.node_of(id);
        if let NodeKind::Element { eid, listeners, .. } = &mut self.node_mut(idx).kind {
            *eid = Some(id.0);
            match name {
                "click" => listeners.click = true,
                "input" => listeners.input = true,
                "change" => listeners.change = true,
                _ => {}
            }
        }
    }

    fn remove_event_listener(&mut self, name: &'static str, id: ElementId) {
        let idx = self.node_of(id);
        if let NodeKind::Element { listeners, .. } = &mut self.node_mut(idx).kind {
            match name {
                "click" => listeners.click = false,
                "input" => listeners.input = false,
                "change" => listeners.change = false,
                _ => {}
            }
        }
    }

    fn remove_node(&mut self, id: ElementId) {
        let target = self.node_of(id);
        if let Some(parent) = self.node_ref(target).parent {
            self.children_mut(parent).retain(|&c| c != target);
        }
    }

    fn push_root(&mut self, id: ElementId) {
        let idx = self.node_of(id);
        self.stack.push(idx);
    }
}

#[cfg(test)]
mod style_tests {
    use super::*;
    use egui_taffy::taffy::{AlignItems, FlexDirection, JustifyContent};

    fn attrs(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn parse_color_forms() {
        assert_eq!(parse_color(Some("#ff8800")), Some(Color32::from_rgb(255, 136, 0)));
        assert_eq!(
            parse_color(Some("#11223344")),
            Some(Color32::from_rgba_unmultiplied(0x11, 0x22, 0x33, 0x44))
        );
        assert_eq!(parse_color(Some("#00000000")).map(|c| c.a()), Some(0));
        assert_eq!(parse_color(None), None);
        assert_eq!(parse_color(Some("nope")), None);
        assert_eq!(parse_color(Some("#abc")), None);
    }

    #[test]
    fn deco_only_when_painting() {
        assert!(Deco::parse(&attrs(&[("radius", "8")])).is_none());
        assert!(Deco::parse(&attrs(&[("gap", "4")])).is_none());

        let d = Deco::parse(&attrs(&[("bg", "#1c2230"), ("radius", "8")])).expect("bg deco");
        assert_eq!(d.bg, Some(Color32::from_rgb(0x1c, 0x22, 0x30)));
        assert_eq!(d.radius, 8);
        assert_eq!(d.border, None);

        let d = Deco::parse(&attrs(&[("border", "#2c3648"), ("border-width", "2")])).expect("border deco");
        assert_eq!(d.border, Some(Color32::from_rgb(0x2c, 0x36, 0x48)));
        assert_eq!(d.border_width, 2.0);
    }

    #[test]
    fn container_style_attrs() {
        let s = container_style(&attrs(&[
            ("dir", "row"),
            ("gap", "12"),
            ("padding", "8"),
            ("width", "200"),
            ("grow", "1"),
            ("align", "start"),
            ("justify", "between"),
        ]));
        assert_eq!(s.flex_direction, FlexDirection::Row);
        assert_eq!(s.flex_grow, 1.0);
        assert_eq!(s.align_items, Some(AlignItems::Start));
        assert_eq!(s.justify_content, Some(JustifyContent::SpaceBetween));
    }

    #[test]
    fn container_style_grid_item_align() {
        let s = container_style(&attrs(&[
            ("display", "grid"),
            ("grid-cols", "label-control"),
            ("align", "start"),
        ]));
        assert_eq!(s.align_items, Some(AlignItems::Start));
        assert_eq!(s.justify_items, Some(AlignItems::Start));
    }

    #[test]
    fn container_style_scroll_overflow() {
        use egui_taffy::taffy::Overflow;

        let s = container_style(&attrs(&[("scroll", "y")]));
        assert_eq!(s.overflow.x, Overflow::Visible);
        assert_eq!(s.overflow.y, Overflow::Scroll);

        let s = container_style(&attrs(&[("scroll", "x")]));
        assert_eq!(s.overflow.x, Overflow::Scroll);
        assert_eq!(s.overflow.y, Overflow::Visible);
    }

    #[test]
    fn text_ctx_inherits_and_overrides() {
        let parent = TextCtx::default().inherit(&attrs(&[("color", "#eaeff6"), ("font-size", "14")]));
        assert_eq!(parent.color, Some(Color32::from_rgb(0xea, 0xef, 0xf6)));
        assert_eq!(parent.size, Some(14.0));
        assert!(!parent.strong);

        // Child overrides size + weight, inherits color.
        let child = parent.inherit(&attrs(&[("font-size", "20"), ("weight", "bold")]));
        assert_eq!(child.color, parent.color);
        assert_eq!(child.size, Some(20.0));
        assert!(child.strong);
    }
}
