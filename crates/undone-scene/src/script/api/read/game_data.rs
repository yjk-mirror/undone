//! `gd` (game-data) read accessors. Bodies lifted verbatim from
//! `read_api/game_data.rs`.

use undone_world::World;

use crate::scene_ctx::SceneCtx;
use crate::script::api::{ApiArg, ApiError, ApiValue};
use undone_packs::PackRegistry;

fn str0<'a>(a: &[ApiArg<'a>], method: &'static str) -> Result<&'a str, ApiError> {
    a.first()
        .and_then(ApiArg::as_str)
        .ok_or(ApiError::BadArgs { method })
}

/// Parse a liking-level threshold string (mirrors `parse_liking_level`).
fn parse_liking_level(s: &str) -> Option<undone_domain::LikingLevel> {
    match s {
        "Neutral" => Some(undone_domain::LikingLevel::Neutral),
        "Ok" => Some(undone_domain::LikingLevel::Ok),
        "Like" => Some(undone_domain::LikingLevel::Like),
        "Close" => Some(undone_domain::LikingLevel::Close),
        _ => None,
    }
}

/// First NPC (male, then female) holding `role`, defaulting to `Neutral`.
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

// ── bool ────────────────────────────────────────────────────────────────────

pub fn has_game_flag(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let flag = str0(a, "hasGameFlag")?;
    Ok(ApiValue::Bool(w.game_data.has_flag(flag)))
}

pub fn is_weekday(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.game_data.is_weekday()))
}

pub fn is_weekend(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.game_data.is_weekend()))
}

pub fn arc_started(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let arc_id = str0(a, "arcStarted")?;
    Ok(ApiValue::Bool(w.game_data.arc_state(arc_id).is_some()))
}

/// Role + threshold, both string args (threshold e.g. `"Like"`).
pub fn npc_liking_at_least(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let role = str0(a, "npcLikingAtLeast")?;
    let threshold = a
        .get(1)
        .and_then(ApiArg::as_str)
        .and_then(parse_liking_level)
        .ok_or(ApiError::BadArgs {
            method: "npcLikingAtLeast",
        })?;
    let liking = find_npc_liking_by_role(role, w);
    Ok(ApiValue::Bool(liking >= threshold))
}

// ── int ───────────────────────────────────────────────────────────────────────

pub fn week(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Int(w.game_data.week as i64))
}

pub fn day(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Int(w.game_data.day as i64))
}

pub fn desire(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Int(w.game_data.desire() as i64))
}

/// Stat id is NOT registry-validated: an un-interned stat returns 0 (matches Rhai).
pub fn get_stat(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let id = str0(a, "getStat")?;
    Ok(ApiValue::Int(match r.get_stat(id) {
        Some(stat_id) => w.game_data.get_stat(stat_id) as i64,
        None => 0,
    }))
}

// ── string ──────────────────────────────────────────────────────────────────

pub fn time_slot(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.game_data.time_slot)))
}

pub fn get_job_title(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(w.game_data.job_title.clone()))
}

/// Unstarted arc → `""` (authors branch on `== ""`).
pub fn arc_state(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let arc_id = str0(a, "arcState")?;
    Ok(ApiValue::Str(
        w.game_data.arc_state(arc_id).unwrap_or("").to_string(),
    ))
}

/// Role liking display string, defaulting to `"Neutral"`.
pub fn npc_liking(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let role = str0(a, "npcLiking")?;
    let liking = w
        .male_npcs
        .values()
        .find(|npc| npc.core.roles.contains(role))
        .map(|npc| npc.core.pc_liking.to_string())
        .or_else(|| {
            w.female_npcs
                .values()
                .find(|npc| npc.core.roles.contains(role))
                .map(|npc| npc.core.pc_liking.to_string())
        })
        .unwrap_or_else(|| "Neutral".to_string());
    Ok(ApiValue::Str(liking))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_ctx::SceneCtx;
    use undone_world::test_helpers::make_test_world;

    #[test]
    fn week_and_desire_match_world() {
        let mut w = make_test_world();
        w.game_data.week = 3;
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(week(&w, &r, &c, &[]).unwrap(), ApiValue::Int(3));
        assert_eq!(
            desire(&w, &r, &c, &[]).unwrap(),
            ApiValue::Int(w.game_data.desire() as i64)
        );
    }

    #[test]
    fn get_stat_unregistered_is_zero() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(
            get_stat(&w, &r, &c, &[ApiArg::Str("NOPE")]).unwrap(),
            ApiValue::Int(0)
        );
    }

    #[test]
    fn arc_state_unstarted_is_empty() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(
            arc_state(&w, &r, &c, &[ApiArg::Str("base::nope")]).unwrap(),
            ApiValue::Str(String::new())
        );
    }

    #[test]
    fn npc_liking_default_neutral() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(
            npc_liking(&w, &r, &c, &[ApiArg::Str("ROLE_NONE")]).unwrap(),
            ApiValue::Str("Neutral".to_string())
        );
    }
}
