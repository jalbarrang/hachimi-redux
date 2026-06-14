use std::sync::OnceLock;

use crate::{
    core::{utils::truncate_text_il2cpp, Hachimi},
    il2cpp::{
        api::il2cpp_field_static_set_value,
        hook::UnityEngine_UI::Text,
        symbols::{get_field_from_name, get_field_object_value, get_method_addr, get_static_field_value},
        types::*,
    },
};

static mut _COMICTITLE_FIELD: *mut FieldInfo = 0 as _;
fn get__comicTitle(this: *mut Il2CppObject) -> *mut Il2CppObject {
    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    get_field_object_value(this, unsafe { _COMICTITLE_FIELD })
}

const COMIC_TITLE_LINE_WIDTH: usize = 23;

// Static `System.Single` fade duration constants on NowLoading. Scaling these
// down speeds up the loading/scene-transition fades regardless of whether they
// are driven by DOTween or by coroutines.
static mut FADE_TIME_FIELDS: [*mut FieldInfo; 3] = [0 as _; 3];
// Original (unscaled) durations, captured once before we ever overwrite them.
static FADE_TIME_ORIGINALS: OnceLock<[f32; 3]> = OnceLock::new();

fn set_static_f32(field: *mut FieldInfo, mut value: f32) {
    if field.is_null() {
        return;
    }
    il2cpp_field_static_set_value(field, &mut value as *mut f32 as *mut _);
}

/// Re-applies the configured loading fade speed to the static duration fields.
/// Originals are captured lazily on first call (after the class cctor has run,
/// guaranteed since this is only reached from instance methods).
fn apply_loading_fade_scale() {
    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    let fields = unsafe { FADE_TIME_FIELDS };

    let originals = FADE_TIME_ORIGINALS.get_or_init(|| {
        [
            get_static_field_value::<f32>(fields[0]),
            get_static_field_value::<f32>(fields[1]),
            get_static_field_value::<f32>(fields[2]),
        ]
    });

    let scale = Hachimi::instance().config.load().loading_fade_scale;
    let scale = if scale > 0.0 { scale } else { 1.0 };

    for (field, &orig) in fields.iter().zip(originals.iter()) {
        set_static_f32(*field, orig / scale);
    }
}

type StartFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Start(this: *mut Il2CppObject) {
    get_orig_fn!(Start, StartFn)(this);
    apply_loading_fade_scale();
}

type SetupLoadingTipsFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn SetupLoadingTips(this: *mut Il2CppObject) {
    get_orig_fn!(SetupLoadingTips, SetupLoadingTipsFn)(this);

    // Re-apply each time loading is shown so config changes take effect live.
    apply_loading_fade_scale();

    if Hachimi::instance()
        .localized_data
        .load()
        .config
        .now_loading_comic_title_ellipsis
    {
        let comic_title = get__comicTitle(this);
        if comic_title.is_null() {
            return;
        }

        let text = Text::get_text(comic_title);
        if text.is_null() {
            return;
        }

        if let Some(new_text) = truncate_text_il2cpp(text, COMIC_TITLE_LINE_WIDTH, true) {
            Text::set_horizontalOverflow(comic_title, 1);
            Text::set_text(comic_title, new_text);
        }
    }
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, NowLoading);

    let SetupLoadingTips_addr = get_method_addr(NowLoading, c"SetupLoadingTips", 0);
    new_hook!(SetupLoadingTips_addr, SetupLoadingTips);

    let Start_addr = get_method_addr(NowLoading, c"Start", 0);
    new_hook!(Start_addr, Start);

    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    unsafe {
        _COMICTITLE_FIELD = get_field_from_name(NowLoading, c"_comicTitle");
        FADE_TIME_FIELDS = [
            get_field_from_name(NowLoading, c"FADE_TIME"),
            get_field_from_name(NowLoading, c"BLACK_FADE_TIME"),
            get_field_from_name(NowLoading, c"WHITE_OUT_HORSE_SHOE_FADE_TIME"),
        ];
    }
}
