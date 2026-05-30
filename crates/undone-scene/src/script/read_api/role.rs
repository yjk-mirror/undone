//! The `role` receiver — role-bound-NPC reads.
//!
//! Unlike `m`/`f`, every method takes the role id as its first argument
//! (`role.getName("ROLE_X")`, `role.hasFlag("ROLE_X", "flag")`). Each method
//! mirrors a `Receiver::RoleLookup` arm in `undone_expr::eval`
//! (`eval_call_bool` / `eval_call_string`), preserving the exact method name and
//! argument shape so existing condition strings work verbatim.

use crate::scene_ctx::SceneNpcRef;
use undone_world::World;

use crate::script::context::with_read_ctx;

type RhaiResult<T> = Result<T, Box<rhai::EvalAltResult>>;

/// Zero-sized `role` handle; reads the thread-local evaluation context.
#[derive(Clone)]
pub struct Role;

/// The NPC a role resolves to — mirrors `undone_expr::eval::ResolvedRoleNpc`.
enum ResolvedRoleNpc<'a> {
    Male(&'a undone_domain::MaleNpc),
    Female(&'a undone_domain::FemaleNpc),
}

/// Resolve a role id to its bound NPC, mirroring `eval::resolve_role_npc`.
///
/// Returns a Rhai runtime error matching the legacy `EvalError` messages when
/// the role is unbound (`no NPC bound for role '<role>'`) or its key is stale
/// (`NPC key not found in world`).
fn resolve_role_npc<'a>(
    role: &str,
    world: &'a World,
    ctx: &crate::scene_ctx::SceneCtx,
) -> RhaiResult<ResolvedRoleNpc<'a>> {
    match ctx
        .role_binding(role)
        .ok_or_else(|| -> Box<rhai::EvalAltResult> {
            format!("no NPC bound for role '{role}'").into()
        })? {
        SceneNpcRef::Male(key) => world
            .male_npc(key)
            .map(ResolvedRoleNpc::Male)
            .ok_or_else(|| -> Box<rhai::EvalAltResult> { "NPC key not found in world".into() }),
        SceneNpcRef::Female(key) => world
            .female_npc(key)
            .map(ResolvedRoleNpc::Female)
            .ok_or_else(|| -> Box<rhai::EvalAltResult> { "NPC key not found in world".into() }),
    }
}

impl Role {
    // ── bool methods (eval_call_bool, Receiver::RoleLookup) ───────────────────

    fn is_partner(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.is_partner(),
                ResolvedRoleNpc::Female(npc) => npc.core.is_partner(),
            })
        })
    }

    fn is_friend(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.is_friend(),
                ResolvedRoleNpc::Female(npc) => npc.core.is_friend(),
            })
        })
    }

    fn is_contactable(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.contactable,
                ResolvedRoleNpc::Female(npc) => npc.core.contactable,
            })
        })
    }

    fn is_pregnant(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(_) => false,
                ResolvedRoleNpc::Female(npc) => npc.pregnancy.is_some(),
            })
        })
    }

    fn is_virgin(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(_) => false,
                ResolvedRoleNpc::Female(npc) => npc.virgin,
            })
        })
    }

    fn had_orgasm(&mut self, role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.had_orgasm,
                ResolvedRoleNpc::Female(_) => false,
            })
        })
    }

    fn has_flag(&mut self, role: &str, flag: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.relationship_flags.contains(flag),
                ResolvedRoleNpc::Female(npc) => npc.core.relationship_flags.contains(flag),
            })
        })
    }

    fn has_role(&mut self, role: &str, nested_role: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.roles.contains(nested_role),
                ResolvedRoleNpc::Female(npc) => npc.core.roles.contains(nested_role),
            })
        })
    }

    // ── string methods (eval_call_string, Receiver::RoleLookup) ───────────────

    fn get_name(&mut self, role: &str) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.name.clone(),
                ResolvedRoleNpc::Female(npc) => npc.core.name.clone(),
            })
        })
    }

    fn get_liking(&mut self, role: &str) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.pc_liking.to_string(),
                ResolvedRoleNpc::Female(npc) => npc.core.pc_liking.to_string(),
            })
        })
    }

    fn get_love(&mut self, role: &str) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => format!("{:?}", npc.core.pc_love),
                ResolvedRoleNpc::Female(npc) => format!("{:?}", npc.core.pc_love),
            })
        })
    }

    fn get_attraction(&mut self, role: &str) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => npc.core.pc_attraction.to_string(),
                ResolvedRoleNpc::Female(npc) => npc.core.pc_attraction.to_string(),
            })
        })
    }

    fn get_behaviour(&mut self, role: &str) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, ctx| {
            Ok(match resolve_role_npc(role, world, ctx)? {
                ResolvedRoleNpc::Male(npc) => format!("{:?}", npc.core.behaviour),
                ResolvedRoleNpc::Female(npc) => format!("{:?}", npc.core.behaviour),
            })
        })
    }
}

/// Register the `Role` type and its methods. Names match the authored condition
/// syntax (`role.isPartner("ROLE_X")`, `role.getName("ROLE_X")`, …) exactly.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Role>()
        // bool
        .register_fn("isPartner", Role::is_partner)
        .register_fn("isFriend", Role::is_friend)
        .register_fn("isContactable", Role::is_contactable)
        .register_fn("isPregnant", Role::is_pregnant)
        .register_fn("isVirgin", Role::is_virgin)
        .register_fn("hadOrgasm", Role::had_orgasm)
        .register_fn("hasFlag", Role::has_flag)
        .register_fn("hasRole", Role::has_role)
        // string
        .register_fn("getName", Role::get_name)
        .register_fn("getLiking", Role::get_liking)
        .register_fn("getLove", Role::get_love)
        .register_fn("getAttraction", Role::get_attraction)
        .register_fn("getBehaviour", Role::get_behaviour);
}
