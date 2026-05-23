# Build & Deploy

## Build

- **Core**: `cargo build --release` → `target/release/hachimi.dll`
- **Plugin**: `cargo build --release` from `plugins/training-tracker/` → `plugins/training-tracker/target/release/hachimi_training_tracker.dll`

## Deploy

- **Deploy core**: Copy `target/release/hachimi.dll` as `C:/Program Files (x86)/Steam/steamapps/common/UmamusumePrettyDerby/cri_mana_vpx.dll`
- **Deploy plugin**: Copy plugin DLL to the game directory root
- **Config**: `hachimi/config.json` in the game directory. `menu_open_key: 68` (D key). Plugins listed in `load_libraries`.
