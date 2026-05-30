//! Skill shop reconstruction from game memory.
//!
//! Reads `_skillTipsList` from `WorkSingleModeCharaData` and resolves
//! skill names and costs via the game's own master data accessor chain:
//!
//! ```text
//! MasterDataManager (singleton)
//!   → get_masterSkillData() → MasterSkillData
//!     → GetListWithGroupIdOrderByIdAsc(group_id) → List<SkillData>
//!       → each: .Id, .Rarity, get_Name(), .GradeValue
//!   → get_masterSingleModeSkillNeedPoint() → MasterSingleModeSkillNeedPoint
//!     → Get(skill_id) → SingleModeSkillNeedPoint
//!       → .NeedSkillPoint
//! ```
//!
//! Shop reads are invoked from [`crate::overlay_cache`] on the Unity main thread only.

use std::ffi::c_void;
use std::sync::OnceLock;

use std::collections::HashSet;

use crate::memory_reader;
use crate::shop_hooks;
use crate::skill_shop_prefs::{DistanceFilter, ShopSortMode, SkillShopPrefs, StyleFilter};
use hachimi_plugin_sdk::Sdk;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Resolved IL2CPP pointers (all from one-time resolution)
// ---------------------------------------------------------------------------

struct Resolved {
    // MasterDataManager (singleton)
    mdm_klass: *mut c_void,
    m_get_master_skill_data: *const c_void, // → MasterSkillData
    m_get_skill_need_point: *const c_void,  // → MasterSingleModeSkillNeedPoint

    // MasterSkillData
    m_msd_get: *const c_void,               // Get(int) → SkillData
    m_msd_get_list_by_group: *const c_void, // GetListWithGroupIdOrderByIdAsc(int)

    // MasterSingleModeSkillNeedPoint
    m_snp_get: *const c_void, // Get(int) → SingleModeSkillNeedPoint

    // MasterSkillData.SkillData fields/methods
    f_sd_id: *mut c_void,
    f_sd_rarity: *mut c_void,
    f_sd_group_rate: *mut c_void,
    f_sd_group_id: *mut c_void,
    f_sd_filter_switch: *mut c_void,
    m_sd_get_name: *const c_void,
    m_sd_get_tag_ids: *const c_void, // GetTagIds() → List<Int32>

    // SingleModeSkillNeedPoint fields
    f_snp_need_skill_point: *mut c_void,

    // SkillTips backing fields
    f_tips_group_id: *mut c_void,
    f_tips_rarity: *mut c_void,
    f_tips_level: *mut c_void,

    // WorkSingleModeCharaData skill point
    f_skill_point: *mut c_void,
}

// SAFETY: IL2CPP pointers are stable for process lifetime.
unsafe impl Send for Resolved {}
// SAFETY: IL2CPP pointers are stable for process lifetime.
unsafe impl Sync for Resolved {}

static RESOLVED: OnceLock<Resolved> = OnceLock::new();

// ---------------------------------------------------------------------------
// IL2CPP helpers
// ---------------------------------------------------------------------------

#[inline]
unsafe fn mptr(mi: *const c_void) -> usize {
    // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
    unsafe { *(mi as *const usize) }
}

#[inline]
unsafe fn call_obj(this: *mut c_void, mi: *const c_void) -> *mut c_void {
    // SAFETY: Transmuting IL2CPP MethodInfo pointer to callable function pointer.
    let f: extern "C" fn(*mut c_void, *const c_void) -> *mut c_void = unsafe { std::mem::transmute(mptr(mi)) };
    f(this, mi)
}

#[inline]
unsafe fn call_i32(this: *mut c_void, mi: *const c_void) -> i32 {
    // SAFETY: Transmuting IL2CPP MethodInfo pointer to callable function pointer.
    let f: extern "C" fn(*mut c_void, *const c_void) -> i32 = unsafe { std::mem::transmute(mptr(mi)) };
    f(this, mi)
}

#[inline]
unsafe fn call_obj_i32(this: *mut c_void, mi: *const c_void, arg: i32) -> *mut c_void {
    // SAFETY: Transmuting IL2CPP MethodInfo pointer to callable function pointer.
    let f: extern "C" fn(*mut c_void, i32, *const c_void) -> *mut c_void = unsafe { std::mem::transmute(mptr(mi)) };
    f(this, arg, mi)
}

