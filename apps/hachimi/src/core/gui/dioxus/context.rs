//! Shared host state for Dioxus Control Center components.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::core::hachimi;

use dioxus::prelude::{MouseEvent, ScopeId, Signal};

use super::super::shell::ControlTab;

/// Side effects queued during a Dioxus frame and drained by the host after render.
#[derive(Debug, Clone)]
pub(crate) enum HostAction {
    OpenThemeEditor,
    OpenDisableOverlayWarning,
    OpenLiveVocalsSwap,
    AutoTranslateWarning,
    SaveConfig,
    RevertConfig,
    CloseMenu,
}

/// Cross-frame context provided to every Control Center Dioxus component.
///
/// `config` stays an `Rc<RefCell<_>>` because native egui islands read/write it
/// outside the Dioxus runtime. Interactive fields use [`Signal`] so component
/// scopes re-run when they change; config edits bump [`Self::revision`].
#[derive(Clone)]
pub(crate) struct ControlCenterCtx {
    pub config: Rc<RefCell<hachimi::Config>>,
    pub active_tab: Signal<ControlTab>,
    pub scale: Signal<f32>,
    pub height: Signal<f32>,
    pub preview_stubs: Signal<bool>,
    pub revision: Signal<u32>,
    pub detached: bool,
    pub keep_open: Rc<Cell<bool>>,
    pub actions: Rc<RefCell<Vec<HostAction>>>,
}

impl ControlCenterCtx {
    /// Create host context with signals bound to the VDOM root scope.
    /// Must be called inside [`dioxus::dioxus_core::VirtualDom::in_runtime`].
    pub fn new_in_runtime() -> Self {
        Self {
            config: Rc::new(RefCell::new(hachimi::Config::default())),
            active_tab: Signal::new_in_scope(ControlTab::default(), ScopeId::ROOT),
            scale: Signal::new_in_scope(1.0, ScopeId::ROOT),
            height: Signal::new_in_scope(600.0, ScopeId::ROOT),
            preview_stubs: Signal::new_in_scope(false, ScopeId::ROOT),
            revision: Signal::new_in_scope(0, ScopeId::ROOT),
            detached: false,
            keep_open: Rc::new(Cell::new(true)),
            actions: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn actions_rc(&self) -> Rc<RefCell<Vec<HostAction>>> {
        Rc::clone(&self.actions)
    }

    /// Mutate the shared config and invalidate config-displaying components.
    pub fn bind<T, F>(&self, f: F) -> impl FnMut(T)
    where
        F: Fn(&mut hachimi::Config, T) + Clone + 'static,
        T: 'static,
    {
        let config = Rc::clone(&self.config);
        let mut revision = self.revision;
        move |v| {
            f(&mut config.borrow_mut(), v);
            revision += 1;
        }
    }

    pub fn bump_revision(&self) {
        let mut revision = self.revision;
        revision += 1;
    }
}

/// Queue a host action from a Dioxus click handler.
pub(crate) fn bind_action(actions: &Rc<RefCell<Vec<HostAction>>>, action: HostAction) -> impl Fn(MouseEvent) {
    let rc = Rc::clone(actions);
    move |_| rc.borrow_mut().push(action.clone())
}
