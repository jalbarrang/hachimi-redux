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
//! Shop reads are invoked from [`crate::core::modules::training_tracker::overlay_cache`] on the Unity main thread only.

use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::OnceLock;

use crate::core::modules::training_tracker::compat::Sdk;

use crate::core::modules::training_tracker::memory_reader;
use crate::core::modules::training_tracker::shop_hooks;

use super::il2cpp::{
    call_i32, call_obj, call_obj_i32, decrypt_obscured_int, read_field_i32, read_i32_list, read_string,
};
use super::logic::{pick_best_variant, sort_shop_entries, SkillCandidate};
use super::SkillShopEntry;

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

    let prefs = crate::core::modules::training_tracker::skill_shop_prefs::prefs();
    if prefs.show_hintless {
        merge_hintless_entries(&mut entries, msd, snp, r, &learned_ids);
    }

    sort_shop_entries(&mut entries, prefs.sort_mode);
    entries
}

// ---------------------------------------------------------------------------
// Per-skill master data reads
// ---------------------------------------------------------------------------

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