#[inline]
unsafe fn call_i32_i32(this: *mut c_void, mi: *const c_void, arg: i32) -> i32 {
    // SAFETY: Transmuting IL2CPP MethodInfo pointer to callable function pointer.
    let f: extern "C" fn(*mut c_void, i32, *const c_void) -> i32 = unsafe { std::mem::transmute(mptr(mi)) };
    f(this, arg, mi)
}

unsafe fn read_string(s: *mut c_void) -> Option<String> {
    // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
    unsafe {
        if s.is_null() {
            return None;
        }
        let len = *(s.byte_add(0x10) as *const i32);
        if len <= 0 || len > 4096 {
            return None;
        }
        String::from_utf16(std::slice::from_raw_parts(s.byte_add(0x14) as *const u16, len as usize)).ok()
    }
}

unsafe fn read_field_i32(obj: *mut c_void, field: *mut c_void) -> i32 {
    let mut v: i32 = 0;
    // SAFETY: IL2CPP object and field pointers from resolved metadata.
    unsafe {
        Sdk::get().get_field_value(obj.cast(), field.cast(), &mut v as *mut _ as *mut c_void);
    }
    v
}

unsafe fn decrypt_obscured_int(obj: *mut c_void, field: *mut c_void) -> i32 {
    let mut buf = [0u8; 16];
    // SAFETY: IL2CPP object and field pointers from resolved metadata.
    unsafe {
        Sdk::get().get_field_value(obj.cast(), field.cast(), buf.as_mut_ptr() as *mut c_void);
    }
    let raw: [u8; 8] = [buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]];
    decrypt_obscured_int_raw(&raw)
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

fn ensure_resolved() -> bool {
    if RESOLVED.get().is_some() {
        return true;
    }
    match try_resolve() {
        Ok(r) => {
            let _ = RESOLVED.set(r);
            true
        }
        Err(e) => {
            hlog_error!("Skill shop resolution failed: {}", e);
            false
        }
    }
}

macro_rules! resolve {
    (class $img:expr, $ns:literal, $name:literal) => {{
        let sdk = Sdk::get();
        let Some(k) = sdk.get_class($img, $ns, $name) else {
            return Err(concat!($name, " not found"));
        };
        k.cast::<c_void>()
    }};
    (nested $parent:expr, $name:literal) => {{
        let sdk = Sdk::get();
        let Some(k) = sdk.find_nested_class($parent.cast(), $name) else {
            return Err(concat!("nested ", $name, " not found"));
        };
        k.cast::<c_void>()
    }};
    (method $klass:expr, $name:literal, $args:expr) => {{
        let sdk = Sdk::get();
        let Some(m) = sdk.get_method($klass.cast(), $name, $args) else {
            return Err(concat!($name, " method not found"));
        };
        m.cast::<c_void>()
    }};
    (field $klass:expr, $name:literal) => {{
        let sdk = Sdk::get();
        let Some(f) = sdk.get_field_from_name($klass.cast(), $name) else {
            return Err(concat!($name, " field not found"));
        };
        f.cast::<c_void>()
    }};
    (field_opt $klass:expr, $name:literal) => {{
        Sdk::get()
            .get_field_from_name($klass.cast(), $name)
            .map(|f| f.cast::<c_void>())
            .unwrap_or(std::ptr::null_mut())
    }};
}

