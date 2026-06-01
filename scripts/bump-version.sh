#!/usr/bin/env bash
# =============================================================================
# Bump the release version from conventional-commit history.
#
# Computes the next semver from commits since the last `v*` tag (via git-cliff,
# standard semver — see cliff.toml) and writes it into apps/hachimi/Cargo.toml,
# refreshing Cargo.lock at the same time (via `cargo set-version`).
#
# This only edits files; it does NOT commit, tag, or push. After running:
#   1. review the diff, commit (e.g. `chore(release): vX.Y.Z`), push to main
#   2. trigger the "Create Release" workflow (workflow_dispatch) on GitHub
#
# Prerequisites (one-time):
#   cargo install git-cliff
#   cargo install cargo-edit
#
# Usage:
#   ./scripts/bump-version.sh
# =============================================================================

set -euo pipefail

CRATE="hachimi"
MANIFEST="apps/hachimi/Cargo.toml"

# --- Resolve repo root so the script works from anywhere -----------------------
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# --- Preflight: required tools -------------------------------------------------
missing=0
if ! command -v git-cliff >/dev/null 2>&1; then
    echo "❌ git-cliff not found. Install it with: cargo install git-cliff"
    missing=1
fi
if ! cargo set-version --help >/dev/null 2>&1; then
    echo "❌ 'cargo set-version' not found. Install it with: cargo install cargo-edit"
    missing=1
fi
if [[ "$missing" -ne 0 ]]; then
    exit 1
fi

# --- Current version -----------------------------------------------------------
CURRENT=$(awk '/^\[package\]/{flag=1} flag && /^version/{gsub(/[" ]/,"",$3); print $3; exit}' "$MANIFEST")
echo "Current version: ${CURRENT:-unknown}"

# --- Compute the next version from conventional commits ------------------------
# `--bumped-version` prints the next version (with a leading 'v').
# It can fail/return empty when every unreleased commit is skipped by the
# commit_parsers (git-cliff #816) — treat that as "no bump warranted".
if ! NEW_RAW=$(git cliff --bumped-version 2>/dev/null); then
    echo "ℹ️  git-cliff found no version bump warranted (no qualifying commits). Nothing to do."
    exit 0
fi

NEW_VERSION="${NEW_RAW#v}"

if [[ -z "$NEW_VERSION" ]]; then
    echo "ℹ️  No version bump warranted (no qualifying commits since last tag). Nothing to do."
    exit 0
fi

if [[ "$NEW_VERSION" == "$CURRENT" ]]; then
    echo "ℹ️  Computed version ($NEW_VERSION) matches current. Nothing to do."
    exit 0
fi

# --- Apply (edits Cargo.toml AND updates Cargo.lock) ---------------------------
echo "Bumping ${CRATE}: ${CURRENT} -> ${NEW_VERSION}"
cargo set-version -p "$CRATE" "$NEW_VERSION"

echo ""
echo "✅ Bumped to ${NEW_VERSION} (apps/hachimi/Cargo.toml + Cargo.lock)."
echo "   Next: review the diff, commit as 'chore(release): v${NEW_VERSION}',"
echo "   push to main, then run the 'Create Release' workflow on GitHub."
