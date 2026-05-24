# IL2CPP Class Map

Confirmed classes, methods, fields, and properties extracted from the game's `global-metadata.dat` (IL2CPP metadata v31). All names are in the `Gallop` namespace unless noted.

## Metadata Analysis Details

- **Source**: `UmamusumePrettyDerby_Data/il2cpp_data/Metadata/global-metadata.dat`
- **Metadata version**: 31
- **IL2CPP version**: 31
- **File size**: ~29 MB
- **Method**: Direct string table parsing (PE loader failed due to DllMain dependency on Unity runtime)

## Career Mode Controllers

### `SingleModeMainViewController`
Primary view controller for career mode. 95 methods confirmed at runtime (2026-05-23).

**Methods:**
| Name | Args | Confirmed |
|------|------|-----------|
| `OnClickTraining` | 0 | ✅ runtime (opens training view, no command_id) |
| `OnClickTrainingMenu` | 1 | ✅ runtime + hooked |
| `SendCommandAsync` | 6 | ✅ runtime + hooked (arg1=command_id) |
| `CommonSendCommandAsync` | 2 | ✅ runtime + hooked |
| `OnClickRace` | 1 | ✅ runtime |
| `OnClickHospital` | 0 | ✅ runtime |
| `OnClickOuting` | 0 | ✅ runtime |
| `SetupCommandSelectStart` | 2 | ✅ runtime |
| `BackFromTraining` | 0 | ✅ runtime |

**Properties:**
- `get_SelectedTrainingCommandId` / `set_SelectedTrainingCommandId`
- `get_TrainingCommandId`
- `get_SeaTrainingCommandId`
- `get_SingleModeCharaId`
- `get_SingleModeScenario`
- `get_TrainingView`
- `get_TrainingController`
- `get_TrainingMenu`
- `get_TrainingButton`
- `get_TrainingButtonRoot`
- `get_TrainingFooter`
- `get_TrainingNum`
- `get_TrainingRank`
- `get_TrainingStatus`
- `get_TrainingTipsModel`
- `get_TrainingTurnInfo`
- `get_IsInTraining`
- `get_IsPlayingTrainingCutt`
- `get_IsPlayingOrWillPlayTrainingCutt`
- `get_IsSingleMode`
- `get_IsSingleModeChara`
- `get_SingleModeFlashRoot`
- `get_SingleModeFooter`
- `get_SingleModeHeader`

**Fields:**
> **⚠️ Runtime note (2026-05-23):** Field probing found 0/41 expected fields on this class. These names are from metadata analysis and may be auto-properties accessible only via getters, or the names may differ at runtime. Use property getters (`get_*` methods) instead.

- `_commandId`, `_commandType`, `_currentCommandId` — ❌ not found at runtime
- `_trainingCommandId`, `selectedCommandId` — ❌ not found at runtime
- `_singleModeCharaData`, `_singleModeData` — ❌ not found at runtime
- `_trainingView`, `_trainingController` — ❌ not found (but `get_TrainingController` method exists)
- `_trainingLevelDic` — ❌ not found at runtime

## Training UI Classes

### `TrainingView`

> **⚠️ Runtime (2026-05-23):** Class NOT FOUND under `Gallop` in `umamusume.dll`. Present in metadata strings but may be nested or in a different assembly.

| Member | Type | Confirmed |
|--------|------|-----------|
| `OnDecide` | method | ✅ metadata only |
| `get_SelectedTrainingCommandId` | property | ✅ metadata only |
| `get_TrainingCommandId` | property | ✅ metadata only |

### `TrainingSelectDecide`

> **⚠️ Runtime (2026-05-23):** Class NOT FOUND at runtime.

| Member | Type | Confirmed |
|--------|------|-----------|
| `OnDecide` | method | ✅ metadata only |

### `TrainingController`

> **⚠️ Runtime (2026-05-23):** Class NOT FOUND at runtime.

| Member | Type | Confirmed |
|--------|------|-----------|
| `OnDecide` | method | ✅ metadata only |
| `get_TrainingLevel` | property | ✅ metadata only |
| `get_TrainingRank` | property | ✅ metadata only |
| `get_TrainingStatus` | property | ✅ metadata only |
| `get_IsInTraining` | property | ✅ metadata only |
| `get_TrainingHorse` | property | ✅ |
| `get_TrainingHorseList` | property | ✅ |
| `get_TrainingCutStatus` | property | ✅ |
| `get_TrainingHighSpeedType` | property | ✅ |
| `get_TrainingEventType` | property | ✅ |

### `TrainingMain`
| Member | Type | Confirmed |
|--------|------|-----------|
| `OnDecide` | method | ✅ |

### `TrainingMenu`
| Member | Type | Confirmed |
|--------|------|-----------|
| `_trainingButton` | field | ✅ |
| `_trainingCountText` | field | ✅ |
| `_trainingLevelText` | field | ✅ |
| `_trainingNameText` | field | ✅ |
| `_trainingNameShadow` | field | ✅ |
| `_trainingIconImage` | field | ✅ |
| `_trainingIconBase` | field | ✅ |
| `_trainingMaxIcon` | field | ✅ |

