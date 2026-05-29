//! `npc("m"|"f"|role).*` effect mutators — NPC writes.
//!
//! Each method mirrors an `apply_effect` arm in `effects.rs` that targets an
//! NPC, wrapped in `with_write_ctx` for continue-on-error. The `Npc` handle
//! carries the unresolved NPC reference string (`"m"`, `"f"`, or a role name);
//! every method resolves it through `resolve_npc_ref` inside the write context,
//! exactly as the corresponding match arm does.

use crate::effects::{
    parse_behaviour, parse_relationship_status, resolve_npc_ref, step_attraction, step_liking,
    step_love, EffectError, NpcRef,
};
use crate::script::context::with_write_ctx;

/// Handle for `npc("m"|"f"|role)` — holds the unresolved NPC reference.
#[derive(Clone)]
pub struct Npc {
    id: String,
}

/// `npc("m")` — construct an NPC handle from a reference string.
fn npc(id: &str) -> Npc {
    Npc { id: id.to_string() }
}

impl Npc {
    fn add_liking(&mut self, delta: i64) {
        let id = self.id.clone();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.pc_liking = step_liking(npc_data.core.pc_liking, delta as i8);
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.pc_liking = step_liking(npc_data.core.pc_liking, delta as i8);
                }
            }
            Ok(())
        });
    }

    fn add_love(&mut self, delta: i64) {
        let id = self.id.clone();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.npc_love = step_love(npc_data.core.npc_love, delta as i8);
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.npc_love = step_love(npc_data.core.npc_love, delta as i8);
                }
            }
            Ok(())
        });
    }

    fn add_w_liking(&mut self, delta: i64) {
        let id = self.id.clone();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.npc_liking = step_liking(npc_data.core.npc_liking, delta as i8);
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.npc_liking = step_liking(npc_data.core.npc_liking, delta as i8);
                }
            }
            Ok(())
        });
    }

    fn set_attraction(&mut self, delta: i64) {
        let id = self.id.clone();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.npc_attraction =
                        step_attraction(npc_data.core.npc_attraction, delta as i8);
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.npc_attraction =
                        step_attraction(npc_data.core.npc_attraction, delta as i8);
                }
            }
            Ok(())
        });
    }

    fn set_flag(&mut self, flag: &str) {
        let id = self.id.clone();
        let flag = flag.to_string();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.relationship_flags.insert(flag.clone());
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.relationship_flags.insert(flag.clone());
                }
            }
            Ok(())
        });
    }

    fn add_trait(&mut self, trait_id: &str) {
        let id = self.id.clone();
        let trait_id = trait_id.to_string();
        with_write_ctx(move |world, ctx, reg| {
            let tid = reg
                .resolve_npc_trait(&trait_id)
                .map_err(|_| EffectError::UnknownNpcTrait(trait_id.clone()))?;
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.traits.insert(tid);
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.traits.insert(tid);
                }
            }
            Ok(())
        });
    }

    fn set_relationship(&mut self, status: &str) {
        let id = self.id.clone();
        let status = status.to_string();
        with_write_ctx(move |world, ctx, _reg| {
            let parsed = parse_relationship_status(&status)
                .ok_or_else(|| EffectError::UnknownRelationshipStatus(status.clone()))?;
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.relationship = parsed;
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.relationship = parsed;
                }
            }
            Ok(())
        });
    }

    fn set_behaviour(&mut self, behaviour: &str) {
        let id = self.id.clone();
        let behaviour = behaviour.to_string();
        with_write_ctx(move |world, ctx, _reg| {
            let parsed = parse_behaviour(&behaviour)
                .ok_or_else(|| EffectError::UnknownBehaviour(behaviour.clone()))?;
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.behaviour = parsed;
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.behaviour = parsed;
                }
            }
            Ok(())
        });
    }

    fn set_contactable(&mut self, value: bool) {
        let id = self.id.clone();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.contactable = value;
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.contactable = value;
                }
            }
            Ok(())
        });
    }

    fn add_sexual_activity(&mut self, activity: &str) {
        let id = self.id.clone();
        let activity = activity.to_string();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.sexual_activities.insert(activity.clone());
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.sexual_activities.insert(activity.clone());
                }
            }
            Ok(())
        });
    }

    fn set_role(&mut self, role: &str) {
        let id = self.id.clone();
        let role = role.to_string();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.roles.insert(role.clone());
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.roles.insert(role.clone());
                }
            }
            Ok(())
        });
    }

    fn set_name(&mut self, name: &str) {
        let id = self.id.clone();
        let name = name.to_string();
        with_write_ctx(move |world, ctx, _reg| {
            match resolve_npc_ref(&id, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.display_name = Some(name.clone());
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.display_name = Some(name.clone());
                }
            }
            Ok(())
        });
    }
}

/// Register the `npc(...).* ` effect mutators. Names are the authored effect
/// vocabulary.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Npc>()
        .register_fn("npc", npc)
        .register_fn("addLiking", Npc::add_liking)
        .register_fn("addLove", Npc::add_love)
        .register_fn("addWLiking", Npc::add_w_liking)
        .register_fn("setAttraction", Npc::set_attraction)
        .register_fn("setFlag", Npc::set_flag)
        .register_fn("addTrait", Npc::add_trait)
        .register_fn("setRelationship", Npc::set_relationship)
        .register_fn("setBehaviour", Npc::set_behaviour)
        .register_fn("setContactable", Npc::set_contactable)
        .register_fn("addSexualActivity", Npc::add_sexual_activity)
        .register_fn("setRole", Npc::set_role)
        .register_fn("setName", Npc::set_name);
}
