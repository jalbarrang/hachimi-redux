//! Visual primitives for the Career panel, matching the honse-tracker dashboard's
//! "uma-kit" look: a green striped section header, raised dark card faces, pill
//! chips, stat-colored icon chips, and the rainbow-ready border. Colors mirror
//! the dashboard CSS tokens (`apps/web/src/index.css`).
//!
//! Everything here is pure egui painting — no game/IL2CPP access.

use hachimi_plugin_sdk::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};

use super::super::textures;
use crate::career_meta;

// ── Palette (dashboard --color-* tokens) ──────────────────────────────────
pub const SURFACE_1: Color32 = Color32::from_rgb(0x15, 0x1a, 0x23);
pub const SURFACE_2: Color32 = Color32::from_rgb(0x1c, 0x22, 0x30);
pub const SURFACE_3: Color32 = Color32::from_rgb(0x24, 0x2c, 0x3d);
pub const LINE: Color32 = Color32::from_rgb(0x2c, 0x36, 0x48);
pub const FG: Color32 = Color32::from_rgb(0xea, 0xef, 0xf6);
pub const FG_MUTED: Color32 = Color32::from_rgb(0xa3, 0xb1, 0xc4);
pub const FG_DIM: Color32 = Color32::from_rgb(0x6e, 0x7d, 0x92);

pub const UMA_300: Color32 = Color32::from_rgb(0x8f, 0xe0, 0x8f);
pub const UMA_400: Color32 = Color32::from_rgb(0x6f, 0xd0, 0x6f);
pub const GRADE_A: Color32 = Color32::from_rgb(0xff, 0x7a, 0x6b);

pub const STAT_SPEED: Color32 = Color32::from_rgb(0x5f, 0xb2, 0xff);
pub const STAT_STAMINA: Color32 = Color32::from_rgb(0xff, 0x8a, 0x5c);
pub const STAT_POWER: Color32 = Color32::from_rgb(0xff, 0xb0, 0x4d);
pub const STAT_GUTS: Color32 = Color32::from_rgb(0xff, 0x7a, 0x90);
pub const STAT_WIT: Color32 = Color32::from_rgb(0x4d, 0xdc, 0xb0);

const STRIP_TOP: Color32 = Color32::from_rgb(0x58, 0xc4, 0x54);
const STRIP_BOTTOM: Color32 = Color32::from_rgb(0x3f, 0xae, 0x3c);

/// Gold used for the rank-badge ring and filled stars.
pub const GOLD: Color32 = Color32::from_rgb(0xf0, 0xa8, 0x18);

/// Mood accent color for a motivation level (1 Awful … 5 Great); mirrors the
/// dashboard `--color-mood-*` tokens. Out-of-range → muted.
#[must_use]
pub fn mood_color(motivation: i32) -> Color32 {
    match motivation {
        5 => Color32::from_rgb(0xe8, 0x5f, 0x9c), // great
        4 => Color32::from_rgb(0xff, 0x9a, 0x3d), // good
        3 => Color32::from_rgb(0xc2, 0xa8, 0x3d), // normal
        2 => Color32::from_rgb(0x4d, 0x8f, 0xd6), // bad
        1 => Color32::from_rgb(0xa8, 0x6f, 0xd6), // awful
        _ => FG_MUTED,
    }
}

/// Stat-type accent color for a facility index (0 Speed … 4 Wit).
#[must_use]
pub fn stat_color(facility: usize) -> Color32 {
    [STAT_SPEED, STAT_STAMINA, STAT_POWER, STAT_GUTS, STAT_WIT]
        .get(facility)
        .copied()
        .unwrap_or(SURFACE_3)
}

