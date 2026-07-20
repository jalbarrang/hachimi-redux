# Maintenance — hosted data pipeline

This repo is the active home of the HachimiRedux mod (see [README.md](README.md)). This file documents the hosted-data publishing pipeline: the in-core Training Tracker downloads its game-data snapshots from this repo's `main/data/…` raw GitHub URLs (defaults in `apps/hachimi/src/core/hosted_data/mod.rs`).

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

## Workflow inventory

| Workflow | Role |
| --- | --- |
| `data_refresh.yml` | Daily/`workflow_dispatch` refresh. Runs `cargo run -p` for data tools only (`fetch-master-db`, `skill-grades`, `course-data`, `tracker-data-manifest`, `gametora-sync`). Does **not** build mod crates. Commits `data/` (+ mirrored `plugins/training-tracker/assets`). |
| `ci.yml` | Fmt/deny/machete/hiker/clippy/test/check for the Rust mod workspace. |
| `create_release.yml` | Builds and releases the core mod + installer binaries. |
| `sdk_release.yml` | Tags/releases plugin-SDK crates. |
| `audit.yml` | Daily `cargo audit` over the whole workspace (mod + tools). |

## URL-STABILITY WARNING

Deployed builds of the mod hardcode these download bases (`apps/hachimi/src/core/hosted_data/mod.rs`):

```
https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data
https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data/gametora
https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data/icons
```

Renaming this repository, renaming or replacing the `main` branch, or moving/renaming the `data/` directory **breaks hosted-data updates for every deployed build** until a new release changes the defaults (or users override the URLs in their config).
