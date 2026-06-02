//! `role` (role-bound NPC) read accessors. Every method takes the role id as
//! source-arg 0 (`role.getName("ROLE_X")`). Bodies lifted from `read_api/role.rs`.
//!
//! `getName` is lifted to `effective_name()` (NOT `core.name`) — the headline
//! divergence fix (design §1): prose already used the display name; Rhai used the
//! raw spawn name. The unified accessor adopts the display name.

use undone_domain::{FemaleNpc, MaleNpc};
use undone_packs::PackRegistry;
use undone_world::World;

use crate::scene_ctx::{SceneCtx, SceneNpcRef};
use crate::script::api::{ApiArg, ApiError, ApiValue};

enum Resolved<'a> {
    Male(&'a MaleNpc),
    Female(&'a FemaleNpc),
}

fn role0<'a>(a: &[ApiArg<'a>]) -> Result<&'a str, ApiError> {
    a.first()
        .and_then(ApiArg::as_str)
        .ok_or(ApiError::BadArgs { method: "role" })
}

/// Resolve a role id to its bound NPC. Unbound → `UnboundRole`; stale key →
/// `NpcNotFound` (mirrors `read_api/role.rs::resolve_role_npc`).
fn resolve<'a>(role: &str, w: &'a World, c: &SceneCtx) -> Result<Resolved<'a>, ApiError> {
    match c.role_binding(role).ok_or(ApiError::UnboundRole {
        role: role.to_string(),
    })? {
        SceneNpcRef::Male(key) => w
            .male_npc(key)
            .map(Resolved::Male)
            .ok_or(ApiError::NpcNotFound),
        SceneNpcRef::Female(key) => w
            .female_npc(key)
            .map(Resolved::Female)
            .ok_or(ApiError::NpcNotFound),
    }
}

// ── bool ────────────────────────────────────────────────────────────────────

pub fn is_partner(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => npc.core.is_partner(),
        Resolved::Female(npc) => npc.core.is_partner(),
    }))
}

pub fn is_friend(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => npc.core.is_friend(),
        Resolved::Female(npc) => npc.core.is_friend(),
    }))
}

pub fn is_contactable(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => npc.core.contactable,
        Resolved::Female(npc) => npc.core.contactable,
    }))
}

pub fn is_pregnant(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(match resolve(role0(a)?, w, c)? {
        Resolved::Male(_) => false,
        Resolved::Female(npc) => npc.pregnancy.is_some(),
    }))
}

pub fn is_virgin(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(match resolve(role0(a)?, w, c)? {
        Resolved::Male(_) => false,
        Resolved::Female(npc) => npc.virgin,
    }))
}

pub fn had_orgasm(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => npc.had_orgasm,
        Resolved::Female(_) => false,
    }))
}

pub fn has_flag(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let role = role0(a)?;
    let flag = a
        .get(1)
        .and_then(ApiArg::as_str)
        .ok_or(ApiError::BadArgs { method: "hasFlag" })?;
    Ok(ApiValue::Bool(match resolve(role, w, c)? {
        Resolved::Male(npc) => npc.core.relationship_flags.contains(flag),
        Resolved::Female(npc) => npc.core.relationship_flags.contains(flag),
    }))
}

pub fn has_role(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let role = role0(a)?;
    let nested = a
        .get(1)
        .and_then(ApiArg::as_str)
        .ok_or(ApiError::BadArgs { method: "hasRole" })?;
    Ok(ApiValue::Bool(match resolve(role, w, c)? {
        Resolved::Male(npc) => npc.core.roles.contains(nested),
        Resolved::Female(npc) => npc.core.roles.contains(nested),
    }))
}

// ── string ──────────────────────────────────────────────────────────────────

/// `getName` → `effective_name()` (display name), NOT `core.name` (spawn name).
pub fn get_name(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => npc.core.effective_name().to_string(),
        Resolved::Female(npc) => npc.core.effective_name().to_string(),
    }))
}

pub fn get_liking(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => npc.core.pc_liking.to_string(),
        Resolved::Female(npc) => npc.core.pc_liking.to_string(),
    }))
}

pub fn get_love(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => format!("{:?}", npc.core.pc_love),
        Resolved::Female(npc) => format!("{:?}", npc.core.pc_love),
    }))
}

pub fn get_attraction(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => npc.core.pc_attraction.to_string(),
        Resolved::Female(npc) => npc.core.pc_attraction.to_string(),
    }))
}

pub fn get_behaviour(
    w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(match resolve(role0(a)?, w, c)? {
        Resolved::Male(npc) => format!("{:?}", npc.core.behaviour),
        Resolved::Female(npc) => format!("{:?}", npc.core.behaviour),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_ctx::SceneCtx;
    use undone_world::test_helpers::{make_test_male_npc, make_test_world};

    #[test]
    fn get_name_is_effective_name_not_spawn() {
        let mut r = PackRegistry::new();
        let personality = r.intern_personality("ROMANTIC");
        let mut w = make_test_world();
        let mut male_npc = make_test_male_npc(personality); // core.name = "Jake"
        male_npc.core.display_name = Some("Theo".into());
        let key = w.male_npcs.insert(male_npc);
        let mut c = SceneCtx::new();
        c.bind_role("ROLE_X", SceneNpcRef::Male(key));
        assert_eq!(
            get_name(&w, &r, &c, &[ApiArg::Str("ROLE_X")]).unwrap(),
            ApiValue::Str("Theo".to_string())
        );
    }

    #[test]
    fn unbound_role_errors() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert!(matches!(
            get_name(&w, &r, &c, &[ApiArg::Str("ROLE_NOPE")]),
            Err(ApiError::UnboundRole { .. })
        ));
    }
}