### `TrainingButton`
| Member | Type | Confirmed |
|--------|------|-----------|
| `_trainingIconImage` | field | ✅ |
| `_trainingIconBase` | field | ✅ |
| `_trainingLevelTitle` | field | ✅ |
| `_trainingLevelText` | field | ✅ |
| `_trainingButtonFlash` | field | ✅ |

### `TrainingParamChangeA2U`
Text display for training stat gain captions. Already hooked by Hachimi for localization.

### `TrainingParamChangePlate`
Plate/log text for training stat changes. Already hooked by Hachimi for localization.

### `TrainingParamChangeUI`
Overlay for stat change visualization.

### `TrainingParamChangeSupportMemberA2U`
Support member stat change display.

### `TrainingDefine` / `TrainingDefineExtensions`
Constants and utility methods for the training system.

### `TrainingCuttController` / `TrainingCuttData`
Training cutscene playback controller and data.

### `TrainingEnvParam` / `TrainingEnvParamHelper`
Training environment parameters (visuals, effects).

### `TrainingModelController`
3D model management during training scenes.

### `TrainingFootSmokeController`
Foot smoke particle effects during training.

## Race Telemetry Classes

> Source: Trainers-Legend-G cross-reference. See [trainers-legend-g-crossref.md](trainers-legend-g-crossref.md).

### `HorseRaceInfo`
Real-time race statistics for each horse during a race.

| Property | Return Type | Confirmed |
|----------|-------------|----------|
| `get_RaceBaseSpeed` | float | ✅ TLG |
| `get_MinSpeed` | float | ✅ TLG |
| `get_StartDashSpeedThreshold` | float | ✅ TLG |
| `get_IsOverRun` | bool | ✅ TLG |
| `GetHp` | float | ✅ TLG |
| `GetMaxHp` | float | ✅ TLG |
| `GetHpPer` | float | ✅ TLG |
| `get_NearHorseCount` | int | ⚠️ TLG (commented out) |
| `get_CongestionTime` | float | ⚠️ TLG (commented out) |
| `get_RawSpeed` / `get_BaseSpeed` / `get_Speed` | int/int/float | ✅ TLG |
| `get_RawStamina` / `get_BaseStamina` / `get_Stamina` | int/float/float | ✅ TLG |
| `get_RawPow` / `get_BasePow` / `get_Pow` | int/float/float | ✅ TLG |
| `get_RawGuts` / `get_BaseGuts` / `get_Guts` | int/float/float | ✅ TLG |
| `get_RawWiz` / `get_BaseWiz` / `get_Wiz` | int/float/float | ✅ TLG |
| `get_IsStartDash` | bool | ✅ TLG |
| `get_MoveDistance` | float | ✅ TLG |

### `HorseRaceInfoReplay`
Replay/recorded race data.

| Member | Type | Args | Confirmed |
|--------|------|------|----------|
| `.ctor` | method | 2 | ✅ TLG (hooked) |
| `get_RunMotionSpeed` | property | 0 | ✅ TLG (hooked) |
| `get_RunMotionRate` | property | 0 | ✅ TLG |
| `get_RaceMotion` | property | 0 | ✅ TLG |
| `get_IsLastSpurt` | property | 0 | ✅ TLG |
| `get_LastSpurtStartDistance` | property | 0 | ✅ TLG |
| `get_FinishOrder` | property | 0 | ✅ TLG |

### `HorseData`
Per-horse identification data.

| Member | Args | Confirmed |
|--------|------|----------|
| `get_GateNo` | 0 | ✅ TLG |
| `get_charaName` | 0 | ✅ TLG |
| `get_TrainerName` | 0 | ✅ TLG |
| `InitTrainerName` | 0 | ✅ TLG |
| `get_SingleModeTeamRank` | 0 | ✅ TLG |

## Skill System Classes

> Source: Trainers-Legend-G cross-reference + runtime diagnostics (2026-05-24).

### `SkillManager`

Race-time skill manager. Not a singleton. Holds the active skill list for a horse during races.

**Fields (runtime-verified 2026-05-24):**
| Field | Type | Purpose |
|-------|------|---------|
| `_ownerInfo` | `IHorseRaceInfo` | Owner horse reference |
| `_skills` | `List<SkillBase>` | All skills on this horse |
| `_skillArray` | `SkillBase[]` | Array copy of skills |
| `_usedSkillIdList` | `List<Int32>` | Skill IDs that have been used |
| `_prevActivateSkills` | `List<SkillBase>` | Previously activated skills |
| `_skillView` | `ISkillView` | Skill UI view reference |

