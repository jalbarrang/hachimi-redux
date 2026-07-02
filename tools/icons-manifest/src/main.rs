//! Publishes the Career-panel icon sprites for **hosted download**.
//!
//! The training-tracker Career panel loads ~16 MB of game UI sprites (trainee
//! portraits, support-card icons, rank sprites, stat icons) on demand from the
//! host data dir (`<game-data>/hachimi/icons/…`). Unlike the JSON resources, these
//! are binary PNGs in nested dirs, so they ride a dedicated hosted-data set that
//! carries binary + nested paths (see `apps/hachimi/src/core/hosted_data`).
//!
//! This tool copies a source icon tree into the repo's `data/icons/` (preserving
//! subdirs) and (re)writes `data/icons/manifest.json` (`relpath -> blake3`), the
//! same manifest contract the host's `hosted_data` sync consumes. Clients then
//! download these committed files from the repo's raw GitHub URL.
//!
//! **Run manually** by the maintainer whenever the icon set changes:
//!
//! ```text
//! # source dir order: $HONSE_ICONS_DIR, else ../honse-tracker/apps/web/public/icons
//! cargo run -p icons-manifest
//! ```
//!
//! Then commit the updated `data/icons/**` + `data/icons/manifest.json`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;

const MANIFEST_FILE: &str = "manifest.json";

/// Our published manifest: `{ generated_at, source, files: { relpath: blake3 } }`.
/// Mirrors `tools/tracker-data-manifest`'s contract so one client reads both.
#[derive(Serialize)]
struct HostedManifest {
    generated_at: String,
    source: String,
    files: BTreeMap<String, String>,
}

/// Repo root: this crate lives at `tools/icons-manifest`, so two up.
fn repo_root() -> PathBuf {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    here.canonicalize().unwrap_or(here)
}

/// Resolve the source icon tree: `$HONSE_ICONS_DIR`, else the sibling
/// `honse-tracker` web assets beside this repo.
fn source_dir(root: &Path) -> PathBuf {
    if let Ok(dir) = std::env::var("HONSE_ICONS_DIR") {
        return PathBuf::from(dir);
    }
    root.join("../honse-tracker/apps/web/public/icons")
}

/// Manifest keys use forward slashes on every platform.
fn rel_key(rel: &Path) -> String {
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn main() -> Result<(), String> {
    let root = repo_root();
    let src = source_dir(&root);
    if !src.is_dir() {
        return Err(format!(
            "icon source dir not found: {} (set $HONSE_ICONS_DIR or clone honse-tracker beside this repo)",
            src.display()
        ));
    }
    let out_dir = root.join("data/icons");
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("mkdir {}: {e}", out_dir.display()))?;

    let mut files = BTreeMap::new();
    let mut total_bytes: u64 = 0;
    for entry in WalkDir::new(&src).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        // Only publish PNG sprites; ignore any stray files in the source tree.
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("png"))
            != Some(true)
        {
            continue;
        }
        let rel = path.strip_prefix(&src).map_err(|e| format!("strip_prefix: {e}"))?;
        let key = rel_key(rel);
        let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let dst = out_dir.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
        std::fs::write(&dst, &bytes).map_err(|e| format!("writing {}: {e}", dst.display()))?;
        total_bytes += bytes.len() as u64;
        files.insert(key, blake3::hash(&bytes).to_hex().to_string());
    }

    if files.is_empty() {
        return Err(format!("no PNG files found under {}", src.display()));
    }

    let manifest = HostedManifest {
        generated_at: chrono::Utc::now().to_rfc3339(),
        source: "honse-tracker web assets (Career panel icon sprites)".to_owned(),
        files,
    };
    let json = serde_json::to_string_pretty(&manifest).map_err(|e| format!("serializing manifest: {e}"))?;
    let manifest_path = out_dir.join(MANIFEST_FILE);
    std::fs::write(&manifest_path, format!("{json}\n")).map_err(|e| format!("writing manifest: {e}"))?;
    println!(
        "published {} icon(s) ({:.1} MB) -> {}",
        manifest.files.len(),
        total_bytes as f64 / (1024.0 * 1024.0),
        out_dir.display()
    );
    println!("wrote {}", manifest_path.display());
    Ok(())
}
