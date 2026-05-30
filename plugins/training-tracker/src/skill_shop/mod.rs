//! Skill shop feature: memory reconstruction, pure logic, and crypto.
//!
//! Submodules are re-exported flatly so existing `skill_shop::*` call sites
//! keep working:
//! - [`access`]: IL2CPP resolution + memory reading (unsafe)
//! - [`il2cpp`]: low-level read/call primitives
//! - [`logic`]: pure, unit-tested shop logic (discounts, sorting, filtering)
//! - [`crypto`]: ObscuredInt decryption

mod access;
mod crypto;
mod il2cpp;
mod logic;

pub(crate) use access::{read_skill_points, read_skill_shop};
pub use logic::{discount_pct, discounted_cost, prepare_entries_for_display, rarity_label};

/// A skill available in the shop, resolved from tips + master data.
#[derive(Debug, Clone)]
pub struct SkillShopEntry {
    pub skill_id: i32,
    pub group_id: i32,
    pub rarity: i32,
    pub hint_level: i32,
    pub name: String,
    pub base_cost: i32,
    pub is_learned: bool,
    /// `true` when derived from `_skillTipsList`; `false` for full-price (no hint) rows.
    pub has_hint: bool,
    /// Tag IDs from `MasterSkillData.SkillData.GetTagIds()` (distance/style/etc.).
    pub tags: Vec<i32>,
    /// `FilterSwitch` field — shop UI filter bitmask when tags are empty.
    pub filter_switch: i32,
}