**Methods (runtime-verified 2026-05-24):**
| Method | Args | Return | Confirmed |
|--------|------|--------|-----------|
| `AddSkill` | 1 | void | ✅ runtime |
| `GetSkill` | 1 | `SkillBase` | ✅ TLG + runtime |
| `GetSkills` | 0 | `SkillBase[]` | ✅ runtime |
| `CreateSkillArray` | 0 | void | ✅ runtime |
| `ClearSkills` | 0 | void | ✅ runtime |
| `AddUsedSkillId` | 1 | void | ✅ TLG (hooked) + runtime |
| `GetUsedSkillIdList` | 0 | `List<Int32>` | ✅ runtime |
| `AddCurrentActiveSkill` | 1 | void | ✅ runtime |
| `RemoveCurrentActiveSkill` | 1 | void | ✅ runtime |
| `GetCurrentActiveSkill` | 0 | `List<ISkillDetail>` | ✅ runtime |
| `AddPrevActivateSkill` | 1 | void | ✅ runtime |
| `RemovePrevActivateSkill` | 1 | void | ✅ runtime |
| `GetPrevActivateSkillList` | 0 | `IReadOnlyList<SkillBase>` | ✅ runtime |
| `Update` | 1 | void | ✅ runtime |
| `LotActivateSkill` | 0 | void | ✅ runtime |
| `CheckSkillTriggerAndActivate` | 0 | void | ✅ runtime |
| `InitSkillEffect` | 1 | void | ✅ runtime |
| `InitSkillSE` | 0 | void | ✅ runtime |
| `PlaySkillEffect` | 3 | void | ✅ runtime |
| `PlaySkillEffect` | 4 | void | ✅ runtime (overload) |
| `StopEffect` | 0 | void | ✅ runtime |
| `PauseEffect` | 0 | void | ✅ runtime |
| `ResumeEffect` | 0 | void | ✅ runtime |
| `SetEffectSpeed` | 1 | void | ✅ runtime |
| `PlaySkillSE` | 2 | void | ✅ runtime |
| `StopSE` | 0 | void | ✅ runtime |

### `SkillBase`

Base class for individual skills attached to a horse. Contains the master data reference and level.

**Fields (runtime-verified 2026-05-24):**
| Field | Type | Purpose |
|-------|------|---------|
| `SKILL_DETAIL_CAPACITY` | `Int32` | Static constant |
| `SKILL_ACTIVATE_LOT_TRUE` | `Int32` | Static constant |
| `_ownerInfo` | `IHorseRaceInfo` | Owner horse |
| `_triggerCreator` | `ISkillTriggerCreator` | Trigger factory |
| `_randomGenerator` | `IRaceRandomGenerator` | RNG for activation |
| `_skillParam` | `RaceParamDefine.SkillParam` | Skill parameters |
| `<IsActivateEnable>k__BackingField` | `Boolean` | Whether skill can activate |
| `<Details>k__BackingField` | `List<ISkillDetail>` | Skill detail instances |
| `<SkillMaster>k__BackingField` | `MasterSkillData.SkillData` | Master data reference |
| `<Level>k__BackingField` | `Int32` | Skill level |
| `_coolDownTime` | `Single` | Cooldown timer |

**Methods (runtime-verified 2026-05-24):**
| Method | Args | Return | Confirmed |
|--------|------|--------|-----------|
| `get_IsActivateEnable` | 0 | `Boolean` | ✅ runtime |
| `get_Details` | 0 | `List<ISkillDetail>` | ✅ TLG + runtime |
| `get_SkillMaster` | 0 | `MasterSkillData.SkillData` | ✅ TLG + runtime |
| `get_SkillMasterId` | 0 | `Int32` | ✅ runtime |
| `get_Level` | 0 | `Int32` | ✅ TLG + runtime |
| `get_CoolDownTime` | 0 | `Single` | ✅ runtime |
| `.ctor` | 2 | void | ✅ runtime |
| `CreateSkillDetail` | 8 | `ISkillDetail` | ✅ runtime |
| `SetupDetail` | 8 | void | ✅ runtime |
| `CreateAbility` | 11 | `ISkillAbility` | ✅ runtime |
| `Stop` | 0 | void | ✅ runtime |
| `IsActivatedAny` | 0 | `Boolean` | ✅ runtime |
| `Update` | 1 | void | ✅ runtime |
| `CheckCoolDown` | 1 | `Boolean` | ✅ runtime |
| `CheckTriggerAndActivate` | 0 | void | ✅ runtime |
| `LotActivate` | 0 | void | ✅ runtime |
| `CheckActivateEnable` | 0 | `Boolean` | ✅ runtime |

### `MasterSkillData`

Master database table for skill definitions (from `master.mdb`). Contains a nested `SkillData` class.

**Fields (runtime-verified 2026-05-24):**
| Field | Type | Purpose |
|-------|------|---------|
| `TABLE_NAME` | `String` | SQLite table name (static) |
| `_db` | `MasterCardDatabase` | Database reference |
| `_preloaded` | `Boolean` | Whether all entries are preloaded |
| `_notFounds` | `HashSet<Int32>` | Cache of missing skill IDs |
| `_lazyPrimaryKeyDictionary` | `Dictionary<Int32, SkillData>` | Cached skill ID → SkillData map |
| `_dictionaryWithGroupId` | `Dictionary<Int32, List<SkillData>>` | Group ID → SkillData list |

**Methods (runtime-verified 2026-05-24):**
| Method | Args | Return | Purpose |
|--------|------|--------|--------|
| `Get` | 1 | `SkillData` | Look up skill by ID |
| `get_dictionary` | 0 | `Dictionary<Int32, SkillData>` | Get full cached dictionary |
| `GetWithGroupIdOrderByIdAsc` | 1 | `SkillData` | Look up by group ID |
| `GetListWithGroupIdOrderByIdAsc` | 1 | `List<SkillData>` | List by group ID |
| `Unload` | 0 | void | Release cached data |
| `_ForcePreloadAllEntries` | 0 | void | Force-load all entries |