fn try_resolve() -> Result<Resolved, &'static str> {
    let sdk = Sdk::get();
    let Some(img) = sdk.get_assembly_image("umamusume.dll") else {
        return Err("umamusume.dll not found");
    };

    // MasterDataManager (singleton hub)
    let mdm = resolve!(class img, "Gallop", "MasterDataManager");
    let m_get_msd = resolve!(method mdm, "get_masterSkillData", 0);
    let m_get_snp = resolve!(method mdm, "get_masterSingleModeSkillNeedPoint", 0);

    // MasterSkillData
    let msd_klass = resolve!(class img, "Gallop", "MasterSkillData");
    let m_msd_get = resolve!(method msd_klass, "Get", 1);
    let m_msd_get_list = resolve!(method msd_klass, "GetListWithGroupIdOrderByIdAsc", 1);

    // MasterSkillData.SkillData (nested)
    let sd_klass = resolve!(nested msd_klass, "SkillData");
    let f_sd_id = resolve!(field sd_klass, "Id");
    let f_sd_rarity = resolve!(field sd_klass, "Rarity");
    let f_sd_grate = resolve!(field sd_klass, "GroupRate");
    let f_sd_gid = resolve!(field sd_klass, "GroupId");
    let f_sd_fswitch = resolve!(field_opt sd_klass, "FilterSwitch");
    let m_sd_name = resolve!(method sd_klass, "get_Name", 0);
    let m_sd_get_tag_ids = sdk
        .get_method(sd_klass.cast(), "GetTagIds", 0)
        .map(|m| m.cast::<c_void>())
        .unwrap_or(std::ptr::null());

    // MasterSingleModeSkillNeedPoint
    let snp_klass = resolve!(class img, "Gallop", "MasterSingleModeSkillNeedPoint");
    let m_snp_get = resolve!(method snp_klass, "Get", 1);
    let snp_row = resolve!(nested snp_klass, "SingleModeSkillNeedPoint");
    let f_snp_cost = resolve!(field snp_row, "NeedSkillPoint");

    // SkillTips
    let wsmcd = resolve!(class img, "Gallop", "WorkSingleModeCharaData");
    let tips = resolve!(nested wsmcd, "SkillTips");
    let f_gid = resolve!(field tips, "<GroupId>k__BackingField");
    let f_rar = resolve!(field tips, "<Rarity>k__BackingField");
    let f_lvl = resolve!(field tips, "<Level>k__BackingField");

    // SkillPoint
    let f_sp = resolve!(field_opt wsmcd, "<SkillPoint>k__BackingField");

    hlog_info!("Skill shop: full IL2CPP chain resolved (MasterDataManager → SkillData + NeedPoint)");
    Ok(Resolved {
        mdm_klass: mdm as _,
        m_get_master_skill_data: m_get_msd,
        m_get_skill_need_point: m_get_snp,
        m_msd_get,
        m_msd_get_list_by_group: m_msd_get_list,
        m_snp_get,
        f_sd_id,
        f_sd_rarity,
        f_sd_group_rate: f_sd_grate,
        f_sd_group_id: f_sd_gid,
        f_sd_filter_switch: f_sd_fswitch,
        m_sd_get_name: m_sd_name,
        m_sd_get_tag_ids,
        f_snp_need_skill_point: f_snp_cost,
        f_tips_group_id: f_gid,
        f_tips_rarity: f_rar,
        f_tips_level: f_lvl,
        f_skill_point: f_sp,
    })
}

// ---------------------------------------------------------------------------
// Public: read current SP
// ---------------------------------------------------------------------------

pub(crate) fn read_skill_points() -> Option<i32> {
    let r = RESOLVED.get()?;
    if r.f_skill_point.is_null() {
        return None;
    }
    let chara = memory_reader::get_chara_ptr()?;
    // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
    Some(unsafe { decrypt_obscured_int(chara, r.f_skill_point) })
}

// ---------------------------------------------------------------------------
// Core read
// ---------------------------------------------------------------------------

/// Full skill-shop reconstruction (main thread only — via overlay cache).
pub(crate) fn read_skill_shop() -> Vec<SkillShopEntry> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(read_skill_shop_inner)) {
        Ok(v) => v,
        Err(_) => {
            hlog_error!("read_skill_shop PANICKED");
            Vec::new()
        }
    }
}

