//! `w.*` write accessors. Bodies lifted from `write_api/player.rs`.

use undone_domain::{NpcKey, SkillValue};
use undone_packs::PackRegistry;
use undone_world::World;

use crate::effects::{resolve_npc_ref, step_alcohol, step_arousal, EffectError, NpcRef};
use crate::scene_ctx::SceneCtx;
use crate::script::api::ApiArg;

fn int0(a: &[ApiArg], m: &'static str) -> Result<i64, EffectError> {
    a.first()
        .and_then(ApiArg::as_int)
        .ok_or(EffectError::BadArgs(m))
}
fn str0<'a>(a: &[ApiArg<'a>], m: &'static str) -> Result<&'a str, EffectError> {
    a.first()
        .and_then(ApiArg::as_str)
        .ok_or(EffectError::BadArgs(m))
}

pub fn change_stress(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.player.stress.apply_delta(int0(a, "changeStress")? as i32);
    Ok(())
}

pub fn change_money(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.player.money += int0(a, "changeMoney")? as i32;
    Ok(())
}

pub fn change_anxiety(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.player
        .anxiety
        .apply_delta(int0(a, "changeAnxiety")? as i32);
    Ok(())
}

/// Change the structural COMPOSURE skill; clamps to the skill's min/max.
pub fn change_composure(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let amount = int0(a, "changeComposure")?;
    let sid = r
        .composure_skill()
        .map_err(|_| EffectError::UnknownSkill("COMPOSURE".to_string()))?;
    let entry = w.player.skills.entry(sid).or_insert(SkillValue {
        value: 0,
        modifier: 0,
    });
    entry.value += amount as i32;
    let def = r
        .get_skill_def(&sid)
        .expect("composure skill resolved above — def must exist");
    entry.value = entry.value.clamp(def.min, def.max);
    Ok(())
}

pub fn add_arousal(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let delta = int0(a, "addArousal")?;
    w.player.arousal = step_arousal(w.player.arousal, delta as i8);
    Ok(())
}

pub fn change_alcohol(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let delta = int0(a, "changeAlcohol")?;
    w.player.alcohol = step_alcohol(w.player.alcohol, delta as i8);
    Ok(())
}

pub fn skill_increase(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let skill = str0(a, "skillIncrease")?;
    let amount = a
        .get(1)
        .and_then(ApiArg::as_int)
        .ok_or(EffectError::BadArgs("skillIncrease"))?;
    let sid = r
        .resolve_skill(skill)
        .map_err(|_| EffectError::UnknownSkill(skill.to_string()))?;
    let entry = w.player.skills.entry(sid).or_insert(SkillValue {
        value: 0,
        modifier: 0,
    });
    entry.value += amount as i32;
    let def = r
        .get_skill_def(&sid)
        .expect("skill resolved above — def must exist");
    entry.value = entry.value.clamp(def.min, def.max);
    Ok(())
}

pub fn add_trait(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let trait_id = str0(a, "addTrait")?;
    let tid = r
        .resolve_trait(trait_id)
        .map_err(|_| EffectError::UnknownTrait(trait_id.to_string()))?;
    if let Some(conflict_msg) = r.check_trait_conflict(&w.player.traits, tid) {
        return Err(EffectError::TraitConflict(conflict_msg));
    }
    w.player.traits.insert(tid);
    Ok(())
}

pub fn remove_trait(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let trait_id = str0(a, "removeTrait")?;
    let tid = r
        .resolve_trait(trait_id)
        .map_err(|_| EffectError::UnknownTrait(trait_id.to_string()))?;
    w.player.traits.remove(&tid);
    Ok(())
}

pub fn add_stuff(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let item = str0(a, "addStuff")?;
    let stuff_id = r
        .resolve_stuff(item)
        .ok_or_else(|| EffectError::UnknownStuff(item.to_string()))?;
    w.player.stuff.insert(stuff_id);
    Ok(())
}

pub fn remove_stuff(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let item = str0(a, "removeStuff")?;
    let stuff_id = r
        .resolve_stuff(item)
        .ok_or_else(|| EffectError::UnknownStuff(item.to_string()))?;
    w.player.stuff.remove(&stuff_id);
    Ok(())
}

/// Overloaded: `setVirgin(value)` (vaginal) or `setVirgin(value, "type")`.
pub fn set_virgin(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let value = a
        .first()
        .and_then(ApiArg::as_bool)
        .ok_or(EffectError::BadArgs("setVirgin"))?;
    match a.get(1).and_then(ApiArg::as_str) {
        None => w.player.virgin = value,
        Some("vaginal") => w.player.virgin = value,
        Some("anal") => w.player.anal_virgin = value,
        Some("lesbian") => w.player.lesbian_virgin = value,
        Some(other) => {
            return Err(EffectError::UnknownVirginType(format!(
                "set_virgin: unknown virgin_type '{other}'"
            )))
        }
    }
    Ok(())
}

pub fn set_partner(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let npc = str0(a, "setPartner")?;
    let npc_key = match resolve_npc_ref(npc, c)? {
        NpcRef::Male(key) => NpcKey::Male(key),
        NpcRef::Female(key) => NpcKey::Female(key),
    };
    w.player.partner = Some(npc_key);
    Ok(())
}

pub fn add_friend(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let npc = str0(a, "addFriend")?;
    let npc_key = match resolve_npc_ref(npc, c)? {
        NpcRef::Male(key) => NpcKey::Male(key),
        NpcRef::Female(key) => NpcKey::Female(key),
    };
    if !w.player.friends.contains(&npc_key) {
        w.player.friends.push(npc_key);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_ctx::SceneCtx;
    use undone_world::test_helpers::make_test_world;

    #[test]
    fn change_money_applies_delta() {
        let mut w = make_test_world();
        let mut c = SceneCtx::new();
        let r = PackRegistry::new();
        let start = w.player.money;
        change_money(&mut w, &mut c, &r, &[ApiArg::Int(-30)]).unwrap();
        assert_eq!(w.player.money, start - 30);
    }

    #[test]
    fn set_virgin_one_and_two_arg() {
        let mut w = make_test_world();
        let mut c = SceneCtx::new();
        let r = PackRegistry::new();
        set_virgin(&mut w, &mut c, &r, &[ApiArg::Bool(false)]).unwrap();
        assert!(!w.player.virgin);
        set_virgin(
            &mut w,
            &mut c,
            &r,
            &[ApiArg::Bool(false), ApiArg::Str("anal")],
        )
        .unwrap();
        assert!(!w.player.anal_virgin);
    }

    #[test]
    fn add_trait_unknown_errors() {
        let mut w = make_test_world();
        let mut c = SceneCtx::new();
        let r = PackRegistry::new();
        assert!(matches!(
            add_trait(&mut w, &mut c, &r, &[ApiArg::Str("NOPE")]),
            Err(EffectError::UnknownTrait(_))
        ));
    }
}
