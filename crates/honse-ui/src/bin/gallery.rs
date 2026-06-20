//! honse-ui gallery — every component, driven by real state.
//! Run: cargo run -p honse-ui --bin gallery

#![allow(clippy::disallowed_methods)] // dioxus rsx! macro uses unwrap internally

use dioxus_egui::dioxus::prelude::*;
use honse_ui::{
    theme, Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card, CardDescription, CardTitle, Field, Grade,
    Image, Mood, Separator, Stat,
};

fn main() -> dioxus_egui::eframe::Result<()> {
    dioxus_egui::run("honse-ui gallery", app)
}

fn app() -> Element {
    let mut count = use_signal(|| 0);
    let mut agree = use_signal(|| false);
    let mut volume = use_signal(|| 40.0_f64);
    let mut name = use_signal(String::new);

    let fg_dim = theme::FG_DIM;

    rsx! {
        div { "dir": "col", "gap": "14", "align": "stretch", "width": "420",

            Card {
                CardTitle { "Image" }
                CardDescription { "fixed-size img slot (bytes:// URI)" }
                Image { src: "bytes://gallery-placeholder.png".to_string(), width: 24.0, height: 24.0 }
            }

            Card {
                CardTitle { "Buttons" }
                CardDescription { "variant x size, each a themed face with hover/press" }
                div { "dir": "row", "gap": "8", "align": "center",
                    Button { variant: ButtonVariant::Primary, onclick: move |_| count += 1, "Primary" }
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| count += 1, "Secondary" }
                    Button { variant: ButtonVariant::Outline, onclick: move |_| count += 1, "Outline" }
                }
                div { "dir": "row", "gap": "8", "align": "center",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| count += 1, "Ghost" }
                    Button { variant: ButtonVariant::Destructive, onclick: move |_| count.set(0), "Reset" }
                }
                Separator {}
                div { "dir": "row", "gap": "8", "align": "center",
                    Button { size: ButtonSize::Sm, onclick: move |_| count += 1, "Sm" }
                    Button { size: ButtonSize::Md, onclick: move |_| count += 1, "Md" }
                    Button { size: ButtonSize::Lg, onclick: move |_| count += 1, "Lg" }
                }
                div { "color": fg_dim, "font-size": "13", "clicked {count} times" }
            }

            Card {
                CardTitle { "Badges" }
                div { "dir": "row", "gap": "6", "align": "center",
                    Badge { variant: BadgeVariant::Neutral, "Neutral" }
                    Badge { variant: BadgeVariant::Accent, "Leader" }
                    Badge { variant: BadgeVariant::Good, "Ready" }
                    Badge { variant: BadgeVariant::Destructive, "Kakari" }
                    Badge { variant: BadgeVariant::Warn, "Blocked" }
                }
                Separator {}
                div { "dir": "row", "gap": "6", "align": "center",
                    Badge { variant: BadgeVariant::Grade(Grade::S), "S" }
                    Badge { variant: BadgeVariant::Grade(Grade::A), "A" }
                    Badge { variant: BadgeVariant::Grade(Grade::B), "B" }
                    Badge { variant: BadgeVariant::Grade(Grade::C), "C" }
                    Badge { variant: BadgeVariant::Grade(Grade::D), "D" }
                    Badge { variant: BadgeVariant::Grade(Grade::E), "E" }
                    Badge { variant: BadgeVariant::Grade(Grade::G), "G" }
                }
                Separator {}
                div { "dir": "row", "gap": "6", "align": "center",
                    Badge { variant: BadgeVariant::Mood(Mood::Great), "Great" }
                    Badge { variant: BadgeVariant::Mood(Mood::Good), "Good" }
                    Badge { variant: BadgeVariant::Mood(Mood::Normal), "Normal" }
                    Badge { variant: BadgeVariant::Mood(Mood::Bad), "Bad" }
                    Badge { variant: BadgeVariant::Mood(Mood::Awful), "Awful" }
                }
                Separator {}
                div { "dir": "row", "gap": "6", "align": "center",
                    Badge { variant: BadgeVariant::Stat(Stat::Speed), "Spd" }
                    Badge { variant: BadgeVariant::Stat(Stat::Stamina), "Sta" }
                    Badge { variant: BadgeVariant::Stat(Stat::Power), "Pow" }
                    Badge { variant: BadgeVariant::Stat(Stat::Guts), "Guts" }
                    Badge { variant: BadgeVariant::Stat(Stat::Wit), "Wit" }
                }
            }

            Card {
                CardTitle { "Form" }
                Field { label: "Agree to terms".to_string(),
                    input {
                        r#type: "checkbox",
                        checked: agree(),
                        onchange: move |e| agree.set(e.checked()),
                    }
                }
                Field { label: "Volume".to_string(),
                    div { "dir": "row", "gap": "8", "align": "center",
                        input {
                            r#type: "range",
                            value: "{volume}",
                            min: "0",
                            max: "100",
                            "width": "220",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<f64>() {
                                    volume.set(v);
                                }
                            },
                        }
                        div { "color": fg_dim, "{volume:.0}" }
                    }
                }
                Field { label: "Name".to_string(),
                    input {
                        value: "{name}",
                        "width": "260",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                Separator {}
                div { "dir": "row", "gap": "6", "align": "center",
                    if name().is_empty() {
                        div { "color": fg_dim, "type your name above" }
                    } else {
                        "hello {name}"
                        if agree() {
                            Badge { variant: BadgeVariant::Good, "ready" }
                        }
                    }
                }
            }
        }
    }
}
