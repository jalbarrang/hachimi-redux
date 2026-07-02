//! Hooks the career-mode command flow.
//!
//! Two jobs:
//! 1. Emit [`event::TRAINING_COMMAND`] to subscribed plugins on training submit.
//! 2. Bracket the training-tracker's memory reads around a command sequence so the
//!    reader never walks `HomeInfo`/`TurnInfo` while the game tears down and rebuilds
//!    the Home scene (server round-trip → animation → `Push/PopSceneResourceHash`
//!    asset unload/reload). That teardown is NOT a `SceneManager.ChangeView`, so the
//!    view-change cooldown does not cover it; reading mid-teardown is a use-after-free
//!    that crashes the game (surfaces in the game's own `HomeBgController.CreateBgModel`).
//!
//! Per the IL2CPP class dump (`Gallop.SingleModeMainViewController`):
//! - All command submits funnel through
//!   `CommonSendCommandAsync(CommandType, TrainingCommandId) -> IEnumerator` — the arm
//!   point (rest / infirmary / outing / training).
//! - `SendCommandAsync(CommandType, TrainingCommandId, i32, i32, Action, Action) ->
//!   IEnumerator` is the training-specific submit; we also read the command id here
//!   for the plugin event.
//! - `SetupCommandSelectStart(bool, bool)` / `SetupCommandSelectStartStepTurn(bool)`
//!   rebuild the command-select screen — the disarm point (safe to read again).
//!
//! Both `*SendCommandAsync` are **coroutine kickoffs**: they return an `IEnumerator`
//! the caller feeds to `StartCoroutine`. The hook MUST forward that return value —
//! declaring a `void` hook leaves garbage in the return register and the game crashes
//! when it starts the coroutine.

use crate::il2cpp::{symbols::get_method_addr, types::*};

/// Suspend reads while a command sequence plays out.
#[inline]
fn suspend_reads() {
    crate::core::modules::training_tracker::suspend_reads_for_command();
}

/// Resume reads once the command-select screen is (re)built.
#[inline]
fn resume_reads() {
    crate::core::modules::training_tracker::resume_reads_on_command_select();
}

type SendCommandAsyncFn = extern "C" fn(
    this: *mut Il2CppObject,
    command_type: usize,
    command_id: usize,
    command_group_id: usize,
    select_id: usize,
    on_success: usize,
    on_error: usize,
) -> *mut Il2CppObject;

extern "C" fn SendCommandAsync(
    this: *mut Il2CppObject,
    command_type: usize,
    command_id: usize,
    command_group_id: usize,
    select_id: usize,
    on_success: usize,
    on_error: usize,
) -> *mut Il2CppObject {
    // Arg1 is CommandType, arg2 is the TrainingCommandId (per the class dump).
    crate::core::plugin::events::dispatch_training_command(command_id as i32);
    suspend_reads();
    get_orig_fn!(SendCommandAsync, SendCommandAsyncFn)(
        this,
        command_type,
        command_id,
        command_group_id,
        select_id,
        on_success,
        on_error,
    )
}

type CommonSendCommandAsyncFn =
    extern "C" fn(this: *mut Il2CppObject, command_type: usize, command_id: usize) -> *mut Il2CppObject;

extern "C" fn CommonSendCommandAsync(
    this: *mut Il2CppObject,
    command_type: usize,
    command_id: usize,
) -> *mut Il2CppObject {
    // Every command submit (rest / infirmary / outing / training) funnels here, so
    // this is the reliable read-suspend arm point regardless of command kind.
    suspend_reads();
    get_orig_fn!(CommonSendCommandAsync, CommonSendCommandAsyncFn)(this, command_type, command_id)
}

type SetupCommandSelectStartFn = extern "C" fn(this: *mut Il2CppObject, play_voice: bool, to_top: bool);

extern "C" fn SetupCommandSelectStart(this: *mut Il2CppObject, play_voice: bool, to_top: bool) {
    // Command-select is being rebuilt: the Single Mode objects are fresh; safe to read.
    resume_reads();
    get_orig_fn!(SetupCommandSelectStart, SetupCommandSelectStartFn)(this, play_voice, to_top)
}

type SetupCommandSelectStartStepTurnFn = extern "C" fn(this: *mut Il2CppObject, play_voice: bool);

extern "C" fn SetupCommandSelectStartStepTurn(this: *mut Il2CppObject, play_voice: bool) {
    // Turn advanced back to command-select after a command: safe to read again.
    resume_reads();
    get_orig_fn!(SetupCommandSelectStartStepTurn, SetupCommandSelectStartStepTurnFn)(this, play_voice)
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, SingleModeMainViewController);

    let SendCommandAsync_addr = get_method_addr(SingleModeMainViewController, c"SendCommandAsync", 6);
    new_hook!(SendCommandAsync_addr, SendCommandAsync);

    let CommonSendCommandAsync_addr = get_method_addr(SingleModeMainViewController, c"CommonSendCommandAsync", 2);
    new_hook!(CommonSendCommandAsync_addr, CommonSendCommandAsync);

    let SetupCommandSelectStart_addr = get_method_addr(SingleModeMainViewController, c"SetupCommandSelectStart", 2);
    new_hook!(SetupCommandSelectStart_addr, SetupCommandSelectStart);

    let SetupCommandSelectStartStepTurn_addr =
        get_method_addr(SingleModeMainViewController, c"SetupCommandSelectStartStepTurn", 1);
    new_hook!(SetupCommandSelectStartStepTurn_addr, SetupCommandSelectStartStepTurn);
}
