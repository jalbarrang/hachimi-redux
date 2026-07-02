#!/usr/bin/env bash
# run.sh — check + verify the manual-tracking intent against the live codebase.
#
#   .hiker/tents/manual-tracking/run.sh
#
# Exits non-zero if the intent no longer compiles OR the codebase violates it
# (i.e. tracking gained an automatic start/stop path again). Requires `hiker` on
# PATH (https://github.com/jalbarrang/hiker) — CI installs it; locally:
#   irm https://raw.githubusercontent.com/jalbarrang/hiker/stable/install.ps1 | iex   # Windows
#   curl -fsSL https://raw.githubusercontent.com/jalbarrang/hiker/stable/install | sh # *nix
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tent="$here/manual-tracking.tent"

facts="$(mktemp)"
trap 'rm -f "$facts"' EXIT
"$here/extract-facts.sh" >"$facts"

hiker check "$tent"
hiker verify "$tent" --facts "$facts"
