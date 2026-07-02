//! Hosted-data sync: download content-hashed JSON snapshots published in this
//! repo (served via raw GitHub / CDN) and cache them under the game data dir for
//! plugins to consume. Generic over a [`DataSet`] descriptor so multiple data
//! sets share one downloader.
//!
//! Three sets are defined:
//! - [`GAMETORA`] — GameTora catalog snapshots (skills / support-cards / events),
//!   extracted in CI by `tools/gametora-sync`, cached under `gametora/`.
//! - [`TRACKER`] — the training-tracker's own generated resources
//!   (`skill_grades.json`, `course_params.json`), regenerated locally by the
//!   maintainer (`tools/skill-grades` / `tools/course-data`) and published under
//!   `data/`. Cached flat in the data-dir root (no subdir).
//! - [`ICONS`] — Career-panel icon sprites (binary PNGs, nested subdirs),
//!   published under `data/icons/` by `tools/icons-manifest`, cached under
//!   `icons/`. The only binary + nested set.
//!
//! Each set has its own published `manifest.json` (`{ generated_at, source,
//! files: { "<file>": "<blake3>" } }`); only files whose hash changed (or whose
//! snapshot is missing) are re-downloaded. A per-set cache manifest tracks the
//! last-synced hashes.

mod cache;
mod client;
mod updater;

pub use updater::Updater;

use rust_i18n::t;

use super::hachimi::Config;

/// Descriptor for one hosted data set. Static so a single generic [`Updater`]
/// instance can serve any set; the function-pointer fields keep the config reads
/// and (literal-keyed) i18n notifications inside each set.
pub struct DataSet {
    /// Log target (e.g. `"gametora_data"`).
    pub log_target: &'static str,
    /// Subdir under the game data dir where snapshots cache; `""` = data-dir root.
    pub subdir: &'static str,
    /// Filename of the local cache manifest within the cache dir.
    pub cache_filename: &'static str,
    /// Default hosted base URL (no trailing slash).
    pub default_url: &'static str,
    /// Whether this set is disabled (config toggle).
    pub is_disabled: fn(&Config) -> bool,
    /// Optional base-URL override (config; dev/testing).
    pub url_override: fn(&Config) -> Option<String>,
    /// `true` for binary snapshots (e.g. icon PNGs): fetched as raw bytes with no
    /// JSON validation, integrity checked via the manifest hash. `false` (JSON)
    /// for the catalog/resource sets.
    pub binary: bool,
    /// `true` if manifest keys may be `/`-separated relative paths (nested
    /// subdirs), validated with `is_safe_relpath`. `false` keeps names flat.
    pub allow_subdirs: bool,
    /// Optional hook run after a successful sync that wrote ≥ 1 file (arg =
    /// number updated). Used by the icon set to drop the texture negative-cache
    /// so freshly-downloaded icons appear without a restart.
    pub on_synced: Option<fn(usize)>,
    /// "Syncing…" persistent message.
    pub msg_syncing: fn() -> String,
    /// "Up to date" message.
    pub msg_up_to_date: fn() -> String,
    /// "Sync complete (N updated)" message.
    pub msg_complete: fn(usize) -> String,
    /// "Sync failed: reason" message.
    pub msg_failed: fn(&str) -> String,
}

/// GameTora catalog snapshots (`data/gametora/`), cached under `gametora/`.
pub static GAMETORA: DataSet = DataSet {
    log_target: "gametora_data",
    subdir: hachimi_plugin_abi::GAMETORA_DATA_SUBDIR,
    cache_filename: ".gametora_cache.json",
    default_url: "https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data/gametora",
    is_disabled: |c| c.disable_gametora_data,
    url_override: |c| c.gametora_data_url.clone(),
    binary: false,
    allow_subdirs: false,
    on_synced: None,
    msg_syncing: || t!("notification.gametora_syncing").to_string(),
    msg_up_to_date: || t!("notification.gametora_up_to_date").to_string(),
    msg_complete: |count| t!("notification.gametora_sync_complete", count = count).to_string(),
    msg_failed: |reason| t!("notification.gametora_sync_failed", reason = reason).to_string(),
};

/// Training-tracker generated resources (`data/`), cached flat in the data-dir
/// root: `skill_grades.json`, `course_params.json`.
pub static TRACKER: DataSet = DataSet {
    log_target: "tracker_data",
    subdir: "",
    cache_filename: ".tracker_cache.json",
    default_url: "https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data",
    is_disabled: |c| c.disable_tracker_data,
    url_override: |c| c.tracker_data_url.clone(),
    binary: false,
    allow_subdirs: false,
    on_synced: None,
    msg_syncing: || t!("notification.data_syncing", name = tracker_name()).to_string(),
    msg_up_to_date: || t!("notification.data_up_to_date", name = tracker_name()).to_string(),
    msg_complete: |count| t!("notification.data_sync_complete", name = tracker_name(), count = count).to_string(),
    msg_failed: |reason| t!("notification.data_sync_failed", name = tracker_name(), reason = reason).to_string(),
};

fn tracker_name() -> String {
    t!("notification.tracker_data_name").to_string()
}

/// Career-panel icon sprites (`data/icons/**`), cached under `icons/`. Binary +
/// nested (subdirs `chara/`, `statusrank/`). These are the ~16 MB of game UI
/// sprites the Career panel loads on demand via `host_data_path("icons/…")`.
pub static ICONS: DataSet = DataSet {
    log_target: "icons_data",
    subdir: "icons",
    cache_filename: ".icons_cache.json",
    default_url: "https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data/icons",
    is_disabled: |c| c.disable_icons_data,
    url_override: |c| c.icons_data_url.clone(),
    binary: true,
    allow_subdirs: true,
    on_synced: Some(|_updated| {
        crate::core::modules::training_tracker::clear_icon_cache();
    }),
    msg_syncing: || t!("notification.data_syncing", name = icons_name()).to_string(),
    msg_up_to_date: || t!("notification.data_up_to_date", name = icons_name()).to_string(),
    msg_complete: |count| t!("notification.data_sync_complete", name = icons_name(), count = count).to_string(),
    msg_failed: |reason| t!("notification.data_sync_failed", name = icons_name(), reason = reason).to_string(),
};

fn icons_name() -> String {
    t!("notification.icons_data_name").to_string()
}

#[cfg(test)]
mod tests {
    use super::{GAMETORA, ICONS, TRACKER};

    #[test]
    fn json_sets_are_flat_text() {
        for set in [&GAMETORA, &TRACKER] {
            assert!(!set.binary, "{} must fetch as text/JSON", set.log_target);
            assert!(!set.allow_subdirs, "{} must stay flat", set.log_target);
            assert!(set.on_synced.is_none());
        }
    }

    #[test]
    fn icons_set_is_binary_nested_with_hook() {
        assert!(ICONS.binary, "icons must fetch as raw bytes");
        assert!(ICONS.allow_subdirs, "icons need nested paths (chara/, statusrank/)");
        assert_eq!(ICONS.subdir, "icons");
        assert!(
            ICONS.on_synced.is_some(),
            "icons sync must invalidate the texture cache"
        );
    }
}
