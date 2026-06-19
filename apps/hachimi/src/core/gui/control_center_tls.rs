//! Thread-local Control Center Dioxus mount (`VirtualDom` is `!Send`).

use std::cell::RefCell;

use super::control_center_mount::ControlCenterMount;

thread_local! {
    static MOUNT: RefCell<Option<ControlCenterMount>> = const { RefCell::new(None) };
}

pub(crate) fn with_mount<R>(f: impl FnOnce(&mut ControlCenterMount) -> R) -> R {
    MOUNT.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            *slot = Some(ControlCenterMount::new());
        }
        f(slot.as_mut().expect("mount"))
    })
}