fn read_skill_shop_inner() -> Vec<SkillShopEntry> {
    if !ensure_resolved() {
        return Vec::new();
    }
    let r = match RESOLVED.get() {
        Some(r) => r,
        None => return Vec::new(),
    };

    let chara = match memory_reader::get_chara_ptr() {
        Some(c) => c,
        None => return Vec::new(),
    };

    let mdm = Sdk::get()
        .get_singleton(r.mdm_klass.cast())
        .map(|p| p.cast::<c_void>())
        .unwrap_or(std::ptr::null_mut());
    if mdm.is_null() {
        hlog_warn!("MasterDataManager singleton is null");
        return Vec::new();
    }

    // Get master data table instances
    // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
    let msd = unsafe { call_obj(mdm, r.m_get_master_skill_data) };
    // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
    let snp = unsafe { call_obj(mdm, r.m_get_skill_need_point) };
    if msd.is_null() {
        hlog_warn!("MasterSkillData is null");
        return Vec::new();
    }

    // Learned IDs
    let learned = memory_reader::read_acquired_skills();
    let learned_ids: Vec<i32> = learned.iter().map(|s| s.master_id).collect();

    // Read tips
    // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
    let (list_ptr, count, m_get_item) = match unsafe { memory_reader::read_list_field(chara, c"_skillTipsList") } {
        Some(v) => v,
        None => return Vec::new(),
    };
    if count <= 0 || count > 500 {
        return Vec::new();
    }

    let mut entries = Vec::new();

    for i in 0..count {
        // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
        let item = unsafe { call_obj_i32(list_ptr, m_get_item, i) };
        if item.is_null() {
            continue;
        }

        // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
        let group_id = unsafe { decrypt_obscured_int(item, r.f_tips_group_id) };
        // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
        let tip_rarity = unsafe { decrypt_obscured_int(item, r.f_tips_rarity) };
        // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
        let level = unsafe { decrypt_obscured_int(item, r.f_tips_level) };

        // Expand group → concrete skills via MasterSkillData
        // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
        let skill_list = unsafe { call_obj_i32(msd, r.m_msd_get_list_by_group, group_id) };
        if skill_list.is_null() {
            continue;
        }

        // SAFETY: IL2CPP list object layout — klass pointer at object head.
        let list_klass = unsafe { *(skill_list as *const *mut c_void) };
        let sdk = Sdk::get();
        let Some(m_cnt) = sdk.get_method(list_klass.cast(), "get_Count", 0) else {
            continue;
        };
        let Some(m_itm) = sdk.get_method(list_klass.cast(), "get_Item", 1) else {
            continue;
        };
        if m_cnt.is_null() || m_itm.is_null() {
            continue;
        }

        // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
        let sk_count = unsafe { call_i32(skill_list, m_cnt) };

        // Find the lowest group_rate skill matching this tip's rarity
        // that hasn't been learned yet. The game requires buying skills
        // in order (○ before ◎), so show the next one to buy.
        let mut candidates: Vec<(*mut c_void, SkillCandidate)> = Vec::new();
        for j in 0..sk_count.min(20) {
            // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
            let sd = unsafe { call_obj_i32(skill_list, m_itm, j) };
            if sd.is_null() {
                continue;
            }

            // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
            let rarity = unsafe { read_field_i32(sd, r.f_sd_rarity) };
            if rarity != tip_rarity {
                continue;
            }

            // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
            let group_rate = unsafe { read_field_i32(sd, r.f_sd_group_rate) };
            if group_rate <= 0 {
                continue;
            } // skip × debuff variants

            // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
            let skill_id = unsafe { read_field_i32(sd, r.f_sd_id) };
            candidates.push((sd, SkillCandidate { skill_id, group_rate }));
        }

        let pure: Vec<SkillCandidate> = candidates.iter().map(|(_, c)| c.clone()).collect();
        let Some((skill_id, is_learned)) = pick_best_variant(&pure, &learned_ids) else {
            continue;
        };
        let Some(&(sd, _)) = candidates.iter().find(|(_, c)| c.skill_id == skill_id) else {
            continue;
        };
        // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
        let name = unsafe { read_string(call_obj(sd, r.m_sd_get_name)) }.unwrap_or_default();

        let base_cost = if !snp.is_null() {
            // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
            let row = unsafe { call_obj_i32(snp, r.m_snp_get, skill_id) };
            if !row.is_null() {
                // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
                unsafe { read_field_i32(row, r.f_snp_need_skill_point) }
            } else {
                0
            }
        } else {
            0
        };

        // SAFETY: sd is a valid SkillData from the group's list.
        let (tags, filter_switch) = unsafe { read_skill_tags(sd, r) };

        entries.push(SkillShopEntry {
            skill_id,
            group_id,
            rarity: tip_rarity,
            hint_level: level,
            name,
            base_cost,
            is_learned,
            has_hint: true,
            tags,
            filter_switch,
        });
    }

    let prefs = crate::skill_shop_prefs::prefs();
    if prefs.show_hintless {
        merge_hintless_entries(&mut entries, msd, snp, r, &learned_ids);
    }

    sort_shop_entries(&mut entries, prefs.sort_mode);
    entries
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

pub fn discount_pct(hint_level: i32, has_kiremono: bool) -> i32 {
    let base = match hint_level {
        0 => 0,
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 35,
        _ => 40,
    };
    base + if has_kiremono { 10 } else { 0 }
}

/// Apply a discount percentage to a base cost, returning the discounted cost.
/// Uses integer division: `base_cost * (100 - discount) / 100`.
pub fn discounted_cost(base_cost: i32, hint_level: i32, has_kiremono: bool) -> i32 {
    let pct = discount_pct(hint_level, has_kiremono);
    base_cost * (100 - pct) / 100
}

pub fn rarity_label(rarity: i32) -> &'static str {
    match rarity {
        1 => "\u{26aa}",  // ⚪
        2 => "\u{1f31f}", // 🌟
        _ => "?",
    }
}

