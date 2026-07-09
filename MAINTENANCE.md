# Maintenance — data publishing shell

This repo is retired as a mod. It survives to publish hosted data for [honse-tracker](https://github.com/jalbarrang/honse-tracker) plugins on [Hachimi-Edge](https://github.com/kairusds/Hachimi-Edge). Mod code still exists in-tree; do not delete it without a separate, user-approved cleanup (see Deferred below).

## Data refresh overview

Full sequence and tool notes: [docs/updating-game-data.md](docs/updating-game-data.md). In short: download `master.mdb` (`fetch-master-db`) → regenerate `skill_grades` / `course_data` → publish tracker manifest into `data/` (`tracker-data-manifest`) → sync GameTora catalogs into `data/gametora/` (`gametora-sync`). Clients never embed these files; they download from raw GitHub.

## `data/` layout and manifests

| Path | Role |
| --- | --- |
| `data/manifest.json` | Blake3 hashes for tracker JSON files at the data root |
| `data/skill_grades.json`, `data/course_params.json` | Tracker resources |
| `data/gametora/manifest.json` | Blake3 hashes for GameTora catalog files |
| `data/gametora/*.json` | Skills, character cards, support cards, training events, etc. |
| `data/icons/` | Icon sprites + icons manifest (binary snapshots) |

Manifest format: JSON with `generated_at`, optional `source`, and a `files` map of relative path → blake3 hex digest (see committed `data/manifest.json`).

## Workflow classification

Read-only inventory of `.github/workflows/` (this plan does **not** modify workflows):

| Workflow | Classification | Notes |
| --- | --- | --- |
| `data_refresh.yml` | **KEEP-data** | Daily/`workflow_dispatch` refresh. Runs `cargo run -p` for data tools only (`fetch-master-db`, `skill-grades`, `course-data`, `tracker-data-manifest`, `gametora-sync`). Does **not** build mod crates (`hachimi`, race-hud, etc.). Commits `data/` (+ mirrored `plugins/training-tracker/assets`). |
| `ci.yml` | **OBSOLETE-mod** | Fmt/deny/machete/hiker/clippy/test/check for the Rust mod workspace. |
| `create_release.yml` | **OBSOLETE-mod** | Builds and releases the fork core/installer DLLs. |
| `sdk_release.yml` | **OBSOLETE-mod** | Tags/releases fork plugin-SDK crates. |
| `audit.yml` | **OBSOLETE-mod** | Daily `cargo audit` over the whole workspace (mod + tools). |

### Data-pipeline independence

`data_refresh.yml` does not compile the mod crates. Follow-up risk (not fixed here): the data tools still live in the same Cargo workspace / `Cargo.lock` as the mod, and the refresh job still commits into `plugins/training-tracker/assets/` — a future workspace split must keep those tools and paths green or relocate them.

## URL-STABILITY WARNING

honse-tracker's deployed downloader hardcodes:

```
https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data...
```

(specifically `…/main/data`, `…/main/data/gametora`, `…/main/data/icons`). Renaming this repository, renaming or replacing the `main` branch, or moving/renaming the `data/` directory **breaks every deployed plugin** until a new honse-tracker release changes the defaults (or users override via `hosted_data` in `honseTrackerConfig.json`).

## Deferred suggestion

Future user-approved cleanup: remove or archive the Rust mod workspace (apps, plugins, installer, obsolete workflows) once data tools are extracted or confirmed independent, so this repo is literally only `tools/` + `data/` + docs + `data_refresh.yml`. **Do not do that cleanup in this plan** — deleting shared workspace members risks breaking data publishing.
