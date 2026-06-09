//! Formatting and color helpers for overlay rendering.

use hachimi_plugin_sdk::egui;

use crate::memory_reader;

/// Proximity of a stat to its cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CapLevel {
    Normal,
    Near,
    AtCap,
}

/// Classify a stat value against its cap. Unknown cap (`<= 0`) is always `Normal`.
/// `Near` triggers at ≥ 90% of cap; `AtCap` at ≥ cap.
pub(super) fn cap_level(value: i32, cap: i32) -> CapLevel {
    if cap <= 0 {
        return CapLevel::Normal;
    }
    if value >= cap {
        CapLevel::AtCap
    } else if value * 100 >= cap * 90 {
        CapLevel::Near
    } else {
        CapLevel::Normal
    }
}

/// Color for a single stat value, keyed off the real in-game (data-mined)
/// rank thresholds. Letter ranks G→SS+ span values 1..=1200 (Table 1 colors);
/// above 1200 stats keep ranking on the U ladder UG..US9 (1201..=2000, Table 2
/// family colors). Stats never reach the `L*` ranks — those only exist on the
/// overall evaluation badge. Letter `+`/`-` subranks share their base color.
pub(super) fn stat_rank_color(value: i32) -> egui::Color32 {
    let (r, g, b) = if value >= 1901 {
        (255, 0, 0) // US - red
    } else if value >= 1801 {
        (255, 215, 0) // UA - gold
    } else if value >= 1701 {
        (47, 79, 79) // UB - dark slate gray
    } else if value >= 1601 {
        (169, 169, 169) // UC - dark gray
    } else if value >= 1501 {
        (50, 205, 50) // UD - lime green
    } else if value >= 1401 {
        (30, 144, 255) // UE - dodger blue
    } else if value >= 1301 {
        (255, 20, 147) // UF - deep pink
    } else if value >= 1201 {
        (255, 105, 180) // UG - hot pink
    } else if value >= 1100 {
        (255, 215, 0) // SS / SS+ - bright gold
    } else if value >= 1000 {
        (255, 140, 0) // S / S+ - dark orange
    } else if value >= 800 {
        (0, 255, 0) // A / A+ - bright green
    } else if value >= 600 {
        (0, 206, 209) // B / B+ - dark turquoise
    } else if value >= 400 {
        (30, 144, 255) // C / C+ - dodger blue
    } else if value >= 300 {
        (147, 112, 219) // D / D+ - medium purple
    } else if value >= 200 {
        (186, 85, 211) // E / E+ - medium orchid
    } else if value >= 100 {
        (160, 82, 45) // F / F+ - sienna
    } else {
        (128, 128, 128) // G / G+ - gray
    };
    egui::Color32::from_rgb(r, g, b)
}

/// Table 1 color for a base letter rank (`G`,`F`,`E`,`D`,`C`,`B`,`A`,`S`,`SS`).
/// Used for the stat ladder's letter tier and as the "base letter" color in the
/// two-tone evaluation badge. Unknown letters fall back to gray.
pub fn rank_letter_color(letter: &str) -> egui::Color32 {
    let (r, g, b) = match letter {
        "SS" => (255, 215, 0),  // bright gold
        "S" => (255, 140, 0),   // dark orange
        "A" => (0, 255, 0),     // bright green
        "B" => (0, 206, 209),   // dark turquoise
        "C" => (30, 144, 255),  // dodger blue
        "D" => (147, 112, 219), // medium purple
        "E" => (186, 85, 211),  // medium orchid
        "F" => (160, 82, 45),   // sienna
        _ => (128, 128, 128),   // G / unknown - gray
    };
    egui::Color32::from_rgb(r, g, b)
}

/// Table 2 color for a `U*`/`L*` rank family (first two chars of the label,
/// e.g. `UG`, `LF`). Returns `None` for non-prefixed ranks. Used as the prefix
/// (`U`/`L`) color in the two-tone evaluation badge.
pub fn rank_family_color(family: &str) -> Option<egui::Color32> {
    let (r, g, b) = match family {
        // Upper ranks (also used for stat values 1201..=2000).
        "UG" => (255, 105, 180), // hot pink
        "UF" => (255, 20, 147),  // deep pink
        "UE" => (30, 144, 255),  // dodger blue
        "UD" => (50, 205, 50),   // lime green
        "UC" => (169, 169, 169), // dark gray
        "UB" => (47, 79, 79),    // dark slate gray
        "UA" => (255, 215, 0),   // gold
        "US" => (255, 0, 0),     // red
        // Legend ranks (evaluation badge only).
        "LG" => (128, 0, 128),   // purple
        "LF" => (0, 206, 208),   // dark turquoise
        "LE" => (255, 165, 0),   // orange
        "LD" => (144, 238, 144), // light green
        "LC" => (192, 192, 192), // silver
        "LB" => (139, 69, 19),   // saddle brown
        "LA" => (0, 0, 139),     // dark blue
        "LS" => (0, 100, 0),     // dark green
        _ => return None,
    };
    Some(egui::Color32::from_rgb(r, g, b))
}

