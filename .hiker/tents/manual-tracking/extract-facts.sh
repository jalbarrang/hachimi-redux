#!/usr/bin/env bash
# extract-facts.sh — emit hiker facts.json for manual-tracking.tent.
#
# Two `forbidden relation`s, so ANY emitted tuple is a violation:
#   auto_starts_tracking(CallSite)            — a start_tracking(...) call outside
#                                               the two manual UI controls.
#   auto_track_careers_resurrected(Symbol)    — any mention of the removed
#                                               auto_track_careers mechanism.
#
# A conformant tree emits zero tuples for both → `hiker verify` exits 0.
# hiker never greps source itself; extraction lives here (the consuming repo).
set -euo pipefail

# Repo root = four levels up from this script (.hiker/tents/manual-tracking/).
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
src="$root/apps/hachimi/src"

# Allowed manual callers of start_tracking (path suffixes, forward-slash form).
ALLOWED_RE='ui/(menu|mod)\.rs'
# The definition site is not a caller.
DEF_RE='fn start_tracking'

search() { # pattern -> "relpath:line" per hit (rg if present, else grep -rn)
  local pattern="$1"
  if command -v rg >/dev/null 2>&1; then
    rg -n --no-heading --with-filename -e "$pattern" "$src" 2>/dev/null || true
  else
    grep -rnE "$pattern" "$src" 2>/dev/null || true
  fi
}

# Normalize a raw "path:line:..." hit to a "relpath:line" id (forward slashes,
# relative to repo root, dropping the matched text).
to_id() {
  sed -E 's#\\#/#g' \
    | sed -E "s#^.*/apps/hachimi/src/#apps/hachimi/src/#" \
    | sed -E 's#^([^:]+:[0-9]+):.*#\1#'
}

# --- auto_starts_tracking: start_tracking(...) calls, minus def + allowed UI ---
mapfile -t start_hits < <(
  search 'start_tracking[[:space:]]*\(' \
    | grep -Ev "$DEF_RE" \
    | to_id \
    | grep -Ev "$ALLOWED_RE" \
    | sort -u
)

# --- auto_track_careers_resurrected: any mention of the removed mechanism ---
mapfile -t auto_hits < <(
  search 'auto_track_careers' \
    | to_id \
    | sort -u
)

# JSON assembly ------------------------------------------------------------
json_instances() { # id... -> "{ "id": "x" }, ..."
  local sep="" out=""
  for id in "$@"; do
    [ -z "$id" ] && continue
    out+="${sep}{ \"id\": \"${id}\" }"; sep=", "
  done
  printf '%s' "$out"
}
json_tuples() { # id... -> "[ "x" ], ..."
  local sep="" out=""
  for id in "$@"; do
    [ -z "$id" ] && continue
    out+="${sep}[ \"${id}\" ]"; sep=", "
  done
  printf '%s' "$out"
}

cs_inst="$(json_instances "${start_hits[@]:-}")"
sy_inst="$(json_instances "${auto_hits[@]:-}")"
cs_tup="$(json_tuples "${start_hits[@]:-}")"
sy_tup="$(json_tuples "${auto_hits[@]:-}")"

cat <<JSON
{
  "instances": {
    "CallSite": [${cs_inst}],
    "Symbol": [${sy_inst}]
  },
  "tuples": {
    "auto_starts_tracking": [${cs_tup}],
    "auto_track_careers_resurrected": [${sy_tup}]
  }
}
JSON
