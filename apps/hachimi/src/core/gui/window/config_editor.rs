use std::sync::Arc;

use crate::core::hachimi;
use crate::core::Hachimi;

pub(crate) struct ConfigEditor {
    last_ptr_config: usize,
    config: hachimi::Config,
    /// Desktop-preview mode: no `Hachimi::instance()` backing store. `sync`/
    /// `revert`/Save become inert and game-data combos use static placeholders,
    /// so the editor renders against a plain in-memory config off-game.
    detached: bool,
}

impl ConfigEditor {
    pub fn new() -> ConfigEditor {
        let handle = Hachimi::instance().config.load();
        ConfigEditor {
            last_ptr_config: Arc::as_ptr(&handle) as usize,
            config: (**Hachimi::instance().config.load()).clone(),
            detached: false,
        }
    }

    /// Read the working-copy config (preview harness uses this to mirror the GUI
    /// scale into the egui context).
    #[cfg(feature = "dev-harness")]
    pub(crate) fn working_config(&self) -> &hachimi::Config {
        &self.config
    }

    /// Build a detached editor for the desktop preview harness: backed by the
    /// given in-memory config, with no `Hachimi::instance()` coupling.
    #[cfg(feature = "dev-harness")]
    pub(crate) fn new_detached(config: hachimi::Config) -> ConfigEditor {
        ConfigEditor {
            last_ptr_config: 0,
            config,
            detached: true,
        }
    }

    /// Discard unsaved edits: reset the working copy to the currently saved config
    /// and re-apply its language locale (the language combo applies locale live).
    fn revert(&mut self) {
        if self.detached {
            self.config = hachimi::Config::default();
            self.config.language.set_locale();
            return;
        }
        let handle = Hachimi::instance().config.load();
        self.last_ptr_config = Arc::as_ptr(&handle) as usize;
        self.config = (**handle).clone();
        self.config.language.set_locale();
    }

    /// Discard unsaved edits (public for Dioxus footer Cancel).
    pub(crate) fn revert_edits(&mut self) {
        self.revert();
    }

    pub(crate) fn config_mut(&mut self) -> &mut hachimi::Config {
        &mut self.config
    }

    pub(crate) fn config(&self) -> &hachimi::Config {
        &self.config
    }

    pub(crate) fn is_detached(&self) -> bool {
        self.detached
    }

    /// Sync the working copy if the saved config changed underneath us.
    pub(crate) fn sync(&mut self) {
        if self.detached {
            return;
        }
        let global_handle = Hachimi::instance().config.load();
        let global_ptr = Arc::as_ptr(&global_handle) as usize;
        if global_ptr != self.last_ptr_config {
            self.config = (**global_handle).clone();
            self.last_ptr_config = global_ptr;
        }
        #[cfg(target_os = "windows")]
        {
            self.config.windows.menu_open_key = global_handle.windows.menu_open_key;
        }
    }
}
