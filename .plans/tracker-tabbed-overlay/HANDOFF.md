## Goal (beads: Hachimi-Edge-wnb; rank follow-up deferred to Hachimi-Edge-2we)
Reorganize the **training-tracker L2 overlay panel** from collapsing headers into an **in-panel tab bar** so it fits the small floating panel. Tabs: **Training / Skills / Bonds / Skill Shop**. Scope = tabs 1-5 only. The overall evaluation **Rank** is a separate research-gated follow-up (Hachimi-Edge-2we) — the Training tab reserves a Rank cell that shows `—` until then.

## Where this lives
- `plugins/training-tracker/src/ui.rs` — the overlay render path. Currently `draw_overlay` → `draw_overlay_inner` → `draw_overlay_memory` (memory-read mode) / `draw_overlay_hooks` (fallback). `draw_overlay_memory` today renders stats inline + three `egui::CollapsingHeader`s (Skills/Bonds/Skill Shop) via `draw_skills_panel`, `draw_bonds_panel`, `draw_skill_shop_panel`, `draw_skill_shop_controls`.
- This is the **L2 panel** body (the host wraps it in the floating window/badge per the L1/L2 overhaul). The plugin only draws into the provided `&mut egui::Ui`.
- The L1 Plugins page (`draw_menu_section`) keeps its controls (start/stop tracking, dump classes, show-overlay) — **out of scope**, leave as-is.

## Data sources (all already available; no new IL2CPP work for tabs 1-5)
- `crate::overlay_cache`: `maybe_request_refresh()`, `snapshot() -> Option<CareerSnapshot>`, `skills()`, `evaluations()`, `skill_shop()`, `skill_points() -> Option<i32>`.
- `CareerSnapshot` (memory_reader/snapshot.rs): `is_playing, current_turn, month, speed, stamina, power, guts, wiz, total_stats, hp, max_hp, motivation, fan_count, total_races, win_count, training_levels[5]` ([Speed,Stamina,Power,Guts,Wisdom]).
- Skill shop helpers in `crate::skill_shop` + `crate::skill_shop_prefs` (`prefs/set_prefs/cycle_sort_mode/sort_mode_label`, `StyleFilter`, `DistanceFilter`, `show_hintless`).
- Memory-tracking flag: `memory_reader::TRACKING` (AtomicBool).

## Design decisions (confirmed with user)
- Tabs: **text labels** (Training / Skills / Bonds / Shop). Widen `OVERLAY_MIN_WIDTH` (currently 300) to ~340-360 so 4 text tabs fit.
- Selected tab: **in-memory only** (resets to Training on reload). Store in a `static` in ui.rs (e.g. `AtomicU8` or `Mutex<Tab>`).
- Tracking OFF / no active career: show a **"Start tracking" hint** (no hook-fallback counts inside tabs; keep `draw_overlay_hooks` only as the pre-tracking state if desired, but the agreed behavior is a hint).
- Long lists (Skills/Bonds/Shop): wrap each in an `egui::ScrollArea::vertical().max_height(...)` with a fixed max height so the panel stays compact.
- Training tab uses `egui::Grid` (core egui, no egui_extras) for table-like alignment.

## Tab content spec
- **Training**: Grid 1 → row(Turn, Month), row(Energy hp/max_hp with motivation color). Grid 2 → header row(Speed/Stamina/Power/Guts/Wit each with `(L{level})`), value row(the 5 stat values). Then: `Total {total_stats}` + `Rank: —` (placeholder for Hachimi-Edge-2we).
- **Skills**: list from `overlay_cache::skills()` (reuse `draw_skills_panel` body), scrollable.
- **Bonds**: list from `overlay_cache::evaluations()` (reuse `draw_bonds_panel` body, `bond_color`), scrollable.
- **Skill Shop**: SP (`skill_points()`), then `draw_skill_shop_controls` (sort + Running/Distance filters + show-full-price), then purchasable list (reuse `draw_skill_shop_panel` list body), scrollable.

## Approach
Mostly a refactor of existing `draw_overlay_*` functions into per-tab renderers + a tab-bar dispatcher. Keep panic-safety (`catch_unwind` already wraps `draw_overlay`). Don't change registration (`register_panel`) or the menu section.

## Verification gate (run after each task)
`cargo build` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt` · `cargo test`. Ignore fmt nightly-import warnings. Deploy is the user's job (game must be closed; never launch/kill the game) — do NOT deploy automatically.

## Done when
Overlay panel shows a 4-tab bar; Training tab renders the two grid tables + total + `Rank: —`; Skills/Bonds/Shop tabs render their (scrollable) lists; "Start tracking" hint appears when tracking is off; full gate green. Rank value itself is Hachimi-Edge-2we.