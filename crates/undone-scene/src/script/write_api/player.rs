//! `w.*` effect mutators — player and player-relationship writes.
//!
//! Worked reference for the write-API port. Each method mirrors an `apply_effect`
//! arm in `effects.rs` that targets the player, wrapped in `with_write_ctx` for
//! continue-on-error. Methods are added to the `W` handle defined in
//! `read_api::player` (Rust allows inherent impls across modules in a crate).

use undone_domain::{NpcKey, SkillValue};

use crate::effects::{resolve_npc_ref, step_alcohol, step_arousal, EffectError, NpcRef};
use crate::script::context::with_write_ctx;
use crate::script::read_api::player::W;

impl W {
    fn change_stress(&mut self, amount: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            world.player.stress.apply_delta(amount as i32);
            Ok(())
        });
    }

    fn change_money(&mut self, amount: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            world.player.money += amount as i32;
            Ok(())
        });
    }

    fn change_anxiety(&mut self, amount: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            world.player.anxiety.apply_delta(amount as i32);
            Ok(())
        });
    }

    /// Change the structural COMPOSURE skill (giving in to desire lowers it).
    /// Convenience wrapper over `skillIncrease("COMPOSURE", n)`; clamps to the
    /// skill's min/max.
    fn change_composure(&mut self, amount: i64) {
        with_write_ctx(|world, _ctx, reg| {
            let sid = reg
                .composure_skill()
                .map_err(|_| EffectError::UnknownSkill("COMPOSURE".to_string()))?;
            let entry = world.player.skills.entry(sid).or_insert(SkillValue {
                value: 0,
                modifier: 0,
            });
            entry.value += amount as i32;
            let def = reg
                .get_skill_def(&sid)
                .expect("composure skill resolved above — def must exist");
            entry.value = entry.value.clamp(def.min, def.max);
            Ok(())
        });
    }

    fn add_arousal(&mut self, delta: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            world.player.arousal = step_arousal(world.player.arousal, delta as i8);
            Ok(())
        });
    }

    fn change_alcohol(&mut self, delta: i64) {
        with_write_ctx(|world, _ctx, _reg| {
            world.player.alcohol = step_alcohol(world.player.alcohol, delta as i8);
            Ok(())
        });
    }

    fn skill_increase(&mut self, skill: &str, amount: i64) {
        with_write_ctx(|world, _ctx, reg| {
            let sid = reg
                .resolve_skill(skill)
                .map_err(|_| EffectError::UnknownSkill(skill.to_string()))?;
            let entry = world.player.skills.entry(sid).or_insert(SkillValue {
                value: 0,
                modifier: 0,
            });
            entry.value += amount as i32;
            let def = reg
                .get_skill_def(&sid)
                .expect("skill resolved above — def must exist");
            entry.value = entry.value.clamp(def.min, def.max);
            Ok(())
        });
    }

    fn add_trait(&mut self, trait_id: &str) {
        with_write_ctx(|world, _ctx, reg| {
            let tid = reg
                .resolve_trait(trait_id)
                .map_err(|_| EffectError::UnknownTrait(trait_id.to_string()))?;
            if let Some(conflict_msg) = reg.check_trait_conflict(&world.player.traits, tid) {
                return Err(EffectError::TraitConflict(conflict_msg));
            }
            world.player.traits.insert(tid);
            Ok(())
        });
    }

    fn remove_trait(&mut self, trait_id: &str) {
        with_write_ctx(|world, _ctx, reg| {
            let tid = reg
                .resolve_trait(trait_id)
                .map_err(|_| EffectError::UnknownTrait(trait_id.to_string()))?;
            world.player.traits.remove(&tid);
            Ok(())
        });
    }

    fn add_stuff(&mut self, item: &str) {
        with_write_ctx(|world, _ctx, reg| {
            let stuff_id = reg
                .resolve_stuff(item)
                .ok_or_else(|| EffectError::UnknownStuff(item.to_string()))?;
            world.player.stuff.insert(stuff_id);
            Ok(())
        });
    }

    fn remove_stuff(&mut self, item: &str) {
        with_write_ctx(|world, _ctx, reg| {
            let stuff_id = reg
                .resolve_stuff(item)
                .ok_or_else(|| EffectError::UnknownStuff(item.to_string()))?;
            world.player.stuff.remove(&stuff_id);
            Ok(())
        });
    }

    /// `w.setVirgin(false)` — vaginal virginity (matches EffectDef default).
    fn set_virgin(&mut self, value: bool) {
        with_write_ctx(|world, _ctx, _reg| {
            world.player.virgin = value;
            Ok(())
        });
    }

    /// `w.setVirgin(false, "anal")` — typed virginity.
    fn set_virgin_typed(&mut self, value: bool, virgin_type: &str) {
        with_write_ctx(|world, _ctx, _reg| {
            match virgin_type {
                "vaginal" => world.player.virgin = value,
                "anal" => world.player.anal_virgin = value,
                "lesbian" => world.player.lesbian_virgin = value,
                other => {
                    return Err(EffectError::UnknownVirginType(format!(
                        "set_virgin: unknown virgin_type '{other}'"
                    )))
                }
            }
            Ok(())
        });
    }

    fn set_partner(&mut self, npc: &str) {
        with_write_ctx(|world, ctx, _reg| {
            let npc_key = match resolve_npc_ref(npc, ctx)? {
                NpcRef::Male(key) => NpcKey::Male(key),
                NpcRef::Female(key) => NpcKey::Female(key),
            };
            world.player.partner = Some(npc_key);
            Ok(())
        });
    }

    fn add_friend(&mut self, npc: &str) {
        with_write_ctx(|world, ctx, _reg| {
            let npc_key = match resolve_npc_ref(npc, ctx)? {
                NpcRef::Male(key) => NpcKey::Male(key),
                NpcRef::Female(key) => NpcKey::Female(key),
            };
            if !world.player.friends.contains(&npc_key) {
                world.player.friends.push(npc_key);
            }
            Ok(())
        });
    }
}

/// Register the `w.*` effect mutators. Names are the authored effect vocabulary.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_fn("changeStress", W::change_stress)
        .register_fn("changeMoney", W::change_money)
        .register_fn("changeAnxiety", W::change_anxiety)
        .register_fn("changeComposure", W::change_composure)
        .register_fn("addArousal", W::add_arousal)
        .register_fn("changeAlcohol", W::change_alcohol)
        .register_fn("skillIncrease", W::skill_increase)
        .register_fn("addTrait", W::add_trait)
        .register_fn("removeTrait", W::remove_trait)
        .register_fn("addStuff", W::add_stuff)
        .register_fn("removeStuff", W::remove_stuff)
        .register_fn("setVirgin", W::set_virgin)
        .register_fn("setVirgin", W::set_virgin_typed)
        .register_fn("setPartner", W::set_partner)
        .register_fn("addFriend", W::add_friend);
}