> **Note on `MasterSkillData.SkillData`**: This is a **nested class** — use `il2cpp_find_nested_class(MasterSkillData_klass, "SkillData")` to resolve it. It likely has fields like `id`, `group_id`, `rarity`, `skill_category`, `disp_order`, etc. matching the `master.mdb` schema. Needs further introspection.

### Class resolution failures (2026-05-24)

The following class names were **not found** in the `Gallop` namespace:
- `AcquiredSkill` — likely a nested class (e.g. inside `WorkSingleModeCharaData` or `SingleModeChara`) or may use a different name. The field `_acquiredSkillList` on `WorkSingleModeCharaData` is typed as `List<AcquiredSkill>` in metadata.
- `SkillTips` — similar situation, may be nested
- `SkillData` — exists as `MasterSkillData.SkillData` (nested class), not top-level
- `SingleModeAcquiredSkill` — not found
- `WorkSingleModeSkillData` — not found
- `SkillDataManager` — not found

> **Next step**: Use `il2cpp_find_nested_class` on `WorkSingleModeCharaData` and `SingleModeChara` to find `AcquiredSkill` and `SkillTips`. Also introspect `MasterSkillData.SkillData` nested class for skill name/ID fields.

### `StandaloneSimulator.SkillDetail`

> Note: Namespace is `StandaloneSimulator`, not `Gallop`.

| Member | Type | Confirmed |
|--------|------|----------|
| `get_Abilities` | property | ✅ TLG |
| `get_SkillEffectName` | property | ✅ TLG |
| `Activate` | method | ⚠️ TLG (commented out) |
| `get_DefaultCoolDownTime` | property | ⚠️ TLG (commented out) |

## Character Build Classes

> Source: Trainers-Legend-G cross-reference.

### `CharacterBuildInfo`

| Member | Args | Parameters | Confirmed |
|--------|------|------------|----------|
| `.ctor` | 11 | `charaId, dressId, controllerType, headId, zekken, mobId, backDancerColorId, isUseDressDataHeadModelSubId, audienceId, motionDressId, isEnableModelCache` | ✅ TLG |
| `.ctor` | 14 | `cardId, charaId, dressId, controllerType, headId, zekken, mobId, backDancerColorId, overrideClothCategory, isUseDressDataHeadModelSubId, audienceId, motionDressId, isEnableModelCache, charaDressColorSetId` | ✅ TLG |
| `Rebuild` | 0 | — | ✅ TLG (hooked) |

### `EditableCharacterBuildInfo`

| Member | Args | Parameters | Confirmed |
|--------|------|------------|----------|
| `.ctor` | 11 | `cardId, charaId, dressId, controllerType, zekken, mobId, backDancerColorId, headId, isUseDressDataHeadModelSubId, isEnableModelCache, chara_dress_color_set_id` | ✅ TLG |

### `SingleModeSceneController` (confirmed signature)

| Member | Args | Parameters | Confirmed |
|--------|------|------------|----------|
| `CreateModel` | 3 | `cardId: int, dressId: int, addVoiceCue: bool` | ✅ TLG (hooked) |

## UmaControllerType Enum

> Source: Trainers-Legend-G. Used as the `controllerType` parameter in `CharacterBuildInfo` constructors.

| Name | Value |
|------|-------|
| Default | 0x0 |
| Race | 0x1 |
| Training | 0x2 |
| EventTimeline | 0x3 |
| Live | 0x4 |
| LiveTheater | 0x5 |
| HomeStand | 0x6 |
| HomeTalk | 0x7 |
| HomeWalk | 0x8 |
| CutIn | 0x9 |
| TrainingTop | 0xa |
| SingleRace | 0xb |
| Simple | 0xc |
| Mini | 0xd |
| Paddock | 0xe |
| Champions | 0xf |

## Physics/Spring Classes

> Source: Trainers-Legend-G cross-reference.

### `CySpringParamDataElement`
Bone spring simulation parameters.

| Field | Type | Confirmed |
|-------|------|----------|
| `_boneName` | string | ✅ TLG |
| `_stiffnessForce` | float | ✅ TLG |
| `_dragForce` | float | ✅ TLG |
| `_gravity` | Vector3 | ✅ TLG |
| `_childElements` | array | ✅ TLG |
| `_verticalWindRateSlow` | float | ✅ TLG |
| `_collisionRadius` | float | ✅ TLG |
| `_needEnvCollision` | bool | ✅ TLG |
| `_horizontalWindRateSlow` | float | ✅ TLG |
| `_verticalWindRateFast` | float | ✅ TLG |
| `_horizontalWindRateFast` | float | ✅ TLG |
| `_isLimit` | bool | ✅ TLG |
| `_MoveSpringApplyRate` | float | ✅ TLG |

### `CySpringParamDataChildElement`
Same fields as parent minus `_childElements`, `_collisionRadius`, `_needEnvCollision`, `_isLimit`, `_MoveSpringApplyRate`.

## Miscellaneous Classes (TLG-sourced)

### `Gallop.Certification`
| Method | Args | Confirmed |
|--------|------|----------|
| `get_dmmViewerId` | 0 | ✅ TLG |

### `Gallop.GallopUtil` (additional)
| Method | Args | Confirmed |
|--------|------|----------|
| `GetUserName` | 0 | ✅ TLG |

### `Gallop.GameDefine`
| Method | Args | Confirmed |
|--------|------|----------|
| `get_ApplicationServerUrl` | 0 | ✅ TLG (hooked) |

