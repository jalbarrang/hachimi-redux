# Training System Internals

## Overview

Training is the primary stat-building mechanic in career mode. Each turn, the player chooses one of five training facilities (Speed, Stamina, Power, Guts, Wisdom) or alternative actions (rest, outing, race, skill learning). Each facility has a level that increases with use, support card partners that rotate each turn, and stat gains that depend on multiple factors.

> **Naming convention:** The game internally uses `wiz` as the field name for the Wisdom stat (e.g., `SingleModeChara.wiz`). This documentation uses "Wisdom" in prose and `wiz` only when referencing exact field/property names.

## Command ID Mapping

Training facilities are identified by `command_id`. Different career scenarios use different ID ranges, but they all map to the same 5 facilities:

### Base / URA Scenario
| command_id | Facility | Train Index |
|-----------|----------|-------------|
| 101 | Speed | 0 |
| 105 | Stamina | 1 |
| 102 | Power | 2 |
| 103 | Guts | 3 |
| 106 | Wisdom | 4 |

### Aoharu Scenario
| command_id | Facility | Train Index |
|-----------|----------|-------------|
| 601 | Speed | 0 |
| 602 | Stamina | 1 |
| 603 | Power | 2 |
| 604 | Guts | 3 |
| 605 | Wisdom | 4 |

### Make a New Track (Arc) Scenario
| command_id | Facility | Train Index |
|-----------|----------|-------------|
| 1101 | Speed | 0 |
| 1102 | Stamina | 1 |
| 1103 | Power | 2 |
| 1104 | Guts | 3 |
| 1105 | Wisdom | 4 |

### UAF / Sport Scenario
The UAF scenario has three sub-types per facility:

| command_id | Facility | Sub-type | Train Index |
|-----------|----------|----------|-------------|
| 2101, 2201, 2301 | Speed | A, B, C | 0 |
| 2102, 2202, 2302 | Stamina | A, B, C | 1 |
| 2103, 2203, 2303 | Power | A, B, C | 2 |
| 2104, 2204, 2304 | Guts | A, B, C | 3 |
| 2105, 2205, 2305 | Wisdom | A, B, C | 4 |

### Onsen Scenario (Partial)
Only 3 facilities confirmed:

| command_id | Facility | Train Index |
|-----------|----------|-------------|
| 901 | Speed | 0 |
| 902 | Power | 2 |
| 906 | Wisdom | 4 |

## Command Types

The `command_type` field distinguishes training from other actions:

| command_type | Action |
|-------------|--------|
| 1 | Training |
| 3 | Outing |
| 4 | Rest |
| 7 | Race |

(Values inferred from UmamusumeResponseAnalyzer code comments)

## Training Level System

Each facility has a level (`TrainingLevelInfo.level`) stored on the character:

```csharp
class TrainingLevelInfo {
    int command_id;  // Which facility
    int level;       // Current level (increases with use)
}
```

The `training_level_info_array` on `SingleModeChara` holds one entry per facility. Level increases when you train at that facility, boosting future stat gains. This is the closest the game has to a built-in "hit counter" — higher level ≈ more visits.

The level data is accessible via:
- Server responses: `chara_info.training_level_info_array`
- IL2CPP fields: `_trainingLevelDic` on controller objects
- Properties: `get_TrainingLevel`, `TrainingLevelInfo`, `TrainingLevelMax`

## Per-Turn Training Data

Each turn, the server provides training details via `SingleModeCommandInfo`:

```csharp
class SingleModeCommandInfo {
    int command_type;                          // 1 = training
    int command_id;                            // Facility ID
    int is_enable;                             // Whether available
    int[] training_partner_array;              // Support card positions at this facility
    int[] tips_event_partner_array;            // Hint event partners
    SingleModeParamsIncDecInfo[] params_inc_dec_info_array;  // Stat gain preview
    int failure_rate;                          // Training failure chance (%)
}
```

### Stat Gain Preview

`SingleModeParamsIncDecInfo` describes each stat delta:

```csharp
class SingleModeParamsIncDecInfo {
    int target_type;  // Which stat (speed=1, stamina=2, power=3, guts=4, wiz/Wisdom=5, etc.)
    int value;        // Gain amount (can be negative)
}
```

## Training Partners

