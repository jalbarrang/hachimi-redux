//! CM course-parameter resource: per-course distance / surface / set-status
//! thresholds, used to parameterize the [`crate::cm_model`] survival/threshold
//! math for a chosen target course.
//!
//! Loaded once at runtime from `course_params.json` next to the plugin DLL
//! (copied there by the deploy script). Generated offline by the `course-data`
//! tool (`cargo run -p course-data`) from master.mdb (`race_course_set` +
//! `race_course_set_status`). Fetch master.mdb first with `fetch-master-db`.
//!
//! Keeping the data in a sidecar file (not bundled in the DLL) lets it be updated
//! per game version without rebuilding. Mirrors [`crate::eval_data`].

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::cm_model::CourseParams;

static TABLE: OnceLock<Option<HashMap<i32, CourseParams>>> = OnceLock::new();

/// Path to the resource file (next to the plugin DLL / game exe).
fn resource_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("course_params.json")))
        .unwrap_or_else(|| std::path::PathBuf::from("course_params.json"))
}

fn load() -> Option<HashMap<i32, CourseParams>> {
    let path = resource_path();
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            hlog_warn!(target: "training-tracker", "course_data: {} not found ({e})", path.display());
            return None;
        }
    };
    // Keys are strings in JSON; parse to a string map then convert ids to i32.
    let raw: HashMap<String, CourseParams> = match serde_json::from_slice(&bytes) {
        Ok(m) => m,
        Err(e) => {
            hlog_error!(target: "training-tracker", "course_data: parse failed: {e}");
            return None;
        }
    };
    let map: HashMap<i32, CourseParams> = raw
        .into_iter()
        .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v)))
        .collect();
    hlog_info!(target: "training-tracker", "course_data: loaded {} courses", map.len());
    Some(map)
}

/// Lazily-loaded course table; `None` if the resource is missing/invalid.
fn table() -> Option<&'static HashMap<i32, CourseParams>> {
    TABLE.get_or_init(load).as_ref()
}

/// Course parameters for a course id, or `None` when the resource or id is
/// unavailable (so the CM scorer can fall back gracefully).
pub fn course_params(course_id: i32) -> Option<&'static CourseParams> {
    table()?.get(&course_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cm_model::{StatKind, Surface};

    /// Lock the JSON contract between the `course-data` tool's asset and the
    /// plugin's `CourseParams` (field names + enum string values must agree).
    #[test]
    fn asset_shape_deserializes_into_course_params() {
        let sample = r#"{
            "10104": {
                "distance": 2000.0,
                "surface": "Turf",
                "turn": 1,
                "thresholds": ["Power"],
                "finish_time_min": 1171000.0,
                "finish_time_max": 1230000.0
            },
            "10906": {
                "distance": 1600.0,
                "surface": "Dirt",
                "turn": 2,
                "thresholds": [],
                "finish_time_min": 0.0,
                "finish_time_max": 0.0
            }
        }"#;
        let map: HashMap<String, CourseParams> = serde_json::from_str(sample).expect("asset parses");
        let c = &map["10104"];
        assert_eq!(c.distance, 2000.0);
        assert_eq!(c.surface, Surface::Turf);
        assert_eq!(c.set_status_thresholds, vec![StatKind::Power]);
        assert_eq!(map["10906"].surface, Surface::Dirt);
        assert!(map["10906"].set_status_thresholds.is_empty());
    }

    /// The generated resource asset, if present, must parse cleanly.
    #[test]
    fn shipped_asset_parses() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/course_params.json");
        let Ok(bytes) = std::fs::read(path) else {
            return; // asset not generated in this checkout — skip
        };
        let map: HashMap<String, CourseParams> =
            serde_json::from_slice(&bytes).expect("shipped course_params.json parses");
        assert!(!map.is_empty(), "asset should contain courses");
    }
}