## Manager / Singleton Classes

### `WorkDataManager`
Central singleton hub for all working game data. **✅ LIVE singleton confirmed at runtime (2026-05-24).** 48 fields, 49 methods.

This is THE entry point for reading game state from memory. Every `Work*Data` class is accessible through a property getter on this singleton.

**Access pattern:**
```
WorkDataManager (singleton)
  → get_SingleMode() → WorkSingleModeData
    → get_Character() → WorkSingleModeCharaData
      → get_Speed(), get_Stamina(), get_Power(), get_Guts(), get_Wiz()
      → get_Hp(), get_MaxHp(), get_SkillPoint(), get_Motivation(), get_FanCount()
    → get_IsPlaying() → bool
    → GetCurrentTurn() → int
    → get_HomeInfo() → WorkSingleModeHomeInfo
```

**Key methods (all 0 args, runtime-verified 2026-05-24):**
| Method | Returns |
|--------|---------|
| `get_SingleMode` | `WorkSingleModeData` — **the career data accessor** |
| `get_UserData` | `WorkUserData` |
| `get_CardData` | `WorkCardData` |
| `get_CharaData` | `WorkCharaData` |
| `get_SupportCardData` | `WorkSupportCardData` |
| `get_TrainedCharaData` | `WorkTrainedCharaData` |
| `get_ItemData` | `WorkItemData` |
| `get_TeamStadiumData` | `WorkTeamStadiumData` |
| `get_TrainingChallengeData` | `WorkTrainingChallengeData` |

**All 48 fields** are `<Name>k__BackingField` properties backed by their respective `Work*Data` types. Full list includes: UserData, FriendData, CardData, SupportCardData, CharaData, DressData, MusicData, ItemData, TrainedCharaData, **SingleMode**, PaymentItemData, MainStoryData, CharacterStoryData, ExtraStoryData, PieceData, MissionData, CircleChatData, CircleData, Trophy, Exchange, HomeFavorite, LoginBonusData, AnnounceData, TeamStadiumData, DirectoryData, ScenarioRecordData, SupportDeckData, RaceStateData, DailyLegendRaceData, HonorData, LimitedSalesData, AlreadyReadData, LastCheckTime, ChampionsData, StoryEventData, StoryEventMissionData, RouletteDerbyData, ChallengeMatchData, GalleryData, TalkGalleryData, RoomMatchData, TransferEventData, PracticeRaceData, JukeboxData, ValentineData, TrainingChallengeData, FanRaidData, TeamBuildingData.

### Classes NOT FOUND at runtime (2026-05-24)
These were probed but do not exist as `Gallop::ClassName` in `umamusume.dll`:
- `SingleModeManager` ❌
- `SingleModeWorkDataManager` ❌
- `SingleModeContext` ❌
- `SingleModeDataManager` ❌

## Data Model Classes

### `WorkSingleModeData`
Working copy of career state during gameplay. **✅ Found at runtime (2026-05-23, deep-dived 2026-05-24).** 32 fields, 179 methods.

Not a singleton. Accessed via `WorkDataManager.get_SingleMode()`.

**Key fields (runtime-verified 2026-05-24):**
| Field | Type |
|-------|------|
| `<Character>k__BackingField` | `WorkSingleModeCharaData` |
| `_homeInfo` | `WorkSingleModeHomeInfo` |
| `_isPlaying` | `System.Boolean` |
| `_isExistPlayingData` | `System.Boolean` |
| `_totalTurnNum` | `ObscuredInt` |
| `_state` | `ObscuredInt` |
| `_playingState` | `ObscuredInt` |
| `<SelectedTrainingCommandId>k__BackingField` | `ObscuredInt` |
| `_raceConditions` | `List<RaceCondition>` |
| `_changeParameterInfo` | `WorkSingleModeChangeParameterInfo` |
| `_raceHistoryInfoList` | `List<RaceHistoryInfo>` |
| `_storyInfoListDic` | `Dictionary<EventPlayTiming, List<EventInfo>>` |
| `_scenarioIdList` | `List<ObscuredInt>` |
| `_difficultyInfoList` | `List<DifficultyInfo>` |

**Key methods (runtime-verified 2026-05-24):**
| Method | Args | Returns | Purpose |
| `get_IsPlaying` | 0 | `System.Boolean` | Whether a career is active |
| `get_IsExistPlayingData` | 0 | `System.Boolean` | Whether playing data exists |
| `get_Month` | 0 | `System.Int32` | Current month |
| `get_Character` | 0 | `WorkSingleModeCharaData` | Character data accessor |
| `get_HomeInfo` | 0 | `WorkSingleModeHomeInfo` | Home screen data |
| `get_SelectedTrainingCommandId` | 0 | `ObscuredInt` | Currently selected training |
| `get_State` | 0 | `SingleModeDefine.State` | Career state enum |
| `get_PlayingState` | 0 | `SingleModeDefine.PlayingState` | Playing state enum |
| `GetCurrentTurn` | 0 | `System.Int32` | Current turn number |
| `GetFinalTurn` | 0 | `System.Int32` | Final turn number |
| `GetRemainTurnNum` | 0 | `System.Int32` | Remaining turns |
| `GetScenarioId` | 0 | `SingleModeDefine.ScenarioId` | Active scenario |
| `get_ChangeParameterInfo` | 0 | `WorkSingleModeChangeParameterInfo` | Stat change info |
| `get_RaceConditions` | 0 | `List<RaceCondition>` | Race conditions |
| `get_RaceHistoryInfoList` | 0 | `List<RaceHistoryInfo>` | Race history |
| `get_TotalRaceCount` | 0 | `System.Int32` | Total races run |
| `get_WinCount` | 0 | `System.Int32` | Total wins |
| `get_TeamRace` | 0 | `WorkSingleModeScenarioTeamRace` | Team race data |
| `get_IsStepTurn` | 0 | `System.Boolean` | Step turn flag |
| `get_StoryEventTotalBonus` | 0 | `System.Int32` | Story event bonus |