// ---------------------------------------------------------------------------
// Pure logic extracted for testability
// ---------------------------------------------------------------------------

/// A skill candidate from MasterSkillData expansion (group_rate > 0, matching rarity).
/// This is the pure-data subset of what `read_skill_shop` collects per-group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillCandidate {
    pub skill_id: i32,
    pub group_rate: i32,
}

/// Pick the best skill variant from a list of candidates for a single tip.
///
/// Logic: sort by `group_rate` ascending, pick the first one whose `skill_id`
/// is NOT in `learned_ids`. If all are learned, return the last (highest rate)
/// and mark it learned.
///
/// Returns `(skill_id, is_learned)` or `None` if candidates is empty.
pub fn pick_best_variant(candidates: &[SkillCandidate], learned_ids: &[i32]) -> Option<(i32, bool)> {
    if candidates.is_empty() {
        return None;
    }

    let mut sorted: Vec<&SkillCandidate> = candidates.iter().collect();
    sorted.sort_by_key(|c| c.group_rate);

    // Pick lowest group_rate not yet learned
    if let Some(pick) = sorted.iter().find(|c| !learned_ids.contains(&c.skill_id)) {
        return Some((pick.skill_id, false));
    }

    // All learned → show the top one
    sorted.last().map(|c| (c.skill_id, true))
}

/// Sort shop entries according to [`ShopSortMode`].
pub fn sort_shop_entries(entries: &mut [SkillShopEntry], mode: ShopSortMode) {
    match mode {
        ShopSortMode::RarityThenName => {
            entries.sort_by(|a, b| b.rarity.cmp(&a.rarity).then(a.name.cmp(&b.name)));
        }
        ShopSortMode::NameOnly => {
            entries.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }
}

/// Whether an entry passes the overlay style/distance filters.
pub fn entry_matches_filters(entry: &SkillShopEntry, style: StyleFilter, distance: DistanceFilter) -> bool {
    let style_tag = style.tag_value();
    let dist_tag = distance.tag_value();
    if style_tag.is_none() && dist_tag.is_none() {
        return true;
    }
    if entry.tags.is_empty() {
        return true;
    }
    let style_ok = style_tag.is_none_or(|t| entry.tags.contains(&t));
    let dist_ok = dist_tag.is_none_or(|t| entry.tags.contains(&t));
    style_ok && dist_ok
}

/// Apply overlay filters and sort (for UI rendering).
pub fn prepare_entries_for_display(mut entries: Vec<SkillShopEntry>, prefs: &SkillShopPrefs) -> Vec<SkillShopEntry> {
    entries.retain(|e| !e.is_learned && entry_matches_filters(e, prefs.style_filter, prefs.distance_filter));
    sort_shop_entries(&mut entries, prefs.sort_mode);
    entries
}

unsafe fn read_i32_list(list: *mut c_void) -> Vec<i32> {
    if list.is_null() {
        return Vec::new();
    }
    // SAFETY: IL2CPP list object layout — klass pointer at object head.
    let list_klass = unsafe { *(list as *const *mut c_void) };
    let sdk = Sdk::get();
    let Some(m_cnt) = sdk.get_method(list_klass.cast(), "get_Count", 0) else {
        return Vec::new();
    };
    let Some(m_itm) = sdk.get_method(list_klass.cast(), "get_Item", 1) else {
        return Vec::new();
    };
    if m_cnt.is_null() || m_itm.is_null() {
        return Vec::new();
    }
    // SAFETY: Reading field or calling method on non-null IL2CPP object pointer.
    let count = unsafe { call_i32(list, m_cnt) };
    if count <= 0 || count > 32 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        // SAFETY: List<Int32>.get_Item returns the Int32 value directly, not a boxed object.
        out.push(unsafe { call_i32_i32(list, m_itm, i) });
    }
    out
}

