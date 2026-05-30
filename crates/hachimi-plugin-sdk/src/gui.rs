//! GUI helpers. With the shared-`egui::Ui` model, plugins draw by casting the
//! host-provided `Ui` pointer and calling egui directly — there are no per-widget
//! vtable slots. Host *services* (notifications, overlay visibility) stay on `Sdk`.

use std::ffi::{c_void, CString};

use hachimi_plugin_abi::vt;

use crate::Sdk;

/// Cast a host-provided `Ui` pointer (handed to a menu/overlay/section callback)
/// to a real [`egui::Ui`].
///
/// # Safety
/// `ptr` must be the non-null `*mut c_void` passed by the host into a GUI callback,
/// and the returned reference must not outlive that callback invocation.
#[must_use]
pub unsafe fn ui_from_ptr<'a>(ptr: *mut c_void) -> &'a mut egui::Ui {
    // SAFETY: caller guarantees `ptr` is the host's live `&mut egui::Ui` for this callback.
    unsafe { &mut *(ptr as *mut egui::Ui) }
}

impl Sdk {
    /// Show a host notification toast.
    pub fn show_notification(&self, message: &str) -> bool {
        let Ok(msg_c) = CString::new(message) else {
            return false;
        };
        // SAFETY: host vtable slot is valid after init.
        unsafe { (vt().gui_show_notification)(msg_c.as_ptr()) }
    }

    /// Toggle an overlay's visibility by the id it was registered with.
    pub fn overlay_set_visible(&self, id: &str, visible: bool) -> bool {
        let Ok(id_c) = CString::new(id) else {
            return false;
        };
        // SAFETY: overlay id registered earlier with the same string.
        unsafe { (vt().gui_overlay_set_visible)(id_c.as_ptr(), visible) }
    }
}