### `WorkSingleModeHomeInfo`
Working copy of home screen data including available commands. **✅ Found at runtime (2026-05-23), deep-dived (2026-05-24).** 12 fields, 13 methods.

Not a singleton. Accessed via `WorkSingleModeData.get_HomeInfo()`.

**Fields (runtime-verified 2026-05-24):**
| Field | Type |
|-------|------|
| `_turnInfoListDic` | `Dictionary<CommandType, List<TurnInfo>>` |
| `_disableCommandIdList` | `List<ObscuredInt>` |
| `_availableContinueNum` | `ObscuredInt` |
| `_availableFreeContinueNum` | `ObscuredInt` |
| `_freeContinueNum` | `ObscuredInt` |
| `_prevFreeContinueTime` | `ObscuredLong` |
| `_shortenedRaceState` | `ObscuredInt` |
| `SHORTENED_STATE_NONE/DEBUT/DEBUT_PREOP/DEBUT_OP/DEBUT_PREOP_OP` | `System.Int32` (constants) |

**Key methods:**
| Method | Args | Purpose |
| `get_TurnInfoListDic` | 0 | Turn info dictionary |
| `get_DisableCommandIdList` | 0 | Disabled commands this turn |
| `Apply` | 1 | Apply server response data |

### `WorkSingleModeCharaData`
Working copy of character data during career. **✅ Found at runtime (2026-05-23), deep-dived (2026-05-24).** 73 fields, 131 methods.

Not a singleton. Accessed via `WorkSingleModeData.get_Character()`.

> **IMPORTANT: ObscuredInt fields.** All numeric backing fields use `CodeStage.AntiCheat.ObscuredTypes.ObscuredInt` (encrypted in memory). However, the C# property getters **decrypt and return plain `System.Int32`**. Always use property getters (`get_Speed`, etc.) via IL2CPP method invoke — never read the `_speed` field directly.

**Fields (runtime-verified 2026-05-24):**
| Field | Type | Purpose |
|-------|------|---------||
| `_speed` | `ObscuredInt` | Encrypted speed stat |
| `_stamina` | `ObscuredInt` | Encrypted stamina stat |
| `_power` | `ObscuredInt` | Encrypted power stat |
| `_guts` | `ObscuredInt` | Encrypted guts stat |
| `_wiz` | `ObscuredInt` | Encrypted wisdom stat |
| `_hp` | `ObscuredInt` | Encrypted current HP/vital |
| `_maxHp` | `ObscuredInt` | Encrypted max HP/vital |
| `<SkillPoint>k__BackingField` | `ObscuredInt` | Encrypted skill points |
| `_motivation` | `ObscuredInt` | Encrypted motivation |
| `_fanCount` | `ObscuredInt` | Encrypted fan count |
| `_trainingLevelDic` | `Dictionary<TrainingCommandId, Int32>` | Training levels per facility |
| `_acquiredSkillList` | `List<AcquiredSkill>` | Acquired skills |
| `_skillTipsList` | `List<SkillTips>` | Skill tips/hints |
| `<MaxSpeed/Stamina/Power/Guts/Wiz>k__BackingField` | `ObscuredInt` | Stat caps |
| `<DefaultMaxSpeed/Stamina/Power/Guts/Wiz>k__BackingField` | `ObscuredInt` | Default stat caps |
| `_properDistance*` | `ObscuredInt` | Distance aptitudes (Short/Mile/Middle/Long) |
| `_properRunningStyle*` | `ObscuredInt` | Style aptitudes (Nige/Senko/Sashi/Oikomi) |
| `_properGround*` | `ObscuredInt` | Ground aptitudes (Turf/Dirt) |
| `_cardId` | `ObscuredInt` | Card ID |
| `_scenarioId` | `ObscuredInt` | Scenario ID |
| `_routeId` | `ObscuredInt` | Route ID |
| `_charaGrade` | `ObscuredInt` | Character grade |
| `<Race>k__BackingField` | `WorkSingleModeRaceData` | Race data |
| `<TeamRace>k__BackingField` | `WorkSingleModeScenarioTeamRace` | Team race data |

