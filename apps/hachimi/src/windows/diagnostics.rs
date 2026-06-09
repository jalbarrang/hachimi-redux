//! In-game diagnostics report writer.
//!
//! Writes a single `hachimi_diagnostics.txt` next to the game executable that a
//! user can hand to support instead of manually hunting for logs. It bundles the
//! HachimiRedux version, target/region, the menu key, detected conflicting mods,
//! the game-folder DLL listing, the raw `config.json`, and a pointer to the
//! game's own `Player.log`.

use std::fmt::Write as _;
use std::path::PathBuf;

use crate::core::{conflicts, Hachimi};

use super::utils;

const PLAYER_LOG_HINT: &str = r"%USERPROFILE%\AppData\LocalLow\Cygames\Umamusume\Player.log";

/// Build the diagnostics report text.
fn build_report() -> String {
    let hachimi = Hachimi::instance();
    let game_dir = utils::get_game_dir();

    let mut s = String::new();
    let _ = writeln!(s, "HachimiRedux diagnostics report");
    let _ = writeln!(s, "===============================");
    let _ = writeln!(s, "Version: {}", env!("HACHIMI_DISPLAY_VERSION"));
    let _ = writeln!(s, "Game region: {}", hachimi.game.region);
    let _ = writeln!(s, "Game directory: {}", game_dir.display());
    let _ = writeln!(s, "Data directory: {}", hachimi.game.data_dir.display());

    let key_label = utils::vk_to_display_label(hachimi.config.load().windows.menu_open_key);
    let _ = writeln!(s, "Menu key: {}", key_label);
    let _ = writeln!(s);

    // Conflicting mods / injectors.
    let conflicts = conflicts::scan_dir(&game_dir);
    if conflicts.is_empty() {
        let _ = writeln!(s, "Conflicting mods/injectors: none detected");
    } else {
        let _ = writeln!(s, "Conflicting mods/injectors detected ({}):", conflicts.len());
        for c in &conflicts {
            let _ = writeln!(s, "  - {} ({:?})", c.file_name, c.kind);
        }
        let _ = writeln!(
            s,
            "  -> Stacking injectors commonly crashes the game. Keep only HachimiRedux and remove these."
        );
    }
    let _ = writeln!(s);

    // DLL listing in the game folder.
    let _ = writeln!(s, "DLLs in game folder:");
    if let Ok(entries) = std::fs::read_dir(&game_dir) {
        let mut names: Vec<String> = entries
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.to_ascii_lowercase().ends_with(".dll"))
            .collect();
        names.sort_unstable();
        for name in names {
            let _ = writeln!(s, "  - {}", name);
        }
    } else {
        let _ = writeln!(s, "  (could not read game folder)");
    }
    let _ = writeln!(s);

    // config.json (raw).
    let config_path = hachimi.get_data_path("config.json");
    let _ = writeln!(s, "config.json ({}):", config_path.display());
    match std::fs::read_to_string(&config_path) {
        Ok(text) => {
            let _ = writeln!(s, "{}", text);
        }
        Err(e) => {
            let _ = writeln!(s, "  (could not read: {})", e);
        }
    }
    let _ = writeln!(s);

    let _ = writeln!(s, "Game log (Player.log) is usually at:");
    let _ = writeln!(s, "  {}", PLAYER_LOG_HINT);

    s
}

/// Write the diagnostics report to `<game_dir>/hachimi_diagnostics.txt`.
/// Returns the path on success.
pub fn write_report() -> std::io::Result<PathBuf> {
    let path = utils::get_game_dir().join("hachimi_diagnostics.txt");
    std::fs::write(&path, build_report())?;
    Ok(path)
}
