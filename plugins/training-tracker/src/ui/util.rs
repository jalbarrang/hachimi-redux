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
