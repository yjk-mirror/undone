//! `m` (active male) and `f` (active female) read accessors. Bodies lifted from
//! `read_api/male_npc.rs` / `read_api/female_npc.rs`.
//!
//! `getName` is ADDED to both `m` and `f` (the Rhai surface lacked it; prose had it
//! on the NPC objects). It returns `effective_name()` — the unified `getName`
//! decision (design §1, §4.3): the story-assigned display name, not the spawn name.

use undone_domain::{AttractionLevel, FemaleNpc, LoveLevel, MaleNpc};
use undone_packs::PackRegistry;
use undone_world::World;

use crate::scene_ctx::SceneCtx;
use crate::script::api::{ApiArg, ApiError, ApiValue};

fn str0<'a>(a: &[ApiArg<'a>], method: &'static str) -> Result<&'a str, ApiError> {
    a.first()
        .and_then(ApiArg::as_str)
        .ok_or(ApiError::BadArgs { method })
}

fn male<'a>(w: &'a World, c: &SceneCtx) -> Result<&'a MaleNpc, ApiError> {
    let key = c.active_male.ok_or(ApiError::NoActiveNpc { sex: "male" })?;
    w.male_npc(key).ok_or(ApiError::NpcNotFound)
}

fn female<'a>(w: &'a World, c: &SceneCtx) -> Result<&'a FemaleNpc, ApiError> {
    let key = c
        .active_female
        .ok_or(ApiError::NoActiveNpc { sex: "female" })?;
    w.female_npc(key).ok_or(ApiError::NpcNotFound)
}

// ── m (active male) ───────────────────────────────────────────────────────────

pub fn m_is_partner(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(male(w, c)?.core.is_partner()))
}

pub fn m_is_friend(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(male(w, c)?.core.is_friend()))
}

pub fn m_is_cohabiting(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(male(w, c)?.core.is_cohabiting()))
}

pub fn m_is_contactable(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(male(w, c)?.core.contactable))
}

pub fn m_had_orgasm(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(male(w, c)?.had_orgasm))
}

pub fn m_has_trait(
    w: &World,
    r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let npc = male(w, c)?;
    let id = str0(a, "hasTrait")?;
    let trait_id = r.resolve_npc_trait(id).map_err(|_| ApiError::UnknownId {
        kind: "trait",
        id: id.to_string(),
    })?;
    Ok(ApiValue::Bool(npc.core.has_trait(trait_id)))
}

pub fn m_is_npc_attraction_ok(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(
        male(w, c)?.core.npc_attraction >= AttractionLevel::Ok,
    ))
}

pub fn m_is_npc_attraction_lust(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(
        male(w, c)?.core.npc_attraction == AttractionLevel::Lust,
    ))
}

pub fn m_is_w_attraction_ok(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(
        male(w, c)?.core.pc_attraction >= AttractionLevel::Ok,
    ))
}

pub fn m_is_npc_love_crush(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(
        male(w, c)?.core.npc_love >= LoveLevel::Crush,
    ))
}

pub fn m_is_npc_love_some(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(male(w, c)?.core.npc_love >= LoveLevel::Some))
}

pub fn m_is_w_love_crush(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(male(w, c)?.core.pc_love >= LoveLevel::Crush))
}

pub fn m_has_flag(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let npc = male(w, c)?;
    let flag = str0(a, "hasFlag")?;
    Ok(ApiValue::Bool(npc.core.relationship_flags.contains(flag)))
}

pub fn m_has_role(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let npc = male(w, c)?;
    let role = str0(a, "hasRole")?;
    Ok(ApiValue::Bool(npc.core.roles.contains(role)))
}

pub fn m_get_liking(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(male(w, c)?.core.pc_liking.to_string()))
}

pub fn m_get_love(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", male(w, c)?.core.pc_love)))
}

pub fn m_get_attraction(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(male(w, c)?.core.pc_attraction.to_string()))
}

pub fn m_get_behaviour(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", male(w, c)?.core.behaviour)))
}

/// ADDED to `m` — unified `getName` = `effective_name()` (design §4.3).
pub fn m_get_name(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(male(w, c)?.core.effective_name().to_string()))
}

// ── f (active female) ─────────────────────────────────────────────────────────

pub fn f_is_partner(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(female(w, c)?.core.is_partner()))
}

pub fn f_is_friend(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(female(w, c)?.core.is_friend()))
}

pub fn f_is_pregnant(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(female(w, c)?.pregnancy.is_some()))
}

pub fn f_is_virgin(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(female(w, c)?.virgin))
}

pub fn f_has_flag(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let npc = female(w, c)?;
    let flag = str0(a, "hasFlag")?;
    Ok(ApiValue::Bool(npc.core.relationship_flags.contains(flag)))
}

pub fn f_has_role(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let npc = female(w, c)?;
    let role = str0(a, "hasRole")?;
    Ok(ApiValue::Bool(npc.core.roles.contains(role)))
}

pub fn f_get_liking(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(female(w, c)?.core.pc_liking.to_string()))
}

pub fn f_get_love(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", female(w, c)?.core.pc_love)))
}

pub fn f_get_attraction(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(female(w, c)?.core.pc_attraction.to_string()))
}

pub fn f_get_behaviour(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", female(w, c)?.core.behaviour)))
}

/// ADDED to `f` — unified `getName` = `effective_name()` (design §4.3).
pub fn f_get_name(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        female(w, c)?.core.effective_name().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_ctx::SceneCtx;
    use undone_world::test_helpers::{make_test_male_npc, make_test_world};

    #[test]
    fn m_no_active_male_errors() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert!(matches!(
            m_is_partner(&w, &r, &c, &[]),
            Err(ApiError::NoActiveNpc { sex: "male" })
        ));
    }

    #[test]
    fn m_get_name_is_effective_name() {
        let mut r = PackRegistry::new();
        let personality = r.intern_personality("ROMANTIC");
        let mut w = make_test_world();
        let mut male_npc = make_test_male_npc(personality);
        male_npc.core.display_name = Some("Theo".into());
        let key = w.male_npcs.insert(male_npc);
        let mut c = SceneCtx::new();
        c.active_male = Some(key);
        assert_eq!(
            m_get_name(&w, &r, &c, &[]).unwrap(),
            ApiValue::Str("Theo".to_string())
        );
    }
}
