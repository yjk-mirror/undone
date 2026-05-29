//! The `m` receiver — active-male-NPC reads.
//!
//! Mirrors the `Receiver::MaleNpc` arms in `undone_expr::eval`
//! (`eval_call_bool` / `eval_call_string`), preserving the exact method name,
//! argument shape, and semantics so existing condition strings work verbatim.
//! The active male NPC is resolved from the thread-local scene context; absence
//! of an active male or a dangling key returns a Rhai `Err` matching the
//! `EvalError` messages.

use undone_domain::{AttractionLevel, LoveLevel};

use crate::script::context::{unknown_id_err, with_read_ctx};

type RhaiResult<T> = Result<T, Box<rhai::EvalAltResult>>;

/// Zero-sized `m` handle; reads the thread-local evaluation context.
#[derive(Clone)]
pub struct M;

impl M {
    // ── bool methods (eval_call_bool, Receiver::MaleNpc) ─────────────────────

    fn is_partner(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.is_partner())
        })
    }

    fn is_friend(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.is_friend())
        })
    }

    fn is_cohabiting(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.is_cohabiting())
        })
    }

    fn is_contactable(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.contactable)
        })
    }

    fn had_orgasm(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.had_orgasm)
        })
    }

    fn has_trait(&mut self, id: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            let trait_id = reg
                .resolve_npc_trait(id)
                .map_err(|_| unknown_id_err("trait", id))?;
            Ok(npc.core.has_trait(trait_id))
        })
    }

    fn is_npc_attraction_ok(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.npc_attraction >= AttractionLevel::Ok)
        })
    }

    fn is_npc_attraction_lust(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.npc_attraction == AttractionLevel::Lust)
        })
    }

    fn is_w_attraction_ok(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.pc_attraction >= AttractionLevel::Ok)
        })
    }

    fn is_npc_love_crush(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.npc_love >= LoveLevel::Crush)
        })
    }

    fn is_npc_love_some(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.npc_love >= LoveLevel::Some)
        })
    }

    fn is_w_love_crush(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.pc_love >= LoveLevel::Crush)
        })
    }

    fn has_flag(&mut self, flag: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.relationship_flags.contains(flag))
        })
    }

    fn has_role(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.roles.contains(role))
        })
    }

    // ── string methods (eval_call_string, Receiver::MaleNpc) ─────────────────

    fn get_liking(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.pc_liking.to_string())
        })
    }

    fn get_love(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(format!("{:?}", npc.core.pc_love))
        })
    }

    fn get_attraction(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.pc_attraction.to_string())
        })
    }

    fn get_behaviour(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_male.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active male NPC in scene context")
            })?;
            let npc = world
                .male_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(format!("{:?}", npc.core.behaviour))
        })
    }
}

/// Register the `M` type and its methods. Names match the authored condition
/// syntax (`m.isPartner()`, `m.getLiking()`, …) exactly.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<M>()
        // bool
        .register_fn("isPartner", M::is_partner)
        .register_fn("isFriend", M::is_friend)
        .register_fn("isCohabiting", M::is_cohabiting)
        .register_fn("isContactable", M::is_contactable)
        .register_fn("hadOrgasm", M::had_orgasm)
        .register_fn("hasTrait", M::has_trait)
        .register_fn("isNpcAttractionOk", M::is_npc_attraction_ok)
        .register_fn("isNpcAttractionLust", M::is_npc_attraction_lust)
        .register_fn("isWAttractionOk", M::is_w_attraction_ok)
        .register_fn("isNpcLoveCrush", M::is_npc_love_crush)
        .register_fn("isNpcLoveSome", M::is_npc_love_some)
        .register_fn("isWLoveCrush", M::is_w_love_crush)
        .register_fn("hasFlag", M::has_flag)
        .register_fn("hasRole", M::has_role)
        // string
        .register_fn("getLiking", M::get_liking)
        .register_fn("getLove", M::get_love)
        .register_fn("getAttraction", M::get_attraction)
        .register_fn("getBehaviour", M::get_behaviour);
}
