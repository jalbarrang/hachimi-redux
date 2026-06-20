//! Training Tracker — in-core port of the former `hachimi-training-tracker` cdylib.
//!
//! Tracks training-facility visits and surfaces career analytics overlays/pages.
//! The tracker source moved in near-verbatim against [`compat`], which bridges the
//! old `hachimi-plugin-sdk` surface to host internals. The cdylib `#[hachimi_plugin]`
//! entry point became [`TrainingTracker`], a [`CoreModule`] registered with the host
//! module bootstrap.
//!
//! The `#![allow(...)]` block below carries the lint posture the tracker shipped with
//! as a standalone crate (its `[lints]` table) so the ~15k lines of moved source
//! satisfy the host clippy floor without per-line churn. New code added here should
//! still meet the stricter host bar.
#![allow(
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::as_underscore,
    clippy::fn_to_numeric_cast,
    clippy::fn_to_numeric_cast_any,
    clippy::ptr_as_ptr,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::needless_pass_by_value,
    clippy::missing_safety_doc,
    clippy::missing_transmute_annotations,
    clippy::useless_transmute,
    clippy::transmute_undefined_repr,
    clippy::type_complexity,
    clippy::len_without_is_empty,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::module_name_repetitions,
    clippy::too_many_arguments,
    clippy::wildcard_imports,
    clippy::cast_lossless,
    clippy::used_underscore_binding,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::map_unwrap_or,
    clippy::manual_let_else,
    clippy::unnested_or_patterns,
    clippy::redundant_closure_for_method_calls,
    unnecessary_transmutes,
    function_casts_as_integer
)]

#[macro_use]
pub mod compat;

#[allow(dead_code)]
mod bond_progress;
#[allow(dead_code)]
mod build_profile;
mod career_meta;
mod chara_effects;
mod class_dump;
#[allow(dead_code)]
mod cm_model;
mod config;
#[allow(dead_code)]
mod course_data;
mod deck_bonuses;
mod diagnostics;
mod eval_data;
mod evaluation;
mod gametora_data;
mod hooks;
mod memory_reader;
mod overlay_cache;
mod overlay_prefs;
#[allow(dead_code)]
mod planner;
#[allow(dead_code)]
mod race_context;
mod rank_table;
#[allow(dead_code)]
mod recommend;
mod shop_hooks;
mod skill_shop;
mod skill_shop_prefs;
#[allow(dead_code)]
mod stat_targets;
mod tabs;
mod telemetry;
mod ui;

use compat::Sdk;

use crate::core::plugin::CoreModule;

/// The in-core Training Tracker module.
pub struct TrainingTracker;

impl TrainingTracker {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for TrainingTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreModule for TrainingTracker {
    fn name(&self) -> &str {
        "training-tracker"
    }

    fn init(&mut self) {
        let sdk = Sdk::get();
        hlog_info!(
            target: "training-tracker",
            "Training Tracker (in-core) v{} initializing",
            env!("CARGO_PKG_VERSION")
        );

        config::load();
        // Side-channel telemetry (default disabled via hachimi/telemetry.json).
        hachimi_telemetry::init(sdk.host_data_path("telemetry.json"));
        ui::register_ui();

        // Warm the GameTora catalog off-thread (a 2MB+ parse) so it's ready for
        // tracker features, and surface load diagnostics in the log.
        std::thread::spawn(|| {
            if gametora_data::is_available() {
                hlog_info!(target: "training-tracker", "GameTora catalog ready");
            } else {
                hlog_warn!(
                    target: "training-tracker",
                    "GameTora catalog unavailable (no cache yet)"
                );
            }
        });

        hooks::subscribe_events();
        if shop_hooks::try_install_shop_hooks() {
            hlog_info!(target: "training-tracker", "Skill shop visibility hooks installed");
        }

        hlog_info!(target: "training-tracker", "Training Tracker ready");
        sdk.show_notification("Training Tracker loaded!");
    }

    fn shutdown(&mut self) {
        // IL2CPP hooks are removed by the tracker's SHUTDOWN event subscriber, which
        // the host fires (via `dispatch_shutdown`) just before `module::shutdown_all`.
    }
}