/// Decompose an evaluation rank label into colored segments for two-tone
/// rendering.
///
/// - `G`..`SS+` (no prefix): one segment, the whole label in its Table 1
///   base-letter color.
/// - `U*`/`L*` (e.g. `UG3`, `LF12`): two segments — the prefix char (`U`/`L`)
///   in its Table 2 family color, then the remainder in the base-letter color.
pub fn rank_badge_segments(label: &str) -> Vec<(String, egui::Color32)> {
    let bytes = label.as_bytes();
    if matches!(bytes.first(), Some(b'U') | Some(b'L')) && label.len() >= 2 {
        let family = &label[..2];
        if let Some(prefix_color) = rank_family_color(family) {
            let base_letter = &label[1..2];
            return vec![
                (label[..1].to_string(), prefix_color),
                (label[1..].to_string(), rank_letter_color(base_letter)),
            ];
        }
    }
    // Non-prefixed: strip a trailing `+`/`-` to find the base letter.
    let base_letter = label.trim_end_matches(['+', '-']);
    vec![(label.to_string(), rank_letter_color(base_letter))]
}

/// Color for a training failure rate %: green (safe) → yellow → orange → red.
pub(super) fn failure_rate_color(pct: i32) -> (u8, u8, u8) {
    if pct >= 60 {
        (255, 80, 80) // red - dangerous
    } else if pct >= 40 {
        (255, 140, 50) // orange
    } else if pct >= 20 {
        (255, 200, 50) // yellow - caution
    } else {
        (120, 220, 120) // green - safe
    }
}

/// Color for bond/friendship value: blue → green → orange → gold (max).
pub fn bond_color(value: i32) -> (u8, u8, u8) {
    if value >= 100 {
        (255, 200, 50) // Gold - maxed
    } else if value >= 80 {
        (255, 160, 40) // Orange - high
    } else if value >= 40 {
        (100, 220, 100) // Green - medium
    } else {
        (100, 150, 255) // Blue - low
    }
}

/// Colour for an editorial buy-priority tier.
pub(super) fn worth_color(w: memory_reader::Worth) -> egui::Color32 {
    match w {
        memory_reader::Worth::MustBuy => egui::Color32::from_rgb(110, 200, 110),
        memory_reader::Worth::Situational => egui::Color32::from_rgb(230, 200, 90),
        memory_reader::Worth::Optional => egui::Color32::from_rgb(120, 170, 220),
        memory_reader::Worth::Skip => egui::Color32::from_rgb(150, 150, 150),
    }
}

