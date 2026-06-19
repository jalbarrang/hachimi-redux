//! Translations tab — options grid + native action strip.

use dioxus_egui::dioxus::prelude::*;
use honse_ui::Toggle;
use rust_i18n::t;

use super::super::context::{ControlCenterCtx, HostAction};
use super::layout::{LabelCell, SettingsGrid};

#[component]
pub fn TranslationsTab() -> Element {
    let ctx = use_context::<ControlCenterCtx>();
    let _ = ctx.revision.read();
    let cfg = ctx.config.borrow().clone();

    let label_meta_index_url = t!("config_editor.meta_index_url").to_string();
    let label_disable_translations = t!("config_editor.disable_translations").to_string();
    let label_lazy_translation_updates = t!("config_editor.lazy_translation_updates").to_string();
    let label_translator_mode = t!("config_editor.translator_mode").to_string();
    let label_disable_skill_name_translation = t!("config_editor.disable_skill_name_translation").to_string();
    let label_auto_translate_stories = t!("config_editor.auto_translate_stories").to_string();
    let label_auto_translate_ui = t!("config_editor.auto_translate_ui").to_string();
    let meta_index_url = cfg.meta_index_url.clone();

    let on_auto_stories = {
        let ctx = ctx.clone();
        move |v: bool| {
            ctx.config.borrow_mut().auto_translate_stories = v;
            if v {
                ctx.actions.borrow_mut().push(HostAction::AutoTranslateWarning);
            }
            ctx.bump_revision();
        }
    };
    let on_auto_ui = {
        let ctx = ctx.clone();
        move |v: bool| {
            ctx.config.borrow_mut().auto_translate_localize = v;
            if v {
                ctx.actions.borrow_mut().push(HostAction::AutoTranslateWarning);
            }
            ctx.bump_revision();
        }
    };

    rsx! {
        div {
            "dir": "col",
            "gap": "8",
            "align": "stretch",
            div { "native": "egui" }
            SettingsGrid {
                LabelCell { text: label_meta_index_url }
                input {
                    value: "{meta_index_url}",
                    oninput: ctx.bind(|c, e: Event<FormData>| c.meta_index_url = e.value()),
                }

                LabelCell { text: label_disable_translations }
                Toggle {
                    label: String::new(),
                    checked: cfg.disable_translations,
                    onchange: ctx.bind(|c, v| c.disable_translations = v),
                }

                LabelCell { text: label_lazy_translation_updates }
                Toggle {
                    label: String::new(),
                    checked: cfg.lazy_translation_updates,
                    onchange: ctx.bind(|c, v| c.lazy_translation_updates = v),
                }

                LabelCell { text: label_translator_mode }
                Toggle {
                    label: String::new(),
                    checked: cfg.translator_mode,
                    onchange: ctx.bind(|c, v| c.translator_mode = v),
                }

                LabelCell { text: label_disable_skill_name_translation }
                Toggle {
                    label: String::new(),
                    checked: cfg.disable_skill_name_translation,
                    onchange: ctx.bind(|c, v| c.disable_skill_name_translation = v),
                }

                LabelCell { text: label_auto_translate_stories }
                Toggle {
                    label: String::new(),
                    checked: cfg.auto_translate_stories,
                    onchange: on_auto_stories,
                }

                LabelCell { text: label_auto_translate_ui }
                Toggle {
                    label: String::new(),
                    checked: cfg.auto_translate_localize,
                    onchange: on_auto_ui,
                }
            }
        }
    }
}
