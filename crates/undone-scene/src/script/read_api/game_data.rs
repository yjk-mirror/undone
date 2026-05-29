//! The `gd` receiver — game-data reads.
//!
//! Each method mirrors a `Receiver::GameData` arm in `undone_expr::eval`
//! (`eval_call_bool` / `eval_call_int` / `eval_call_string`), preserving the exact
//! method name and argument shape so existing condition strings work verbatim.

use undone_world::World;

use crate::script::context::with_read_ctx;

type RhaiResult<T> = Result<T, Box<rhai::EvalAltResult>>;

/// Parse a liking-level threshold string (mirrors `parse_liking_level` in eval.rs).
fn parse_liking_level(s: &str) -> Option<undone_domain::LikingLevel> {
    match s {
        "Neutral" => Some(undone_domain::LikingLevel::Neutral),
        "Ok" => Some(undone_domain::LikingLevel::Ok),
        "Like" => Some(undone_domain::LikingLevel::Like),
        "Close" => Some(undone_domain::LikingLevel::Close),
        _ => None,
    }
}

/// Find the PC liking of the first NPC (male, then female) holding `role`,
/// defaulting to `Neutral` (mirrors `find_npc_liking_by_role` in eval.rs).
fn find_npc_liking_by_role(role: &str, world: &World) -> undone_domain::LikingLevel {
    world
        .male_npcs
        .values()
        .find(|npc| npc.core.roles.contains(role))
        .map(|npc| npc.core.pc_liking)
        .or_else(|| {
            world
                .female_npcs
                .values()
                .find(|npc| npc.core.roles.contains(role))
                .map(|npc| npc.core.pc_liking)
        })
        .unwrap_or(undone_domain::LikingLevel::Neutral)
}

/// Zero-sized `gd` handle; reads the thread-local evaluation context.
#[derive(Clone)]
pub struct Gd;

impl Gd {
    // ── bool methods (eval_call_bool, Receiver::GameData) ─────────────────────

    fn has_game_flag(&mut self, flag: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.has_flag(flag)))
    }

    fn is_weekday(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.is_weekday()))
    }

    fn is_weekend(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.is_weekend()))
    }

    fn arc_started(&mut self, arc_id: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.arc_state(arc_id).is_some()))
    }

    fn npc_liking_at_least(&mut self, role: &str, threshold: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| {
            let threshold = parse_liking_level(threshold).ok_or_else(|| {
                Box::<rhai::EvalAltResult>::from("bad argument to npcLikingAtLeast")
            })?;
            let liking = find_npc_liking_by_role(role, world);
            Ok(liking >= threshold)
        })
    }

    // ── int methods (eval_call_int, Receiver::GameData) ───────────────────────

    fn week(&mut self) -> RhaiResult<i64> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.week as i64))
    }

    fn day(&mut self) -> RhaiResult<i64> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.day as i64))
    }

    fn get_stat(&mut self, id: &str) -> RhaiResult<i64> {
        with_read_ctx(|world, reg, _ctx| match reg.get_stat(id) {
            Some(stat_id) => Ok(world.game_data.get_stat(stat_id) as i64),
            None => Ok(0), // stat never interned = was never set
        })
    }

    // ── string methods (eval_call_string, Receiver::GameData) ─────────────────

    fn time_slot(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.game_data.time_slot)))
    }

    fn get_job_title(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.job_title.clone()))
    }

    fn arc_state(&mut self, arc_id: &str) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world.game_data.arc_state(arc_id).unwrap_or("").to_string())
        })
    }

    fn npc_liking(&mut self, role: &str) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            let liking = world
                .male_npcs
                .values()
                .find(|npc| npc.core.roles.contains(role))
                .map(|npc| npc.core.pc_liking.to_string())
                .or_else(|| {
                    world
                        .female_npcs
                        .values()
                        .find(|npc| npc.core.roles.contains(role))
                        .map(|npc| npc.core.pc_liking.to_string())
                })
                .unwrap_or_else(|| "Neutral".to_string());
            Ok(liking)
        })
    }
}

/// Register the `Gd` type and its methods. Names match the authored condition
/// syntax (`gd.hasGameFlag(...)`, `gd.week()`, …) exactly.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Gd>()
        // bool
        .register_fn("hasGameFlag", Gd::has_game_flag)
        .register_fn("isWeekday", Gd::is_weekday)
        .register_fn("isWeekend", Gd::is_weekend)
        .register_fn("arcStarted", Gd::arc_started)
        .register_fn("npcLikingAtLeast", Gd::npc_liking_at_least)
        // int
        .register_fn("week", Gd::week)
        .register_fn("day", Gd::day)
        .register_fn("getStat", Gd::get_stat)
        // string
        .register_fn("timeSlot", Gd::time_slot)
        .register_fn("getJobTitle", Gd::get_job_title)
        .register_fn("arcState", Gd::arc_state)
        .register_fn("npcLiking", Gd::npc_liking);
}