/// Format a number with comma separators.
pub(super) fn format_number(n: i32) -> String {
    if n < 0 {
        return format!("-{}", format_number(-n));
    }
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_reader;

    #[test]
    fn cap_level_thresholds() {
        assert_eq!(cap_level(1200, 0), CapLevel::Normal);
        assert_eq!(cap_level(0, 0), CapLevel::Normal);
        assert_eq!(cap_level(1000, 1200), CapLevel::Normal);
        assert_eq!(cap_level(1079, 1200), CapLevel::Normal);
        assert_eq!(cap_level(1080, 1200), CapLevel::Near);
        assert_eq!(cap_level(1199, 1200), CapLevel::Near);
        assert_eq!(cap_level(1200, 1200), CapLevel::AtCap);
        assert_eq!(cap_level(1300, 1200), CapLevel::AtCap);
    }

    #[test]
    fn stat_rank_color_thresholds() {
        assert_eq!(stat_rank_color(0), egui::Color32::from_rgb(128, 128, 128)); // G
        assert_eq!(stat_rank_color(99), egui::Color32::from_rgb(128, 128, 128)); // G+
        assert_eq!(stat_rank_color(100), egui::Color32::from_rgb(160, 82, 45)); // F
        assert_eq!(stat_rank_color(199), egui::Color32::from_rgb(160, 82, 45)); // F+
        assert_eq!(stat_rank_color(200), egui::Color32::from_rgb(186, 85, 211)); // E
        assert_eq!(stat_rank_color(300), egui::Color32::from_rgb(147, 112, 219)); // D
        assert_eq!(stat_rank_color(400), egui::Color32::from_rgb(30, 144, 255)); // C
        assert_eq!(stat_rank_color(599), egui::Color32::from_rgb(30, 144, 255)); // C+
        assert_eq!(stat_rank_color(600), egui::Color32::from_rgb(0, 206, 209)); // B
        assert_eq!(stat_rank_color(800), egui::Color32::from_rgb(0, 255, 0)); // A
        assert_eq!(stat_rank_color(1000), egui::Color32::from_rgb(255, 140, 0)); // S
        assert_eq!(stat_rank_color(1100), egui::Color32::from_rgb(255, 215, 0)); // SS
        assert_eq!(stat_rank_color(1200), egui::Color32::from_rgb(255, 215, 0)); // SS+
        assert_eq!(stat_rank_color(1201), egui::Color32::from_rgb(255, 105, 180)); // UG
        assert_eq!(stat_rank_color(1401), egui::Color32::from_rgb(30, 144, 255)); // UE
        assert_eq!(stat_rank_color(1901), egui::Color32::from_rgb(255, 0, 0)); // US
        assert_eq!(stat_rank_color(2000), egui::Color32::from_rgb(255, 0, 0)); // US9
    }

    #[test]
    fn rank_badge_segments_split() {
        // Non-prefixed: single segment in the base-letter color.
        let g = rank_badge_segments("G");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0], ("G".to_string(), egui::Color32::from_rgb(128, 128, 128)));

        let cp = rank_badge_segments("C+");
        assert_eq!(cp.len(), 1);
        assert_eq!(cp[0].1, egui::Color32::from_rgb(30, 144, 255)); // C base color

        let ssp = rank_badge_segments("SS+");
        assert_eq!(ssp.len(), 1);
        assert_eq!(ssp[0].1, egui::Color32::from_rgb(255, 215, 0)); // SS base color

        // U-rank: prefix in family color, remainder in base-letter color.
        let ug3 = rank_badge_segments("UG3");
        assert_eq!(ug3.len(), 2);
        assert_eq!(ug3[0], ("U".to_string(), egui::Color32::from_rgb(255, 105, 180)));
        assert_eq!(ug3[1], ("G3".to_string(), egui::Color32::from_rgb(128, 128, 128)));

        // L-rank with multi-digit sub-level.
        let lf12 = rank_badge_segments("LF12");
        assert_eq!(lf12.len(), 2);
        assert_eq!(lf12[0], ("L".to_string(), egui::Color32::from_rgb(0, 206, 208)));
        assert_eq!(lf12[1], ("F12".to_string(), egui::Color32::from_rgb(160, 82, 45)));
    }

    #[test]
    fn failure_rate_color_thresholds() {
        assert_eq!(failure_rate_color(0), (120, 220, 120));
        assert_eq!(failure_rate_color(19), (120, 220, 120));
        assert_eq!(failure_rate_color(20), (255, 200, 50));
        assert_eq!(failure_rate_color(39), (255, 200, 50));
        assert_eq!(failure_rate_color(40), (255, 140, 50));
        assert_eq!(failure_rate_color(59), (255, 140, 50));
        assert_eq!(failure_rate_color(60), (255, 80, 80));
        assert_eq!(failure_rate_color(100), (255, 80, 80));
    }

    #[test]
    fn bond_color_thresholds() {
        assert_eq!(bond_color(100), (255, 200, 50));
        assert_eq!(bond_color(150), (255, 200, 50));
        assert_eq!(bond_color(80), (255, 160, 40));
        assert_eq!(bond_color(99), (255, 160, 40));
        assert_eq!(bond_color(40), (100, 220, 100));
        assert_eq!(bond_color(79), (100, 220, 100));
        assert_eq!(bond_color(0), (100, 150, 255));
        assert_eq!(bond_color(39), (100, 150, 255));
    }

    #[test]
    fn format_number_basic() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn format_number_negative() {
        assert_eq!(format_number(-1000), "-1,000");
        assert_eq!(format_number(-42), "-42");
    }

    #[test]
    fn mood_labels() {
        assert!(memory_reader::mood_label(5).contains("Great"));
        assert!(memory_reader::mood_label(3).contains("Normal"));
        assert!(memory_reader::mood_label(1).contains("Terrible"));
        assert_eq!(memory_reader::mood_label(0), "???");
    }

    #[test]
    fn motivation_colors_distinct() {
        let colors: Vec<_> = (1..=5).map(memory_reader::motivation_color).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j]);
            }
        }
    }
}
