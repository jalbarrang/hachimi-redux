//! Plugin notification queue shared with the GUI layer.
//! Plugins may enqueue messages from any thread through a `Lazy<Mutex<_>>` buffer.
//! The render thread drains the queue in batches before displaying notifications.

use std::sync::Mutex;

use once_cell::sync::Lazy;

static PLUGIN_NOTIFICATIONS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn enqueue(message: String) {
    PLUGIN_NOTIFICATIONS.lock().expect("lock poisoned").push(message);
}

/// Whether any plugin notifications are waiting to be drained. Lets the render
/// hook keep painting (and thus draining) even when the GUI is otherwise empty,
/// so notifications queued before the menu is ever opened still surface.
pub(crate) fn has_pending() -> bool {
    !PLUGIN_NOTIFICATIONS.lock().expect("lock poisoned").is_empty()
}

pub(crate) fn drain() -> Vec<String> {
    let mut notifications = PLUGIN_NOTIFICATIONS.lock().expect("lock poisoned");
    std::mem::take(&mut *notifications)
}
