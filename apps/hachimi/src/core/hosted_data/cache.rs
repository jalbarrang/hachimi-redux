//! On-disk cache manifest tracking the last-synced hash per snapshot filename.
//!
//! The set of files to fetch is driven entirely by the hosted `manifest.json`;
//! the runtime just mirrors whatever filenames that manifest lists, after
//! sanitizing them. The cache-manifest filename is per-set ([`super::DataSet`]).

use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

/// Persisted record of the last successful sync: content hash per filename.
#[derive(Serialize, Deserialize, Default)]
pub(super) struct CacheManifest {
    #[serde(default)]
    pub(super) synced_at: String,
    #[serde(default)]
    pub(super) files: FnvHashMap<String, String>,
}

/// Reject filenames that could escape the cache dir or nest into subdirs.
/// Flat hosted sets use this (e.g. `skills.json`); nested sets use
/// [`is_safe_relpath`].
pub(super) fn is_safe_filename(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !std::path::Path::new(name).is_absolute()
}

/// Accept a `/`-separated relative path for sets that allow subdirs (e.g. the
/// Career icon set: `chara/chr_icon_1001.png`), while still rejecting anything
/// that could escape the cache dir. Backslashes are disallowed so manifests stay
/// portable and Windows can't be tricked with `..\`.
pub(super) fn is_safe_relpath(name: &str) -> bool {
    if name.is_empty() || name.contains('\\') || std::path::Path::new(name).is_absolute() {
        return false;
    }
    // Every `/`-separated component must be a plain, non-traversing name.
    let mut any = false;
    for comp in name.split('/') {
        if comp.is_empty() || comp == "." || comp == ".." {
            return false;
        }
        any = true;
    }
    any
}

#[cfg(test)]
mod tests {
    use super::{is_safe_filename, is_safe_relpath};

    #[test]
    fn flat_names_stay_flat() {
        assert!(is_safe_filename("skills.json"));
        assert!(!is_safe_filename("chara/x.png"));
        assert!(!is_safe_filename(".."));
    }

    #[test]
    fn relpath_allows_nested_rejects_traversal() {
        assert!(is_safe_relpath("10011.png"));
        assert!(is_safe_relpath("chara/chr_icon_1001.png"));
        assert!(is_safe_relpath("statusrank/ui_statusrank_08.png"));
        // Traversal / escapes / junk.
        assert!(!is_safe_relpath("../secret"));
        assert!(!is_safe_relpath("chara/../../etc/passwd"));
        assert!(!is_safe_relpath("a//b.png"));
        assert!(!is_safe_relpath("chara\\x.png"));
        assert!(!is_safe_relpath("/abs/path.png"));
        assert!(!is_safe_relpath(""));
        assert!(!is_safe_relpath("."));
        assert!(!is_safe_relpath("trailing/"));
    }
}
