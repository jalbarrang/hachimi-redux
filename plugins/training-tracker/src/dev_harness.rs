//! Desktop preview harness for the overlay UI.
//!
//! Renders the *exact same* overlay panel as the in-game plugin, but inside a
//! plain `eframe` window driven by mocked career data — no game process, no
//! IL2CPP, no SDK. This lets you iterate on layout, spacing, zoom and styling
//! with a ~1s rebuild instead of launching the Honse game.
//!
//! Run it with:
//!
//! ```sh
//! cargo run -p hachimi-training-tracker --example overlay_preview --features dev-harness
//! ```
//!
//! Caveats vs. in-game: textures/icons that come from the host data dir won't
//! load (they degrade to placeholders), fonts/DPI differ slightly from the host,
//! and the live memory-read path is not exercised. Everything layout-related is
//! faithful because it is the same draw code on the same egui version.

use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use hachimi_plugin_sdk::egui;

use crate::evaluation::Aptitudes;
use crate::memory_reader::{self, AcquiredSkillInfo, CareerSnapshot, EvaluationInfo};

/// Build a realistic career snapshot mirroring a late-game Seiun Sky run, so the
/// overlay tabs show plausible content to design against.
fn mock_snapshot() -> CareerSnapshot {
    CareerSnapshot {
        is_playing: true,
        current_turn: 67,
        month: 10,
        speed: 784,
        stamina: 644,
        power: 760,
        guts: 459,
        wiz: 822,
        total_stats: 784 + 644 + 760 + 459 + 822,
        hp: 0,
        max_hp: 100,
        motivation: 5, // GREAT
        card_id: 0,
        star: 4,
        // Speed L2, Stamina L1, Power L3, Guts L1, Wisdom L4.
        training_levels: [2, 1, 3, 1, 4],
        stat_caps: [1100, 1100, 1100, 1100, 1100],
        aptitudes: Aptitudes {
            dist_short: 6,
            dist_mile: 7,
            dist_middle: 8,
            dist_long: 8,
            style_nige: 7,
            style_senko: 8,
            style_sashi: 6,
            style_oikomi: 4,
            ground_turf: 8,
            ground_dirt: 3,
        },
        evaluation_value: Some(8991),
        // Per-facility failure %, mirroring the screenshot's 99/99/99/99/81.
        failure_rates: [99, 99, 99, 99, 81],
        // Per-facility total stat gain (the "Total" row).
        stat_gains: [18, 12, 22, 28, 31],
        per_stat_gains: [
            [13, 1, 2, 0, 2],
            [1, 9, 1, 1, 0],
            [2, 1, 13, 4, 2],
            [2, 1, 4, 12, 0],
            [3, 1, 2, 0, 25],
        ],
        ..Default::default()
    }
}

fn mock_skills() -> Vec<AcquiredSkillInfo> {
    [
        ("Professor of Curvature", 1),
        ("Corner Recovery ◎", 1),
        ("Straightaway Acceleration", 1),
        ("Final Push", 2),
        ("Late Surger Savvy", 1),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (name, level))| AcquiredSkillInfo {
        master_id: 200000 + i as i32,
        level,
        name: name.to_string(),
    })
    .collect()
}

fn mock_evaluations() -> Vec<EvaluationInfo> {
    [
        ("Kitasan Black", 100),
        ("Fine Motion", 100),
        ("Yaeno Muteki", 100),
        ("Nice Nature", 96),
        ("Marvelous Sunday", 89),
        ("Shinko Windy", 72),
        ("Director Akikawa", 100),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (name, value))| EvaluationInfo {
        target_id: i as i32 + 1,
        value,
        is_appear: true,
        name: name.to_string(),
        story_step: 0,
        guest_chara_id: 0,
    })
    .collect()
}

/// Locate an on-disk `icons/` root so the preview can render real game sprites
/// (portraits, rank/stat icons) instead of text fallbacks. Tries, in order:
///
/// 1. `TT_PREVIEW_ICONS` — explicit override pointing straight at an icons dir.
/// 2. `HONSE_ICONS_DIR` — same source the deploy script stages from.
/// 3. `<HACHIMI_GAME_DIR>/hachimi/icons` — the deployed location.
/// 4. sibling `../honse-tracker/apps/web/public/icons` checkout.
///
/// Returns the first directory that exists; `None` if none are found (the overlay
/// then just draws its text/colored fallbacks).
fn resolve_icon_root() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(p) = std::env::var("TT_PREVIEW_ICONS") {
        candidates.push(PathBuf::from(p));
    }
    if let Ok(p) = std::env::var("HONSE_ICONS_DIR") {
        candidates.push(PathBuf::from(p));
    }
    if let Ok(game) = std::env::var("HACHIMI_GAME_DIR") {
        candidates.push(PathBuf::from(game).join("hachimi").join("icons"));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("honse-tracker")
            .join("apps")
            .join("web")
            .join("public")
            .join("icons"),
    );
    candidates.into_iter().find(|p| p.is_dir())
}

/// Push the mocked dataset into the overlay cache and flip tracking on, so the
/// overlay renders its populated state instead of the "tracking off" hint.
fn install_mock_data() {
    memory_reader::TRACKING.store(true, Ordering::Relaxed);
    crate::overlay_cache::set_test_data(
        mock_snapshot(),
        mock_skills(),
        mock_evaluations(),
        Vec::new(), // skill shop entries
        Some(1200), // skill points
        // Equipped (deck slot, support_card_id) pairs — ids are illustrative.
        vec![(1, 30001), (2, 30002), (3, 30003), (4, 30004), (5, 30005), (6, 30006)],
    );
}

struct Harness;

impl eframe::App for Harness {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Mid-grey backdrop so the overlay's own rounded panel reads clearly.
        [0.12, 0.12, 0.14, 1.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // The overlay panel draws itself top-down; wrap in a scroll area so tall
            // tabs (e.g. Career) stay reachable in a small window.
            egui::ScrollArea::both().show(ui, |ui| {
                // Contain any panic from an SDK-dependent code path so one bad tab
                // never kills the whole preview window.
                let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    crate::ui::draw_overlay_for_harness(ui);
                }));
                if res.is_err() {
                    ui.colored_label(
                        egui::Color32::LIGHT_RED,
                        "this tab panicked (likely needs the live game SDK) — try another tab",
                    );
                }
            });
        });
    }
}

/// Entry point invoked by the `overlay_preview` example.
pub fn run() -> eframe::Result {
    install_mock_data();

    match resolve_icon_root() {
        Some(root) => {
            eprintln!("[overlay_preview] icons: {}", root.display());
            crate::ui::set_harness_icon_root(root);
        }
        None => eprintln!(
            "[overlay_preview] no icons dir found — set TT_PREVIEW_ICONS or HONSE_ICONS_DIR \
             to render real sprites (falling back to text)"
        ),
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Training Tracker — overlay preview")
            .with_inner_size([600.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "training-tracker-overlay-preview",
        options,
        Box::new(|_cc| Ok(Box::new(Harness))),
    )
}