**Stat getters (all 0 args, return `System.Int32`, runtime-verified 2026-05-24):**
| Method | Returns |
|--------|---------|
| `get_Speed` | Speed stat (decrypted) |
| `get_Stamina` | Stamina stat (decrypted) |
| `get_Power` | Power stat (decrypted) |
| `get_Guts` | Guts stat (decrypted) |
| `get_Wiz` | Wisdom stat (decrypted) |
| `get_Hp` | Current HP/vital (decrypted) |
| `get_MaxHp` | Max HP/vital (decrypted) |
| `get_SkillPoint` | Skill points (`ObscuredInt` return) |
| `get_Motivation` | Motivation (`RaceDefine.Motivation` enum) |
| `get_FanCount` | Fan count (decrypted) |
| `GetAllTotalParameterValue` | Sum of all 5 stats |
| `get_MaxSpeed/Stamina/Power/Guts/Wiz` | Stat caps (`ObscuredInt` return) |

**Training/career methods:**
| Method | Args | Purpose |
| `GetTrainingLevel` | 1 | Training level for a command_id |
| `GetParamFromType` | 1 | Get stat value by type |
| `get_ScenarioProgress` | 0 | Scenario progress |
| `get_RunningStyle` | 0 | Running style |
| `ApplySingleModeChara` | 1 | Apply server chara data |

**Other methods (confirmed via TLG):**
| Method | Args | Confirmed |
|--------|------|----------|
| `GetRaceDressId` | 1 (`isApplyDressChange: bool`) | ✅ TLG (hooked) |

### `WorkSingleModeRaceData`
Working copy of race data.

### `WorkSingleModeScenarioFree`
Working copy of Free scenario data.

### `WorkSingleModeScenarioTeamRace`
Working copy of Team Race scenario data.

### `WorkTrainingChallengeData`
Working copy of Training Challenge data.

## Master Data (Database) Classes

These correspond to SQLite tables in `master.mdb`:

| Class/Table | Purpose | Indexed Queries |
|-------------|---------|-----------------|
| `masterSingleModeTraining` | Training facility definitions | by `commandId`, `commandType`, `commandId+commandLevel` |
| `masterSingleModeTrainingEffect` | Training effect definitions | by `commandId+resultState` |
| `masterSingleModeTrainingSe` | Training sound effects | by `sheetId` |
| `masterSingleModeTurn` | Turn definitions per scenario | by `turnSetId` |
| `masterSingleModeProgram` | Race program schedule | by `month` |
| `masterSingleModeRaceGroup` | Race groupings | by `raceGroupId`, `raceProgramId` |
| `masterSingleModeEvaluation` | Friendship thresholds | by `charaId` |
| `masterSingleModeCharaEffect` | Character effects | — |
| `masterSingleModeCharaGrade` | Character grade data | — |
| `masterSingleModeSkillNeedPoint` | Skill point costs | — |
| `masterSingleModeRoute` | Career route definitions | by `scenarioId`, `scenarioId+charaId` |
| `masterSingleModeRouteRace` | Route race definitions | by `raceSetId` |
| `masterSingleModeScenario` | Scenario definitions | — |
| `masterSingleModeDifficultyData` | Difficulty settings | by `difficultyId+difficultyIndex` |
| `masterSingleModeNpc` | NPC definitions | by `npcGroupId` |
| `masterSingleModeOuting` | Outing definitions | by `commandGroupId` |
| `masterSingleModeFanCount` | Fan count thresholds | by `fanSetId` |
| `masterSingleModeHintGain` | Hint gain data | by `hintId` |
| `masterSingleModeMessage` | In-game messages | — |
| `masterSingleModeRewardSet` | Reward definitions | by `rewardSetId` |
| `masterSingleModeStoryData` | Story data | by `storyId`, `cardId`, `cardCharaId`, etc. |
| `masterTrainingCuttCharaData` | Training cutscene chars | by `commandId+subId` |
| `masterTrainingCuttData` | Training cutscene data | by `commandId+subId` |

## Network Request/Response Classes

### Confirmed Request Types (Base Career)
| Class | Purpose |
|-------|---------|
| `SingleModeStartRequest` | Start career |
| `SingleModeCheckEventRequest` | Check turn events |
| `SingleModeExecCommandRequest` | Execute training/action |
| `SingleModeGainSkillsRequest` | Learn skills |
| `SingleModeRaceEntryRequest` | Enter a race |
| `SingleModeRaceStartRequest` | Start a race |
| `SingleModeRaceEndRequest` | End a race |
| `SingleModeRaceOutRequest` | Exit race |
| `SingleModeRaceReserveRequest` | Reserve a race |
| `SingleModeMultiRaceReserveRequest` | Reserve multi race |
| `SingleModeFinishRequest` | End career |
| `SingleModeContinueRequest` | Continue after failure |
| `SingleModeLoadRequest` | Load saved career |
| `SingleModeChangeRunningStyleRequest` | Change running style |
| `SingleModeChangeShortCutRequest` | Change shortcut |
| `SingleModeMinigameEndRequest` | End minigame |
| `SingleModeGetChoiceRewardRequest` | Get choice reward |
| `SingleModeSaveRaceResultRequest` | Save race result |

Each has a corresponding `*Response` and `*Task` class.

### Free Scenario Extensions
`SingleModeFree*` — Includes all base types plus:
- `SingleModeFreeChoiceRewardRequest`
- `SingleModeFreeMultiItemExchangeRequest` / `MultiItemExchange2Request`
- `SingleModeFreeMultiItemUseRequest`
- `SingleModeFreeRaceAnalyzeRequest`