/// Paint a vertical two-stop gradient inside `rect` (rounded `corner`), top→bottom.
pub fn vgrad(ui: &Ui, rect: Rect, top: Color32, bottom: Color32, corner: u8) {
    // A simple N-band fill approximates the gradient cheaply and crisply.
    const BANDS: usize = 12;
    let painter = ui.painter();
    for i in 0..BANDS {
        let t0 = i as f32 / BANDS as f32;
        let t1 = (i + 1) as f32 / BANDS as f32;
        let y0 = rect.top() + rect.height() * t0;
        let y1 = rect.top() + rect.height() * t1;
        let c = lerp_color(top, bottom, (t0 + t1) * 0.5);
        let band = Rect::from_min_max(Pos2::new(rect.left(), y0), Pos2::new(rect.right(), y1));
        // Only the first/last band carry the rounding so the seam stays square.
        let r = if i == 0 {
            CornerRadius {
                nw: corner,
                ne: corner,
                sw: 0,
                se: 0,
            }
        } else if i == BANDS - 1 {
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: corner,
                se: corner,
            }
        } else {
            CornerRadius::ZERO
        };
        painter.rect_filled(band, r, c);
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

/// The green striped section header with the `//` slash accent. `trailing` is a
/// small right-aligned caption (e.g. "1429 SP · 5"), empty for none.
pub fn section_strip(ui: &mut Ui, label: &str, trailing: &str) {
    let height = (ui.text_style_height(&egui::TextStyle::Body) + 8.0).max(22.0);
    // Deterministic width (not ui.available_width(), which inflates under the
    // host's auto_sized window) so the strip can't grow the panel.
    let width = super::super::overlay::content_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
    let painter = ui.painter();

    // Base gradient + rounded clip.
    vgrad(ui, rect, STRIP_TOP, STRIP_BOTTOM, 6);
    // Diagonal highlight stripes, masked toward the right.
    let clip = painter.with_clip_rect(rect);
    let step = 14.0;
    let mut x = rect.left() - rect.height();
    while x < rect.right() + rect.height() {
        let alpha = (((x - rect.left()) / rect.width()).clamp(0.0, 1.0) * 36.0) as u8;
        let top = Pos2::new(x + rect.height() * 0.5, rect.top());
        let bot = Pos2::new(x - rect.height() * 0.5, rect.bottom());
        clip.line_segment([top, bot], Stroke::new(3.0, Color32::from_white_alpha(alpha)));
        x += step;
    }
    // Top inner highlight line.
    painter.line_segment(
        [
            Pos2::new(rect.left() + 4.0, rect.top() + 1.0),
            Pos2::new(rect.right() - 4.0, rect.top() + 1.0),
        ],
        Stroke::new(1.0, Color32::from_white_alpha(40)),
    );

    // Label (left) + trailing caption (right).
    painter.text(
        Pos2::new(rect.left() + 12.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(height * 0.6),
        Color32::WHITE,
    );
    if !trailing.is_empty() {
        painter.text(
            Pos2::new(rect.right() - 14.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            trailing,
            egui::FontId::proportional(height * 0.46),
            Color32::from_white_alpha(220),
        );
    }
}

/// A small raised pill chip; `add` draws its inline contents.
pub fn pill(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 5))
        .corner_radius(CornerRadius::same(8))
        .fill(SURFACE_2)
        .stroke(Stroke::new(1.0, LINE))
        .show(ui, |ui| {
            ui.horizontal(|ui| add(ui));
        });
}

/// Frame for a bond row: rainbow border when `rainbow`, else the raised face.
pub fn row_frame(rainbow: bool) -> egui::Frame {
    let stroke = if rainbow {
        Stroke::new(1.5, Color32::from_rgb(0x9a, 0x8c, 0xff))
    } else {
        Stroke::new(1.0, LINE)
    };
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 6))
        .corner_radius(CornerRadius::same(8))
        .fill(SURFACE_2)
        .stroke(stroke)
}

/// A stat-colored rounded chip with the stat glyph centered, side `size` px.
/// Falls back to a colored chip with the facility's initial when the icon sprite
/// is unavailable.
pub fn stat_chip(ui: &mut Ui, facility: usize, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), stat_color(facility));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0, Color32::from_black_alpha(40)),
        StrokeKind::Inside,
    );
    let icon = career_meta::stat_icon_path(facility);
    if let Some(tex) = textures::texture(ui.ctx(), &icon) {
        let pad = size * 0.12;
        let inner = rect.shrink(pad);
        ui.painter().image(
            tex.id(),
            inner,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        let label = ["S", "St", "P", "G", "W"].get(facility).copied().unwrap_or("?");
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(size * 0.6),
            Color32::from_black_alpha(180),
        );
    }
}
