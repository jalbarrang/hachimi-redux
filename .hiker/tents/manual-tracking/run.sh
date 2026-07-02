#!/usr/bin/env bash
# run.sh — check + verify the manual-tracking intent against the live codebase.
#
#   .hiker/tents/manual-tracking/run.sh
#
# Exits non-zero if the intent no longer compiles OR the codebase violates it
# (i.e. tracking gained an automatic start/stop path again). Requires `hiker` on
# PATH — install it per .hiker/README.md ("Install hiker").
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tent="$here/manual-tracking.tent"

facts="$(mktemp)"
trap 'rm -f "$facts"' EXIT
# Invoke via `bash` (not a direct exec) so it works regardless of the file's
# executable bit — Windows checkouts don't preserve it.
bash "$here/extract-facts.sh" >"$facts"

hiker check "$tent"
hiker verify "$tent" --facts "$facts"
