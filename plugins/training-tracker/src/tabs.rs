//! Overlay tab definitions, in-panel selection, and the user-configurable
//! enabled set.
//!
//! `Tab` enumerates the overlay's content tabs. The selected tab is in-memory
//! only (resets on reload). The *enabled* set is user-configurable and persisted
//! via [`crate::stat_targets`] (`training_config.json`): disabling tabs lets
//! players slim the HUD, and when only one tab is enabled the overlay's tab row
//! is hidden entirely (see `ui::overlay`).

use std::sync::atomic::{AtomicU8, Ordering};

/// Overlay content tabs, in display order.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Tab {
    Training = 0,
    Skills = 1,
    Shop = 2,
    Scenario = 3,
    /// Unified dashboard-style career view (header + training + bonds + skills).
    Career = 4,
}

impl Tab {
    /// All tabs in display order, paired with their tab-bar label.
    pub(crate) const ALL: [(Tab, &'static str); 5] = [
        (Tab::Career, "Career"),
        (Tab::Training, "Training"),
        (Tab::Skills, "Skills"),
        (Tab::Shop, "Shop"),
        (Tab::Scenario, "Scenario"),
    ];

    fn bit(self) -> u8 {
        1 << (self as u8)
    }

    fn from_index(i: u8) -> Tab {
        match i {
            1 => Tab::Skills,
            2 => Tab::Shop,
            3 => Tab::Scenario,
            4 => Tab::Career,
            _ => Tab::Training,
        }
    }
}

/// Bitmask with every tab enabled (the default).
pub(crate) const ALL_ENABLED_MASK: u8 = 0b1_1111;

// Default to the unified Career view so the dashboard-style panel is what users
// see first; it is also force-enabled (see `set_enabled_mask`).
static SELECTED_TAB: AtomicU8 = AtomicU8::new(Tab::Career as u8);
static ENABLED_TABS: AtomicU8 = AtomicU8::new(ALL_ENABLED_MASK);

/// The active tab, resolved against the enabled set: if the stored selection has
/// been disabled, fall back to the first enabled tab so the body is never blank.
pub(crate) fn selected_tab() -> Tab {
    let stored = Tab::from_index(SELECTED_TAB.load(Ordering::Relaxed));
    if is_enabled(stored) {
        return stored;
    }
    enabled_tabs().first().copied().unwrap_or(Tab::Training)
}

pub(crate) fn set_selected_tab(tab: Tab) {
    SELECTED_TAB.store(tab as u8, Ordering::Relaxed);
}

pub(crate) fn is_enabled(tab: Tab) -> bool {
    ENABLED_TABS.load(Ordering::Relaxed) & tab.bit() != 0
}

/// Enable or disable a tab. Refuses to clear the last enabled tab — at least one
/// always stays on. Returns the resulting enabled state of `tab`.
pub(crate) fn set_enabled(tab: Tab, enabled: bool) -> bool {
    // The flagship Career view is always available and cannot be hidden.
    if tab == Tab::Career {
        ENABLED_TABS.fetch_or(Tab::Career.bit(), Ordering::Relaxed);
        return true;
    }
    let mut mask = ENABLED_TABS.load(Ordering::Relaxed);
    if enabled {
        mask |= tab.bit();
    } else if mask & !tab.bit() & ALL_ENABLED_MASK != 0 {
        // Only clear when at least one other tab remains enabled.
        mask &= !tab.bit();
    }
    ENABLED_TABS.store(mask, Ordering::Relaxed);
    mask & tab.bit() != 0
}

/// Number of currently enabled tabs.
pub(crate) fn enabled_count() -> u32 {
    (ENABLED_TABS.load(Ordering::Relaxed) & ALL_ENABLED_MASK).count_ones()
}

/// Enabled tabs in display order.
pub(crate) fn enabled_tabs() -> Vec<Tab> {
    Tab::ALL.iter().map(|(t, _)| *t).filter(|t| is_enabled(*t)).collect()
}

/// The persisted enabled-set bitmask.
pub(crate) fn enabled_mask() -> u8 {
    ENABLED_TABS.load(Ordering::Relaxed) & ALL_ENABLED_MASK
}

/// Restore the enabled set from persisted config. An empty/invalid mask falls
/// back to all-enabled so a corrupt file never hides every tab.
pub(crate) fn set_enabled_mask(mask: u8) {
    // A corrupt/empty mask restores every tab; otherwise force the Career bit on
    // (it is brand new, so older persisted masks never set it) so the unified
    // panel is always reachable.
    let m = mask & ALL_ENABLED_MASK;
    let m = if m == 0 {
        ALL_ENABLED_MASK
    } else {
        m | Tab::Career.bit()
    };
    ENABLED_TABS.store(m, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes tests that mutate the process-global `ENABLED_TABS`/`SELECTED_TAB`
    /// so the parallel test harness can't interleave them.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn cannot_disable_last_tab() {
        let _g = TEST_LOCK.lock().expect("lock poisoned");
        set_enabled_mask(ALL_ENABLED_MASK);
        for (tab, _) in Tab::ALL {
            set_enabled(tab, false);
        }
        // At least one stayed enabled regardless of disable order.
        assert_eq!(enabled_count(), 1);
        set_enabled_mask(ALL_ENABLED_MASK);
    }

    #[test]
    fn selection_falls_back_to_enabled() {
        let _g = TEST_LOCK.lock().expect("lock poisoned");
        set_enabled_mask(ALL_ENABLED_MASK);
        set_selected_tab(Tab::Scenario);
        set_enabled(Tab::Scenario, false);
        assert!(selected_tab() != Tab::Scenario);
        assert!(is_enabled(selected_tab()));
        set_enabled_mask(ALL_ENABLED_MASK);
    }

    #[test]
    fn empty_mask_restores_all() {
        let _g = TEST_LOCK.lock().expect("lock poisoned");
        set_enabled_mask(0);
        assert_eq!(enabled_mask(), ALL_ENABLED_MASK);
    }
}
