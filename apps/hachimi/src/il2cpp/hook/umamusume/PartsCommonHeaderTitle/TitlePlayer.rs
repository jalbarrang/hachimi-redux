use crate::{
    core::Hachimi,
    il2cpp::{
        ext::{Il2CppStringExt, StringExt},
        hook::UnityEngine_TextRenderingModule::TextGenerator::IgnoreTGFiltersContext,
        symbols::get_method_addr,
        types::{Il2CppClass, Il2CppObject, Il2CppString},
    },
};

// Post-update: `SetText(string)` was replaced by `Play(string text, Action callback)`.
type PlayFn = extern "C" fn(this: *mut Il2CppObject, text: *mut Il2CppString, callback: *mut Il2CppObject);
extern "C" fn Play(this: *mut Il2CppObject, text: *mut Il2CppString, callback: *mut Il2CppObject) {
    if !text.is_null() {
        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
        let utf_str = unsafe { (*text).as_utf16str() };
        // doesn't run through TextGenerator, ignore its filters
        // 36 = dollar sign ($)
        if utf_str.as_slice().contains(&36) {
            let clean_text = Hachimi::instance()
                .template_parser
                .eval_with_context(&utf_str.to_string(), &mut IgnoreTGFiltersContext());
            return get_orig_fn!(Play, PlayFn)(this, clean_text.to_il2cpp_string(), callback);
        }
    }

    get_orig_fn!(Play, PlayFn)(this, text, callback);
}

pub fn init(PartsCommonHeaderTitle: *mut Il2CppClass) {
    find_nested_class_or_return!(PartsCommonHeaderTitle, TitlePlayer);

    let Play_addr = get_method_addr(TitlePlayer, c"Play", 2);

    new_hook!(Play_addr, Play);
}