### Team Scenario Extensions
`SingleModeTeam*` — Includes all base types plus:
- `SingleModeTeamOpponentListRequest`
- `SingleModeTeamRaceAnalyzeRequest`
- `SingleModeTeamTeamEditRequest`
- `SingleModeTeamTeamRaceAnalyzeRequest`
- `SingleModeTeamTeamRaceStartRequest` / `EndRequest` / `OutRequest`
- `SingleModeTeamSaveTeamEditFlagRequest`

## Classes Already Hooked by Hachimi Edge

The tables below list hooks relevant to career/training. For the full list of hooked modules, see `src/il2cpp/hook/umamusume/mod.rs` which initializes **34+ additional modules** covering graphics, story, camera, race, UI, and text rendering.

### Career / Training Hooks

| Class | Method | Purpose | Notes |
|-------|--------|---------|-------|
| `TrainingParamChangeA2U` | `GetCaptionText` | Training caption localization | Strips template filters |
| `TrainingParamChangePlate` | `PlayTypeWriteJp` (JP) / `PlayTypeWrite` (non-JP) | Training plate text | JP variant: 2 args (`message`, `skip_add_system_log`); non-JP: 1 arg (`message`) |
| `SingleModeUtils` | `GetMonthTextByTurn` | Month text formatting | Template context exposes `month` and `half` filters |
| `MasterSingleModeTurn.SingleModeTurn` | `get_Month`, `get_Half` | Turn calendar field accessors | These are field accessors on the nested `SingleModeTurn` class, not method hooks on `MasterSingleModeTurn` itself |
| `PartsSingleModeSkillListItem` | `UpdateItem`, `SetupOnClickSkillButton` | Skill list rendering | `UpdateItem` has region-specific overloads: JP takes 4 args, non-JP takes 2 args |
| `PartsSingleModeSkillLearningListItem` | `UpdateCurrent` | Skill learning text | |
| `PartsSingleModeChoiceRewardTextElementViewModel` | `GetParameterValueText` | Choice reward text | |

### General Hooks (also active during career)

| Class | Method | Purpose |
|-------|--------|--------|
| `Localize` | `Get` | General text localization |
| `LibNative.Sqlite3.Connection` | `Query`, `PreparedQuery` | SQLite query interception |
| `LibNative.Sqlite3.Query` | `GetText`, `Dispose` | SQLite text replacement |
| `LibNative.Sqlite3.PreparedQuery` | `BindInt` | SQLite parameter binding |

### Other Hooked Modules (non-exhaustive)

These are initialized in `src/il2cpp/hook/umamusume/mod.rs` and provide hooks for graphics, story, camera, race, and UI:

`ButtonCommon`, `CameraController`, `CameraData`, `CharacterNoteTopView`, `CharacterNoteTopViewController`, `CySpringController`, `DialogCommon`, `DialogCommonBase`, `DialogManager`, `DialogObject`, `DialogRaceOrientation`, `FlashActionPlayer`, `GallopUtil`, `GameSystem`, `GraphicSettings`, `ImageCommon`, `JikkyoDisplay`, `LiveTheaterCharaSelect`, `LiveTheaterViewController`, `LiveUtil`, `LowResolutionCamera`, `LyricsController`, `MasterDataUtil`, `MasterMissionData`, `NowLoading`, `PartsCommonHeaderTitle`, `PartsRaceAnalyzeRaceEventListItem`, `PaymentUtility`, `RaceInfo`, `RaceUtil`, `SaveDataManager`, `SceneManager`, `Screen`, `SingleModeStartResultCharaViewer`, `StoryChoiceController`, `StoryParamChangeEffect`, `StoryRaceTextAsset`, `StoryTimelineBlockData`, `StoryTimelineCharaTrackData`, `StoryTimelineClipData`, `StoryTimelineController`, `StoryTimelineData`, `StoryTimelineTextClipData`, `StoryTimelineTrackData`, `StoryViewController`, `StoryViewTextControllerLandscape`, `StoryViewTextControllerSingleMode`, `TextCommon`, `TextDotData`, `TextFontManager`, `TextFormat`, `TextFrame`, `TextId`, `TextMeshProUguiCommon`, `TimeUtil`, `TweenAnimationTimelineComponent`, `TweenAnimationTimelineData`, `TweenAnimationTimelineSheetData`, `UIManager`, `ViewControllerBase`, `WebViewDefine`, `WebViewManager`

## Appendix: Complete Training-Related String List

The following strings were extracted from the metadata string table matching training/career patterns. This is a subset of ~6,700 matches; see the full dump for complete results.

Key class names:
```
SingleModeMainViewController
TrainingView
TrainingController
TrainingMain
TrainingMenu
TrainingButton
TrainingTop
TrainingSelectDecide
TrainingParamChangeA2U
TrainingParamChangePlate
TrainingParamChangeUI
TrainingParamChangeSupportMemberA2U
TrainingDefine
TrainingCuttController
TrainingCuttData
TrainingEnvParam
TrainingModelController
WorkSingleModeData
WorkSingleModeHomeInfo
WorkSingleModeCharaData
```

Key method/property names:
```
OnClickTraining
OnDecide
OnClickSelect
OnClickStart
OnClickRace
get_SelectedTrainingCommandId
set_SelectedTrainingCommandId
get_TrainingCommandId
get_TrainingLevel
get_TrainingRank
get_IsInTraining
selectedCommandId
selectedTraining
```
