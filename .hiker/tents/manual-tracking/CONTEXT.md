# Intent: Training Tracker memory tracking is fully manual

**Spec:** [`manual-tracking.tent`](./manual-tracking.tent) · **Enforced by:** `hiker verify` (structural) via [`run.sh`](./run.sh)

## What it means

The in-core Training Tracker reads career state straight from game memory. That
reader is toggled **only** by explicit user action:

- the **Start / Stop Memory Tracking** button (`ui/menu.rs`), and
- the **toggle-tracking hotkey** (`ui/mod.rs`).

There is **no automatic start/stop**. A manual Stop must stick — it must not be
silently re-armed on the next frame or the next career event. This was a real
regression: the reader auto-restarted ~1.7 s after a manual stop.

## Why it's a `hiker` check and not just a test

This is a *structural* invariant about **who may call** `start_tracking`, not a
behavior of one function over random inputs. `hiker verify` evaluates the spec's
`forbidden relation`s over facts extracted from the codebase (a grep), so a
re-introduced auto-start becomes a **failing gate**, not silent drift.

## The two forbidden relations

| Relation | Extracted fact (a violation) |
|---|---|
| `auto_starts_tracking(CallSite)` | a `start_tracking(...)` call **outside** `ui/menu.rs` / `ui/mod.rs` (the manual controls); the `fn start_tracking` definition is excluded |
| `auto_track_careers_resurrected(Symbol)` | any mention of the removed `auto_track_careers` config field / prefs mechanism |

Conformant tree → zero tuples → `verify` exits 0.

## Code anchors (allow-list — the ONLY sanctioned callers)

- `apps/hachimi/src/core/modules/training_tracker/ui/menu.rs` — Start/Stop button.
- `apps/hachimi/src/core/modules/training_tracker/ui/mod.rs` — `toggle_tracking_hotkey`.
- Definition: `apps/hachimi/src/core/modules/training_tracker/memory_reader/chain.rs` (`start_tracking`).

## If this gate fails

`verify` prints one line per violation (`file:line`). Either:

1. You **intended** a new manual entry point → add its file to `ALLOWED_RE` in
   `extract-facts.sh` (and document it here), or
2. You **re-introduced auto-tracking** → remove it. Automatic start/stop is the
   thing this intent forbids. Do not "fix" the gate by loosening the allow-list
   to cover a lifecycle/refresh caller.

## Run it

```sh
.hiker/tents/manual-tracking/run.sh        # check + verify (needs hiker on PATH)
```

Installing hiker + running the whole gate: see [`.hiker/README.md`](../../README.md).
