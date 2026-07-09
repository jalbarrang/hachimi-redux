# HachimiRedux — data publishing shell

**The mod has moved.** Plugin development now lives in **[honse-tracker](https://github.com/jalbarrang/honse-tracker)**, built as plugins for upstream **[Hachimi-Edge](https://github.com/kairusds/Hachimi-Edge)**.

This repository exists **only** to publish hosted game-data snapshots consumed by those plugins (GameTora catalogs, tracker resources, career icons). Do not install a mod from this repo; use Edge + the honse-tracker release DLLs instead.

Maintainer notes for the data pipeline: see [MAINTENANCE.md](MAINTENANCE.md). Full refresh sequence: [docs/updating-game-data.md](docs/updating-game-data.md).

## Hosted data

Clients (honse-tracker plugins) download blake3-manifest snapshots from raw GitHub URLs under `main/data/…`. Layout:

| Path | Contents |
| --- | --- |
| `data/manifest.json` | Tracker resources manifest (`skill_grades.json`, `course_params.json`) |
| `data/skill_grades.json`, `data/course_params.json` | Master.mdb-derived tracker assets |
| `data/gametora/` | GameTora catalogs + `manifest.json` |
| `data/icons/` | Career-panel icon sprites + manifest |

Default base URLs hardcoded in honse-tracker:

```
https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data/gametora
https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data
https://raw.githubusercontent.com/jalbarrang/hachimi-redux/main/data/icons
```

**Do not rename this repo, the `main` branch, or the `data/` path** without coordinating a honse-tracker release that updates those defaults — every deployed plugin breaks otherwise.

## Refreshing data

When the Honse game ships an update, regenerate snapshots from the repo root (see [docs/updating-game-data.md](docs/updating-game-data.md) for details):

```bash
cargo run -p fetch-master-db
cargo run -p skill-grades
cargo run -p course-data
cargo run -p tracker-data-manifest
cargo run -p gametora-sync
```

Or rely on the daily [Data Refresh](.github/workflows/data_refresh.yml) workflow (`workflow_dispatch` / cron). Commit the generated files under `data/` (and the mirrored assets under `plugins/training-tracker/assets/` while that path still exists).

## Please don't link to this repo for mod installs

This project (and the Edge/honse-tracker stack) is against the game's TOS. Sharing in private chats is fine; please avoid public links that name the Honse game by its real title. Prefer "the Honse game" or "UM:PD" in public text.

## License

[GNU GPLv3](LICENSE)