Support cards at each facility are tracked in `training_partner_array`. Each entry is a **deck slot ID** (not a card ID):
- **1–6**: Player's support cards (by deck slot position)
- **7+**: Scenario-specific partners
- **>1000**: NPC partners

Partners affect stat gains (friendship bonus), and when a partner has 80+ friendship and is at their specialty facility, it triggers a **friendship training** (shining) bonus.

## Confirmed UI Classes for Training

From metadata analysis, these classes handle training UI:

| Class | Purpose |
|-------|---------|
| `TrainingView` | Main training facility display |
| `TrainingMenu` | Training facility selection menu |
| `TrainingButton` | Individual training button (one per facility) |
| `TrainingController` | Training logic coordinator |
| `TrainingMain` | Top-level training screen |
| `TrainingTop` | Training screen header/layout |
| `TrainingSelectDecide` | Training confirmation step |
| `TrainingParamChangeA2U` | Training stat gain caption |
| `TrainingParamChangePlate` | Training stat gain plate/log |
| `TrainingParamChangeUI` | Training stat gain overlay |
| `TrainingTipsModel` | Training tips/hints display |
| `TrainingStatusAnimationTimeline` | Stat animation timeline |

## Relevant Fields on Training UI Objects

| Field | Purpose |
|-------|---------|
| `_trainingButton` | Reference to the training button |
| `_trainingCountText` | Text showing training count |
| `_trainingLevelText` | Text showing facility level |
| `_trainingNameText` | Text showing facility name |
| `_trainingNameShadow` | Shadow for facility name |
| `_trainingIconImage` | Facility icon |
| `_trainingMaxIcon` | Max level indicator |
| `_trainingParamChangeUI` | Stat change overlay |
| `_currentTrainingInfo` | Current training state |
| `_previewTrainingInfo` | Preview on hover |
| `_aoharuTrainingCountLabel` | Aoharu-specific count label |

## Approaches for Training Tracking

### Recommended: Direct Memory Read via Singleton Chain (2026-05-24)

The preferred approach reads game state directly from memory on demand, using the confirmed singleton chain:

```
WorkDataManager (singleton)
  → get_SingleMode() → WorkSingleModeData
    → get_IsPlaying() → bool (guard: only read when true)
    → get_Character() → WorkSingleModeCharaData
      → get_Speed/Stamina/Power/Guts/Wiz() → int (decrypted from ObscuredInt)
      → get_Hp/MaxHp() → int
      → get_SkillPoint() → ObscuredInt
      → get_Motivation() → enum
      → get_FanCount() → int
      → GetTrainingLevel(commandId) → int
    → GetCurrentTurn/GetFinalTurn/GetRemainTurnNum() → int
    → get_TotalRaceCount/get_WinCount() → int
```

Advantages:
- No hook-counting drift (always reads current snapshot)
- Works regardless of which hooks fired
- Can read at any time (overlay frame, button click, etc.)
- Returns decrypted values via property getters (bypasses ObscuredInt)

Implementation: Resolve methods via `il2cpp_get_method`, get singleton via `il2cpp_get_singleton_like_instance`, call getters via method pointer cast. User clicks "Start Tracking" to begin.

### Alternative: Hook-Based Event Counting

Hook points for event-driven tracking (still useful for detecting state changes):

1. **`SingleModeMainViewController.SendCommandAsync(6)`** — arg1 is `command_id` (e.g., 106 = Wisdom). **Confirmed working** — this is where command_id reliably appears. ✅
2. **`SingleModeMainViewController.OnClickTrainingMenu(1)`** — Fires when player taps a specific training facility. Arg is an IL2CPP object (not the command_id directly). ✅
3. **`SingleModeMainViewController.OnClickTraining(0)`** — Opens training view, no args. Useful for detecting training view entry but carries no command_id. ✅
4. **Read `WorkSingleModeData.get_SelectedTrainingCommandId`** — Poll the selected command ID from the working data. Field `<SelectedTrainingCommandId>k__BackingField` confirmed at runtime. ✅
5. **Read `WorkSingleModeCharaData.GetTrainingLevel(1)`** — Get training level by command_id. 131 methods confirmed at runtime including all stat getters. ✅

> **Deprecated hook points:** `TrainingSelectDecide.OnDecide`, `TrainingView.OnDecide`, `TrainingController.OnDecide` — these classes were NOT FOUND at runtime under `Gallop` in `umamusume.dll` despite being present in metadata strings. Do not rely on them.
