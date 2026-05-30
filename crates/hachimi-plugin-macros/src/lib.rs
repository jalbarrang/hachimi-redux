//! `#[hachimi_plugin]` — generates the `hachimi_init` and `hachimi_plugin_manifest`
//! C entry points from a single init function.
//!
//! ```ignore
//! use hachimi_plugin_sdk::{hachimi_plugin, Sdk};
//!
//! #[hachimi_plugin]                       // name/version from Cargo, min_api from the SDK
//! fn init(sdk: &Sdk) -> Result<(), String> {
//!     sdk.show_notification("loaded!");
//!     Ok(())
//! }
//! ```
//!
//! Optional args: `#[hachimi_plugin(name = "...", version = "...", min_api = 9, caps = ...)]`.
//! The init function must take `&Sdk` and return `Result<(), E>` where `E: Display`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, ItemFn, Meta, Token};

#[proc_macro_attribute]
pub fn hachimi_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);
    let func = parse_macro_input!(item as ItemFn);
    let fn_name = &func.sig.ident;

    let mut name = quote! { env!("CARGO_PKG_NAME") };
    let mut version = quote! { env!("CARGO_PKG_VERSION") };
    let mut min_api = quote! { ::hachimi_plugin_sdk::API_VERSION };
    let mut caps = quote! { 0u64 };

    for meta in args {
        let Meta::NameValue(nv) = meta else { continue };
        let Some(ident) = nv.path.get_ident() else { continue };
        let value = nv.value;
        match ident.to_string().as_str() {
            "name" => name = quote! { #value },
            "version" => version = quote! { #value },
            "min_api" => min_api = quote! { #value },
            "caps" => caps = quote! { #value },
            _ => {}
        }
    }

    quote! {
        #func

        #[no_mangle]
        pub extern "C" fn hachimi_plugin_manifest() -> *const ::hachimi_plugin_sdk::PluginManifest {
            static MANIFEST: ::hachimi_plugin_sdk::PluginManifest = ::hachimi_plugin_sdk::PluginManifest {
                abi_version: ::hachimi_plugin_sdk::API_VERSION,
                min_host_api: #min_api,
                requested_caps: #caps,
                name: ::core::concat!(#name, "\0").as_ptr() as *const ::core::ffi::c_char,
                version: ::core::concat!(#version, "\0").as_ptr() as *const ::core::ffi::c_char,
            };
            ::core::ptr::addr_of!(MANIFEST)
        }

        /// # Safety
        /// Called by the host with a valid vtable pointer during plugin load.
        #[no_mangle]
        pub extern "C" fn hachimi_init(vtable_ptr: *const ::core::ffi::c_void, version: i32) -> i32 {
            use ::hachimi_plugin_sdk::{init_result_to_i32, InitResult, Sdk};
            // SAFETY: host passes a valid vtable pointer for the process lifetime.
            let __hachimi_init_result = unsafe {
                Sdk::init_min(vtable_ptr as *const ::hachimi_plugin_sdk::Vtable, version, #min_api)
            };
            match __hachimi_init_result {
                Ok(()) => match #fn_name(Sdk::get()) {
                    Ok(()) => init_result_to_i32(InitResult::Ok),
                    Err(err) => {
                        Sdk::get().log_error("plugin", &::std::format!("init failed: {}", err));
                        init_result_to_i32(InitResult::Error)
                    }
                },
                Err(_) => init_result_to_i32(InitResult::Error),
            }
        }
    }
    .into()
}