unsafe fn read_skill_tags(sd: *mut c_void, r: &Resolved) -> (Vec<i32>, i32) {
    let filter_switch = if r.f_sd_filter_switch.is_null() {
        0
    } else {
        // SAFETY: Plain Int32 field on master SkillData.
        unsafe { read_field_i32(sd, r.f_sd_filter_switch) }
    };

    let tags = if r.m_sd_get_tag_ids.is_null() {
        Vec::new()
    } else {
        // SAFETY: GetTagIds on master SkillData row.
        let list = unsafe { call_obj(sd, r.m_sd_get_tag_ids) };
        // SAFETY: List pointer from GetTagIds on valid SkillData.
        unsafe { read_i32_list(list) }
    };

    (tags, filter_switch)
}

unsafe fn skill_need_point(skill_id: i32, snp: *mut c_void, r: &Resolved) -> i32 {
    if snp.is_null() {
        return 0;
    }
    // SAFETY: MasterSingleModeSkillNeedPoint.Get(skill_id).
    let row = unsafe { call_obj_i32(snp, r.m_snp_get, skill_id) };
    if row.is_null() {
        return 0;
    }
    // SAFETY: NeedSkillPoint field on row.
    unsafe { read_field_i32(row, r.f_snp_need_skill_point) }
}

unsafe fn build_entry_from_skill_data(
    sd: *mut c_void,
    snp: *mut c_void,
    r: &Resolved,
    learned_ids: &[i32],
    has_hint: bool,
    hint_level: i32,
) -> Option<SkillShopEntry> {
    if sd.is_null() {
        return None;
    }
    // SAFETY: Master SkillData fields.
    let skill_id = unsafe { read_field_i32(sd, r.f_sd_id) };
    // SAFETY: sd is valid MasterSkillData.SkillData from Get or list item.
    let (rarity, group_id, group_rate) = unsafe {
        (
            read_field_i32(sd, r.f_sd_rarity),
            read_field_i32(sd, r.f_sd_group_id),
            read_field_i32(sd, r.f_sd_group_rate),
        )
    };
    if group_rate <= 0 {
        return None;
    }
    let is_learned = learned_ids.contains(&skill_id);
    // SAFETY: get_Name on SkillData.
    let name = unsafe { read_string(call_obj(sd, r.m_sd_get_name)) }.unwrap_or_default();
    // SAFETY: snp table and sd row are valid master-data pointers.
    let base_cost = unsafe { skill_need_point(skill_id, snp, r) };
    // SAFETY: Tag list from GetTagIds on the same SkillData row.
    let (tags, filter_switch) = unsafe { read_skill_tags(sd, r) };
    Some(SkillShopEntry {
        skill_id,
        group_id,
        rarity,
        hint_level,
        name,
        base_cost,
        is_learned,
        has_hint,
        tags,
        filter_switch,
    })
}

