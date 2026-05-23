# Single Mode (Career) Architecture

## Overview

Single Mode is the core career/training gameplay loop. The player raises a character over a fixed number of turns, choosing training facilities, races, rest, or outings each turn. The game processes each turn as a client-server round-trip using MessagePack-encoded requests/responses.

## Lifecycle

```
┌─────────────────┐
│  SingleModeStart │  ← Player selects character, support cards, inheritance
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│  SingleModeCheckEvent    │  ← Server returns turn state: available commands,
│  (each turn)             │     events, support card positions, stat preview
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Player chooses action   │  ← Training / Race / Rest / Outing / Skill Learn
│  (UI interaction)        │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  SingleModeExecCommand   │  ← Client sends command_type + command_id
│  (request → response)    │     Server returns stat changes, events, etc.
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  SingleModeCheckEvent    │  ← Next turn begins
│  (repeat until final)    │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  SingleModeFinish        │  ← Career ends, results screen
└─────────────────────────┘
```

## Key Controller Classes

### `SingleModeMainViewController`

The primary view controller for career mode. Confirmed methods (runtime-verified 2026-05-23):

| Method | Args | Purpose | Runtime status |
|--------|------|---------|----------------|
| `OnClickTraining` | 0 | Opens training view (no command_id) | ✅ verified |
| `OnClickTrainingMenu` | 1 | Player taps a specific training facility button | ✅ verified, hooked |
| `SendCommandAsync` | 6 | Submits command (arg1=command_id) | ✅ verified, hooked |
| `CommonSendCommandAsync` | 2 | Simpler command sender | ✅ verified, hooked |
| `OnClickRace` | 1 | Player selects a race | ✅ verified |
| `OnClickHospital` | 0 | Player selects rest | ✅ verified |
| `OnClickOuting` | 0 | Player selects outing | ✅ verified |

> **Note (2026-05-23):** Earlier docs listed `OnClickTraining` as taking 1 arg (command_id). Runtime shows it takes 0 args — it just opens the training view. The actual command_id flows through `SendCommandAsync(6)` where arg1 is the command_id (e.g., 106 = Wisdom). Field probes found 0/41 expected fields on this class; state is likely accessed via property getters, not direct fields.

### `TrainingSelectDecide`

Handles the training selection confirmation step:

| Method | Args | Purpose |
|--------|------|---------|
| `OnDecide` | 1 | Confirm training selection |

> **⚠️ Runtime status (2026-05-23):** Class NOT FOUND at runtime under `Gallop` in `umamusume.dll`. Present in metadata strings but may be a nested class or different assembly.

### `TrainingView`

Renders the training facility UI:

| Method | Purpose |
|--------|---------|
| `OnDecide` | Training confirmed from the view layer |
| `get_SelectedTrainingCommandId` | Returns the currently selected command_id |
| `get_TrainingCommandId` | Returns the active training command_id |

> **⚠️ Runtime status (2026-05-23):** Class NOT FOUND at runtime. Same as above.

### `TrainingController`

Manages training logic and state:

| Method | Purpose |
|--------|---------|
| `OnDecide` | Training decision processing |

> **⚠️ Runtime status (2026-05-23):** Class NOT FOUND at runtime. Same as above.

### `TrainingMain`

Top-level training orchestrator:

| Method | Purpose |
|--------|---------|
| `OnDecide` | Training decision processing |

> **⚠️ Runtime status (2026-05-23):** Class NOT FOUND at runtime. Same as above.

### Other Confirmed Classes

| Class | Purpose | Runtime (2026-05-23) |
|-------|---------|---------------------|
| `TrainingMenu` | Training facility menu UI | ⚠️ NOT FOUND |
| `TrainingButton` | Individual training button widget | ⚠️ NOT FOUND |
| `TrainingTop` | Training screen top-level layout | ⚠️ NOT FOUND |
| `WorkSingleModeData` | Working copy of career state | ✅ FOUND, has `<SelectedTrainingCommandId>k__BackingField` |
| `WorkSingleModeCharaData` | Working copy of character data | ✅ FOUND, 131 methods (stats, training level, etc.) |
| `WorkSingleModeHomeInfo` | Working copy of home screen data | ✅ FOUND, 13 methods |

> **Note (2026-05-23):** Many training UI classes (`TrainingView`, `TrainingController`, `TrainingMenu`, `TrainingButton`, etc.) appear in metadata strings but are NOT FOUND as `Gallop::ClassName` in `umamusume.dll` at runtime. They may be nested classes, in a different namespace, or in a different assembly. The `Work*` data classes are the reliable runtime targets.

## Confirmed Fields & Properties

These fields are present on career mode objects:

| Field/Property | Type | Purpose |
|----------------|------|---------|
| `selectedCommandId` | `int` | Currently selected command ID |
| `selectedTraining` | object | Currently selected training info |
| `_commandId` | `int` | Internal command ID backing field |
| `_commandType` | `int` | Internal command type backing field |
| `_currentCommandId` | `int` | Currently active command ID |
| `_trainingCommandId` | `int` | Training-specific command ID |
| `_disableCommandIdList` | `List<int>` | Commands that are disabled this turn |
| `_trainingLevelDic` | `Dictionary` | Training level per facility |
| `_trainingPartnerInfoArray` | `Array` | Support cards at each facility |
| `_currentTrainingInfo` | object | Info about the current training |
| `_previewTrainingInfo` | object | Preview info for hovering |

## Career Scenarios

Each career scenario (URA, Grand Masters, UAF, Cook, etc.) extends the base flow with scenario-specific data sets and command IDs. The scenario type is tracked on `SingleModeChara.scenario_id`.

| Scenario | Data Set Class | Training Command IDs |
|----------|---------------|---------------------|
| URA (base) | (base) | 101, 105, 102, 103, 106 |
| Aoharu | (base) | 601, 602, 603, 604, 605 |
| Make a New Track | `SingleModeArcDataSet` | 1101, 1102, 1103, 1104, 1105 |
| Grand Masters (Venus) | `SingleModeVenusDataSet` | (uses base IDs) |
| UAF (Sport) | `SingleModeSportDataSet` | 2101–2105, 2201–2205, 2301–2305 (3 sub-types × 5 facilities) |
| Cook | `SingleModeCookDataSet` | varies by scenario |
| Mecha | `SingleModeMechaDataSet` | varies by scenario |
| Legend | `SingleModeLegendDataSet` | varies by scenario |
| Pioneer | `SingleModePioneerDataSet` | varies by scenario |
| Onsen | `SingleModeOnsenDataSet` | 901, 902, 906 (only 3 of 5 facilities confirmed) |
| Breeders | `SingleModeBreedersDataSet` | varies by scenario |

> **Note:** Command IDs are sparse, not contiguous ranges. For example, URA uses 101 (Speed), 105 (Stamina), 102 (Power), 103 (Guts), 106 (Wisdom). See [training-system.md](training-system.md) for the complete mapping.

## Data Flow

```
Server Response (MessagePack)
    │
    ▼
SingleModeCheckEventResponse.CommonResponse
    ├── chara_info: SingleModeChara
    │       ├── speed, stamina, power, wiz (Wisdom), guts, vital
    │       ├── training_level_info_array: TrainingLevelInfo[]
    │       ├── skill_array, skill_tips_array
    │       └── support_card_array
    ├── home_info: SingleModeHomeInfo
    │       ├── command_info_array: SingleModeCommandInfo[]
    │       └── disable_command_id_array: int[]
    ├── command_result: SingleModeCommandResult
    └── [scenario]_data_set: scenario-specific data
```
