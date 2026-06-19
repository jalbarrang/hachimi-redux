//! Design tokens — the single source of truth for the kit's look, mirroring the
//! honse-tracker Uma kit palette (`apps/web/src/index.css` `@theme` vars).
//!
//! Components read these instead of hardcoding colors, so re-skinning the whole
//! kit is a one-file edit — the shadcn `tailwind.config`/CSS-variable idea, but
//! as Rust consts. Colors are `#rrggbb` / `#rrggbbaa` strings because the
//! renderer parses style attributes from strings.

// ── Surfaces & lines ──────────────────────────────────────────────────────
/// App background (`--color-bg`).
pub const BG: &str = "#0b0e13";
/// Textured background stop (`--color-bg-tex`).
pub const BG_TEX: &str = "#0d1017";
/// Raised panel surface (`--color-surface-1`).
pub const SURFACE_1: &str = "#151a23";
/// Nested panel / chip surface (`--color-surface-2`).
pub const SURFACE_2: &str = "#1c2230";
/// Deepest nested surface (`--color-surface-3`).
pub const SURFACE_3: &str = "#242c3d";
pub const LINE: &str = "#2c3648";
/// Subtle divider (`--color-line-subtle`).
pub const LINE_SUBTLE: &str = "#202938";
/// Fully transparent — used where a variant wants no fill/border.
pub const TRANSPARENT: &str = "#00000000";

// ── Foreground (text) ─────────────────────────────────────────────────────
pub const FG: &str = "#eaeff6";
pub const FG_MUTED: &str = "#a3b1c4";
pub const FG_DIM: &str = "#6e7d92";

// ── Uma green ramp ────────────────────────────────────────────────────────
pub const UMA_300: &str = "#8fe08f";
pub const UMA_400: &str = "#6fd06f";
pub const UMA_500: &str = "#4fbb4f";
pub const UMA_600: &str = "#379e37";
pub const UMA_700: &str = "#267a26";

// ── Semantic accents (nearest index.css equivalents) ──────────────────────
/// Blue accent — mapped to stat-speed (`--color-stat-speed`).
pub const ACCENT: &str = "#5fb2ff";
/// Green success / primary CTA — mapped to uma-500 (`--color-uma-500`).
pub const GOOD: &str = UMA_500;
/// Amber warning — mapped to stat-power (`--color-stat-power`).
pub const WARN: &str = "#ffb04d";
/// Red danger — mapped to grade-a (`--color-grade-a`).
pub const BAD: &str = "#ff7a6b";
/// Alias for destructive actions.
pub const DESTRUCTIVE: &str = BAD;

// ── Soft tints (pill/badge treatment) ─────────────────────────────────────
/// Translucent fills mirroring `rgba(..., 0.15–0.2)` in index.css.
pub const GOOD_SOFT: &str = "#4fbb4f26";
pub const ACCENT_SOFT: &str = "#5fb2ff26";
pub const WARN_SOFT: &str = "#ffb04d33";
pub const DESTRUCTIVE_SOFT: &str = "#ff7a6b33";
pub const NEUTRAL_SOFT: &str = "#a3b1c426";

// ── Shape & spacing scale ─────────────────────────────────────────────────
/// Corner radius for cards/panels (`--radius-uma` = 12px).
pub const RADIUS: &str = "12";
/// Corner radius for buttons/chips (`--radius-uma-sm` = 8px).
pub const RADIUS_SM: &str = "8";
/// Large panel radius (`--radius-uma-lg` = 16px).
pub const RADIUS_LG: &str = "16";
/// Corner radius for status badges.
pub const RADIUS_BADGE: &str = "4";
/// Pill radius (999px in CSS). Capped at the renderer's `u8` radius limit;
/// large enough to fully round typical chip/badge heights.
pub const RADIUS_PILL: &str = "255";

// ── Game data palettes ────────────────────────────────────────────────────

/// Training grade letter (game order: S gold, A salmon, B pink, C green, D blue, E purple, G gray).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
    E,
    G,
}

impl Grade {
    pub const fn color(self) -> &'static str {
        match self {
            Self::S => "#ffd44d",
            Self::A => "#ff7a6b",
            Self::B => "#ff8fb0",
            Self::C => "#6fd06f",
            Self::D => "#6db5ff",
            Self::E => "#c79bff",
            Self::G => "#8a93a0",
        }
    }
}

/// Stat column color.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Speed,
    Stamina,
    Power,
    Guts,
    Wit,
}

impl Stat {
    pub const fn color(self) -> &'static str {
        match self {
            Self::Speed => "#5fb2ff",
            Self::Stamina => "#ff8a5c",
            Self::Power => "#ffb04d",
            Self::Guts => "#ff7a90",
            Self::Wit => "#4ddcb0",
        }
    }
}

/// Mood indicator color.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Great,
    Good,
    Normal,
    Bad,
    Awful,
}

impl Mood {
    pub const fn color(self) -> &'static str {
        match self {
            Self::Great => "#e85f9c",
            Self::Good => "#ff9a3d",
            Self::Normal => "#c2a83d",
            Self::Bad => "#4d8fd6",
            Self::Awful => "#a86fd6",
        }
    }
}

/// Race tier color.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    G1,
    G2,
    G3,
}

impl Tier {
    pub const fn color(self) -> &'static str {
        match self {
            Self::G1 => "#4d9fff",
            Self::G2 => "#ff5f9e",
            Self::G3 => "#3dcc88",
        }
    }
}

/// Build a soft tinted fill (`#rrggbb26`) from a solid base color.
pub fn soft_fill(base: &str) -> &'static str {
    match base {
        "#ffd44d" => "#ffd44d26",
        "#ff7a6b" => "#ff7a6b33",
        "#ff8fb0" => "#ff8fb026",
        "#6fd06f" => "#6fd06f26",
        "#6db5ff" => "#6db5ff26",
        "#c79bff" => "#c79bff26",
        "#8a93a0" => "#8a93a026",
        "#5fb2ff" => "#5fb2ff26",
        "#ff8a5c" => "#ff8a5c26",
        "#ffb04d" => "#ffb04d33",
        "#ff7a90" => "#ff7a9026",
        "#4ddcb0" => "#4ddcb026",
        "#e85f9c" => "#e85f9c26",
        "#ff9a3d" => "#ff9a3d26",
        "#c2a83d" => "#c2a83d26",
        "#4d8fd6" => "#4d8fd626",
        "#a86fd6" => "#a86fd626",
        "#4d9fff" => "#4d9fff26",
        "#ff5f9e" => "#ff5f9e26",
        "#3dcc88" => "#3dcc8826",
        _ => "#a3b1c426",
    }
}
