//! The `f` receiver — active-female-NPC reads.
//!
//! Each method mirrors a `Receiver::FemaleNpc` arm in `undone_expr::eval`
//! (`eval_call_bool` / `eval_call_string`), preserving the exact method name and
//! argument shape so existing condition strings work verbatim. The active female
//! NPC is resolved from the scene context; absence of one is a Rhai `Err`,
//! matching the `EvalError` variants in the legacy evaluator.

use crate::script::context::with_read_ctx;

type RhaiResult<T> = Result<T, Box<rhai::EvalAltResult>>;

/// Zero-sized `f` handle; reads the thread-local evaluation context.
#[derive(Clone)]
pub struct F;

impl F {
    // ── bool methods (eval_call_bool, Receiver::FemaleNpc) ───────────────────

    fn is_partner(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.is_partner())
        })
    }

    fn is_friend(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.is_friend())
        })
    }

    fn is_pregnant(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.pregnancy.is_some())
        })
    }

    fn is_virgin(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.virgin)
        })
    }

    fn has_flag(&mut self, flag: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.relationship_flags.contains(flag))
        })
    }

    fn has_role(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.roles.contains(role))
        })
    }

    // ── string methods (eval_call_string, Receiver::FemaleNpc) ───────────────

    fn get_liking(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.pc_liking.to_string())
        })
    }

    fn get_love(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(format!("{:?}", npc.core.pc_love))
        })
    }

    fn get_attraction(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(npc.core.pc_attraction.to_string())
        })
    }

    fn get_behaviour(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            let key = ctx.active_female.ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("no active female NPC in scene context")
            })?;
            let npc = world
                .female_npc(key)
                .ok_or_else(|| Box::<rhai::EvalAltResult>::from("NPC key not found in world"))?;
            Ok(format!("{:?}", npc.core.behaviour))
        })
    }
}

/// Register the `F` type and its methods. Names match the authored condition
/// syntax (`f.isPartner()`, `f.getLiking()`, …) exactly.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<F>()
        // bool
        .register_fn("isPartner", F::is_partner)
        .register_fn("isFriend", F::is_friend)
        .register_fn("isPregnant", F::is_pregnant)
        .register_fn("isVirgin", F::is_virgin)
        .register_fn("hasFlag", F::has_flag)
        .register_fn("hasRole", F::has_role)
        // string
        .register_fn("getLiking", F::get_liking)
        .register_fn("getLove", F::get_love)
        .register_fn("getAttraction", F::get_attraction)
        .register_fn("getBehaviour", F::get_behaviour);
}
