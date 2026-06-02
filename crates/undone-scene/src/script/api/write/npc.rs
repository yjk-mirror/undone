//! `npc(ref).*` write accessors. Bodies lifted from `write_api/npc.rs`.
//!
//! The resolved ref string is `ApiArg` index 0 (injected by the adapter from the
//! `npc(ref)` constructor); the method's own argument is index 1.

use undone_domain::NpcCore;
use undone_packs::PackRegistry;
use undone_world::World;

use crate::effects::{
    parse_behaviour, parse_relationship_status, resolve_npc_ref, step_attraction, step_liking,
    step_love, EffectError, NpcRef,
};
use crate::scene_ctx::SceneCtx;
use crate::script::api::ApiArg;

/// Ref is always `ApiArg` index 0.
fn ref0<'a>(a: &[ApiArg<'a>]) -> Result<&'a str, EffectError> {
    a.first()
        .and_then(ApiArg::as_str)
        .ok_or(EffectError::BadArgs("npc"))
}
fn arg_int(a: &[ApiArg], m: &'static str) -> Result<i64, EffectError> {
    a.get(1)
        .and_then(ApiArg::as_int)
        .ok_or(EffectError::BadArgs(m))
}
fn arg_str<'a>(a: &[ApiArg<'a>], m: &'static str) -> Result<&'a str, EffectError> {
    a.get(1)
        .and_then(ApiArg::as_str)
        .ok_or(EffectError::BadArgs(m))
}
fn arg_bool(a: &[ApiArg], m: &'static str) -> Result<bool, EffectError> {
    a.get(1)
        .and_then(ApiArg::as_bool)
        .ok_or(EffectError::BadArgs(m))
}

/// Resolve the ref to a mutable `&mut NpcCore` (male or female).
fn core_mut<'a>(
    w: &'a mut World,
    c: &SceneCtx,
    ref_: &str,
) -> Result<&'a mut NpcCore, EffectError> {
    match resolve_npc_ref(ref_, c)? {
        NpcRef::Male(key) => w
            .male_npc_mut(key)
            .map(|n| &mut n.core)
            .ok_or(EffectError::NpcNotFound),
        NpcRef::Female(key) => w
            .female_npc_mut(key)
            .map(|n| &mut n.core)
            .ok_or(EffectError::NpcNotFound),
    }
}

/// The `npc(ref)` constructor is handled specially by the Rhai adapter (it returns
/// a chained handle, not a mutation). This no-op exists only so the registry has a
/// descriptor for `("npc","npc")` the static gate + persistent-mutation lint can see;
/// it is never invoked as a mutator.
pub fn npc_ctor(
    _w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    _a: &[ApiArg],
) -> Result<(), EffectError> {
    Ok(())
}

pub fn add_liking(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let delta = arg_int(a, "addLiking")? as i8;
    let core = core_mut(w, c, ref0(a)?)?;
    core.pc_liking = step_liking(core.pc_liking, delta);
    Ok(())
}

pub fn add_love(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let delta = arg_int(a, "addLove")? as i8;
    let core = core_mut(w, c, ref0(a)?)?;
    core.npc_love = step_love(core.npc_love, delta);
    Ok(())
}

pub fn add_w_liking(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let delta = arg_int(a, "addWLiking")? as i8;
    let core = core_mut(w, c, ref0(a)?)?;
    core.npc_liking = step_liking(core.npc_liking, delta);
    Ok(())
}

pub fn set_attraction(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let delta = arg_int(a, "setAttraction")? as i8;
    let core = core_mut(w, c, ref0(a)?)?;
    core.npc_attraction = step_attraction(core.npc_attraction, delta);
    Ok(())
}

pub fn set_flag(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let flag = arg_str(a, "setFlag")?.to_string();
    let core = core_mut(w, c, ref0(a)?)?;
    core.relationship_flags.insert(flag);
    Ok(())
}

pub fn add_trait(
    w: &mut World,
    c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let trait_id = arg_str(a, "addTrait")?;
    let tid = r
        .resolve_npc_trait(trait_id)
        .map_err(|_| EffectError::UnknownNpcTrait(trait_id.to_string()))?;
    let core = core_mut(w, c, ref0(a)?)?;
    core.traits.insert(tid);
    Ok(())
}

pub fn set_relationship(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let status = arg_str(a, "setRelationship")?;
    let parsed = parse_relationship_status(status)
        .ok_or_else(|| EffectError::UnknownRelationshipStatus(status.to_string()))?;
    let core = core_mut(w, c, ref0(a)?)?;
    core.relationship = parsed;
    Ok(())
}

pub fn set_behaviour(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let behaviour = arg_str(a, "setBehaviour")?;
    let parsed = parse_behaviour(behaviour)
        .ok_or_else(|| EffectError::UnknownBehaviour(behaviour.to_string()))?;
    let core = core_mut(w, c, ref0(a)?)?;
    core.behaviour = parsed;
    Ok(())
}

pub fn set_contactable(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let value = arg_bool(a, "setContactable")?;
    let core = core_mut(w, c, ref0(a)?)?;
    core.contactable = value;
    Ok(())
}

pub fn add_sexual_activity(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let activity = arg_str(a, "addSexualActivity")?.to_string();
    let core = core_mut(w, c, ref0(a)?)?;
    core.sexual_activities.insert(activity);
    Ok(())
}

pub fn set_role(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let role = arg_str(a, "setRole")?.to_string();
    let core = core_mut(w, c, ref0(a)?)?;
    core.roles.insert(role);
    Ok(())
}

pub fn set_name(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let name = arg_str(a, "setName")?.to_string();
    let core = core_mut(w, c, ref0(a)?)?;
    core.display_name = Some(name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_ctx::SceneCtx;
    use undone_world::test_helpers::{make_test_male_npc, make_test_world};

    fn world_with_active_male() -> (World, SceneCtx, PackRegistry) {
        let mut r = PackRegistry::new();
        let personality = r.intern_personality("ROMANTIC");
        let mut w = make_test_world();
        let key = w.male_npcs.insert(make_test_male_npc(personality));
        let mut c = SceneCtx::new();
        c.active_male = Some(key);
        (w, c, r)
    }

    #[test]
    fn add_liking_steps_via_ref() {
        let (mut w, mut c, r) = world_with_active_male();
        add_liking(&mut w, &mut c, &r, &[ApiArg::Str("m"), ApiArg::Int(2)]).unwrap();
        let key = c.active_male.unwrap();
        assert_eq!(
            w.male_npcs.get(key).unwrap().core.pc_liking,
            undone_domain::LikingLevel::Like
        );
    }

    #[test]
    fn set_name_sets_display_name() {
        let (mut w, mut c, r) = world_with_active_male();
        set_name(&mut w, &mut c, &r, &[ApiArg::Str("m"), ApiArg::Str("Jake")]).unwrap();
        let key = c.active_male.unwrap();
        assert_eq!(w.male_npcs.get(key).unwrap().core.effective_name(), "Jake");
    }

    #[test]
    fn bad_ref_errors() {
        let (mut w, mut c, r) = world_with_active_male();
        assert!(matches!(
            add_liking(
                &mut w,
                &mut c,
                &r,
                &[ApiArg::Str("ROLE_NOPE"), ApiArg::Int(1)]
            ),
            Err(EffectError::BadNpcRef(_))
        ));
    }
}