fn merge_hintless_entries(
    entries: &mut Vec<SkillShopEntry>,
    msd: *mut c_void,
    snp: *mut c_void,
    r: &Resolved,
    learned_ids: &[i32],
) {
    let hinted_groups: HashSet<i32> = entries.iter().map(|e| e.group_id).collect();
    let hinted_ids: HashSet<i32> = entries.iter().map(|e| e.skill_id).collect();
    let visible = shop_hooks::visible_skill_ids();
    if visible.is_empty() {
        return;
    }

    for skill_id in visible {
        if hinted_ids.contains(&skill_id) {
            continue;
        }
        // SAFETY: MasterSkillData.Get(skill_id).
        let sd = unsafe { call_obj_i32(msd, r.m_msd_get, skill_id) };
        if sd.is_null() {
            continue;
        }
        // SAFETY: GroupId on SkillData.
        let group_id = unsafe { read_field_i32(sd, r.f_sd_group_id) };
        if hinted_groups.contains(&group_id) {
            continue;
        }
        // SAFETY: Build full-price row from master data.
        if let Some(entry) = unsafe { build_entry_from_skill_data(sd, snp, r, learned_ids, false, 0) } {
            entries.push(entry);
        }
    }
}

/// Decrypt an ObscuredInt from its raw 8-byte representation.
/// Layout: bytes [0..4] = cryptoKey (i32 LE), bytes [4..8] = hiddenValue (i32 LE).
/// Result: hiddenValue ^ cryptoKey.
pub fn decrypt_obscured_int_raw(buf: &[u8; 8]) -> i32 {
    let key = i32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let val = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
    val ^ key
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- discount_pct ----

    #[test]
    fn discount_pct_levels() {
        assert_eq!(discount_pct(0, false), 0);
        assert_eq!(discount_pct(1, false), 10);
        assert_eq!(discount_pct(2, false), 20);
        assert_eq!(discount_pct(3, false), 30);
        assert_eq!(discount_pct(4, false), 35);
        assert_eq!(discount_pct(5, false), 40);
        assert_eq!(discount_pct(99, false), 40); // clamps at 40
    }

    #[test]
    fn discount_pct_kiremono_adds_10() {
        assert_eq!(discount_pct(0, true), 10);
        assert_eq!(discount_pct(3, true), 40);
        assert_eq!(discount_pct(5, true), 50);
    }

    // ---- discounted_cost ----

    #[test]
    fn discounted_cost_basic() {
        assert_eq!(discounted_cost(100, 0, false), 100);
        assert_eq!(discounted_cost(100, 1, false), 90);
        assert_eq!(discounted_cost(100, 3, true), 60); // 30+10=40% off
        assert_eq!(discounted_cost(170, 2, false), 136); // 170 * 80 / 100
    }

    #[test]
    fn discounted_cost_truncates() {
        // Integer division truncation: 150 * 65 / 100 = 97 (not 97.5)
        assert_eq!(discounted_cost(150, 4, false), 97); // 35% off
    }

    // ---- rarity_label ----

    #[test]
    fn rarity_labels() {
        assert_eq!(rarity_label(1), "\u{26aa}");
        assert_eq!(rarity_label(2), "\u{1f31f}");
        assert_eq!(rarity_label(0), "?");
        assert_eq!(rarity_label(3), "?");
    }

    // ---- pick_best_variant ----

    #[test]
    fn pick_empty_candidates() {
        assert_eq!(pick_best_variant(&[], &[]), None);
    }

    #[test]
    fn pick_single_unlearned() {
        let cs = [SkillCandidate {
            skill_id: 100,
            group_rate: 1,
        }];
        assert_eq!(pick_best_variant(&cs, &[]), Some((100, false)));
    }

    #[test]
    fn pick_lowest_group_rate_first() {
        let cs = [
            SkillCandidate {
                skill_id: 200,
                group_rate: 2,
            },
            SkillCandidate {
                skill_id: 100,
                group_rate: 1,
            },
            SkillCandidate {
                skill_id: 300,
                group_rate: 3,
            },
        ];
        // Should pick skill_id=100 (lowest group_rate)
        assert_eq!(pick_best_variant(&cs, &[]), Some((100, false)));
    }

    #[test]
    fn pick_skips_learned() {
        let cs = [
            SkillCandidate {
                skill_id: 100,
                group_rate: 1,
            },
            SkillCandidate {
                skill_id: 200,
                group_rate: 2,
            },
        ];
        // 100 is learned, should pick 200
        assert_eq!(pick_best_variant(&cs, &[100]), Some((200, false)));
    }

    #[test]
    fn pick_all_learned_returns_highest() {
        let cs = [
            SkillCandidate {
                skill_id: 100,
                group_rate: 1,
            },
            SkillCandidate {
                skill_id: 200,
                group_rate: 2,
            },
        ];
        assert_eq!(pick_best_variant(&cs, &[100, 200]), Some((200, true)));
    }

    // ---- sort_shop_entries ----

    fn entry(name: &str, rarity: i32) -> SkillShopEntry {
        SkillShopEntry {
            skill_id: 0,
            group_id: 0,
            rarity,
            hint_level: 0,
            name: name.to_string(),
            base_cost: 0,
            is_learned: false,
            has_hint: true,
            tags: Vec::new(),
            filter_switch: 0,
        }
    }

    #[test]
    fn sort_gold_first_then_alpha() {
        let mut entries = vec![
            entry("Zetsu", 1),
            entry("Alpha", 2),
            entry("Beta", 1),
            entry("Gamma", 2),
        ];
        sort_shop_entries(&mut entries, ShopSortMode::RarityThenName);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "Gamma", "Beta", "Zetsu"]);
    }

    #[test]
    fn sort_name_only() {
        let mut entries = vec![entry("Zetsu", 2), entry("Alpha", 1), entry("Beta", 2)];
        sort_shop_entries(&mut entries, ShopSortMode::NameOnly);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["Alpha", "Beta", "Zetsu"]);
    }

    #[test]
    fn filter_style_and_distance() {
        let e = SkillShopEntry {
            skill_id: 1,
            group_id: 1,
            rarity: 1,
            hint_level: 0,
            name: "Test".into(),
            base_cost: 100,
            is_learned: false,
            has_hint: true,
            tags: vec![2, 12],
            filter_switch: 0,
        };
        assert!(entry_matches_filters(&e, StyleFilter::All, DistanceFilter::All));
        assert!(entry_matches_filters(&e, StyleFilter::Senko, DistanceFilter::All));
        assert!(!entry_matches_filters(&e, StyleFilter::Nige, DistanceFilter::All));
        assert!(entry_matches_filters(&e, StyleFilter::Senko, DistanceFilter::Mile));
        assert!(!entry_matches_filters(&e, StyleFilter::Senko, DistanceFilter::Short));
    }

    #[test]
    fn sort_stable_same_rarity() {
        let mut entries = vec![entry("C", 1), entry("A", 1), entry("B", 1)];
        sort_shop_entries(&mut entries, ShopSortMode::RarityThenName);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["A", "B", "C"]);
    }

    // ---- ObscuredInt decryption ----

    #[test]
    fn decrypt_obscured_int_basic() {
        // key=42, hiddenValue=42^100=78 → decrypted=78^42=100
        let key: i32 = 42;
        let plaintext: i32 = 100;
        let hidden = plaintext ^ key;
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&key.to_le_bytes());
        buf[4..8].copy_from_slice(&hidden.to_le_bytes());
        assert_eq!(decrypt_obscured_int_raw(&buf), 100);
    }

    #[test]
    fn decrypt_obscured_int_zero_key() {
        let mut buf = [0u8; 8];
        buf[4..8].copy_from_slice(&999i32.to_le_bytes());
        assert_eq!(decrypt_obscured_int_raw(&buf), 999);
    }

    #[test]
    fn decrypt_obscured_int_negative() {
        let key: i32 = 0x1234_5678;
        let plaintext: i32 = -50;
        let hidden = plaintext ^ key;
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&key.to_le_bytes());
        buf[4..8].copy_from_slice(&hidden.to_le_bytes());
        assert_eq!(decrypt_obscured_int_raw(&buf), -50);
    }

    #[test]
    fn decrypt_obscured_int_roundtrip_all_bits() {
        // Verify XOR is its own inverse
        for &(key, plain) in &[(0xFF_FF_FF_FFu32 as i32, 0), (1, i32::MAX), (i32::MIN, i32::MIN)] {
            let hidden = plain ^ key;
            let mut buf = [0u8; 8];
            buf[0..4].copy_from_slice(&key.to_le_bytes());
            buf[4..8].copy_from_slice(&hidden.to_le_bytes());
            assert_eq!(decrypt_obscured_int_raw(&buf), plain);
        }
    }
}
