# Architectural intent (hiker)

This repo states a few **non-negotiable architectural invariants** as compiled,
verified intent using [hiker](https://github.com/jalbarrang/hiker), so code can't
silently drift from them. Authoring/how-to knowledge lives in the vendored skill
at [`.agents/skills/hiker/`](../.agents/skills/hiker/SKILL.md); this file is just
the repo setup.

## Tents (one folder per invariant)

| Tent | Invariant |
|---|---|
| [`tents/manual-tracking/`](./tents/manual-tracking/CONTEXT.md) | Training Tracker memory tracking is fully manual (no automatic start/stop). |

Each tent holds `<slug>.tent` (the spec), `CONTEXT.md` (what it means + code
anchors), and — for structural checks — an `extract-facts.sh` + `run.sh`.

## Install hiker

Use the **same version CI pins** — `HIKER_VERSION` in
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) is the single source of
truth (bump it there when adopting a newer CLI):

```sh
# *nix — match HIKER_VERSION (e.g. 0.1.2)
curl -fsSL https://raw.githubusercontent.com/jalbarrang/hiker/stable/install | sh -s -- --version 0.1.2
```

```powershell
# Windows
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/jalbarrang/hiker/stable/install.ps1))) -Version 0.1.2
```

## Run the gate

```sh
hiker check                              # compile every tent (via .hikerconf globs)
.hiker/tents/manual-tracking/run.sh      # check + verify one tent against the codebase
```

CI runs the same in the `2 · Intent (hiker)` job. Generated `gen` output lands in
the gitignored `.hiker-cache/`.
