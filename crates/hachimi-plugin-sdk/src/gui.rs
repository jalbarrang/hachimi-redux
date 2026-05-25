//! GUI helpers built on the plugin vtable.

use std::ffi::{c_char, c_void, CString};

use hachimi_plugin_abi::{vt, GuiUiCallback};

use crate::Sdk;

impl Sdk {
    pub fn gui_colored_label(&self, ui: *mut c_void, r: u8, g: u8, b: u8, a: u8, text: &str) -> bool {
        let Ok(text_c) = CString::new(text) else {
            return false;
        };
        // SAFETY: `ui` from host overlay/menu callback.
        unsafe { (vt().gui_ui_colored_label)(ui, r, g, b, a, text_c.as_ptr()) }
    }

    pub fn gui_small_button(&self, ui: *mut c_void, text: &str) -> bool {
        let Ok(text_c) = CString::new(text) else {
            return false;
        };
        // SAFETY: `ui` from host callback.
        unsafe { (vt().gui_ui_small_button)(ui, text_c.as_ptr()) }
    }

    /// Single-line text edit; `buffer` must be NUL-terminated within `buffer.len()`.
    pub fn gui_text_edit_singleline(&self, ui: *mut c_void, buffer: &mut [u8]) -> bool {
        if buffer.is_empty() {
            return false;
        }
        // SAFETY: `ui` from host callback; buffer valid for callback duration.
        unsafe { (vt().gui_ui_text_edit_singleline)(ui, buffer.as_mut_ptr() as *mut c_char, buffer.len()) }
    }

    pub fn gui_collapsing(
        &self,
        ui: *mut c_void,
        heading: &str,
        default_open: bool,
        callback: GuiUiCallback,
        userdata: *mut c_void,
    ) -> bool {
        let Ok(heading_c) = CString::new(heading) else {
            return false;
        };
        // SAFETY: Callback lifetime managed by plugin until host unloads.
        unsafe { (vt().gui_ui_collapsing)(ui, heading_c.as_ptr(), default_open, Some(callback), userdata) }
    }

    pub fn gui_horizontal(&self, ui: *mut c_void, callback: GuiUiCallback, userdata: *mut c_void) -> bool {
        // SAFETY: Callback lifetime managed by plugin.
        unsafe { (vt().gui_ui_horizontal)(ui, Some(callback), userdata) }
    }

    pub fn gui_separator(&self, ui: *mut c_void) -> bool {
        // SAFETY: `ui` from host overlay/menu callback.
        unsafe { (vt().gui_ui_separator)(ui) }
    }

    pub fn gui_button(&self, ui: *mut c_void, text: &str) -> bool {
        let Ok(text_c) = CString::new(text) else {
            return false;
        };
        // SAFETY: `ui` from host callback.
        unsafe { (vt().gui_ui_button)(ui, text_c.as_ptr()) }
    }

    pub fn gui_checkbox(&self, ui: *mut c_void, text: &str, value: &mut bool) -> bool {
        let Ok(text_c) = CString::new(text) else {
            return false;
        };
        // SAFETY: `ui` from host callback; `value` lives for callback duration.
        unsafe { (vt().gui_ui_checkbox)(ui, text_c.as_ptr(), value) }
    }

    pub fn gui_set_min_width(&self, ui: *mut c_void, width: f32) -> bool {
        // SAFETY: `ui` from host callback.
        unsafe { (vt().gui_ui_set_min_width)(ui, width) }
    }

    pub fn gui_set_font_size(&self, ui: *mut c_void, size: f32) -> bool {
        // SAFETY: `ui` from host callback.
        unsafe { (vt().gui_ui_set_font_size)(ui, size) }
    }

    pub fn overlay_set_visible(&self, id: &str, visible: bool) -> bool {
        let Ok(id_c) = CString::new(id) else {
            return false;
        };
        // SAFETY: Overlay id registered earlier with same string.
        unsafe { (vt().gui_overlay_set_visible)(id_c.as_ptr(), visible) }
    }
}
