//! `gd.*` write accessors. Bodies lifted from `write_api/game_data.rs`.

use undone_packs::PackRegistry;
use undone_world::World;

use crate::effects::EffectError;
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

pub fn set_game_flag(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.game_data.set_flag(str0(a, "setGameFlag")?.to_string());
    Ok(())
}

pub fn remove_game_flag(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.game_data.remove_flag(str0(a, "removeGameFlag")?);
    Ok(())
}

pub fn add_stat(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let stat = str0(a, "addStat")?;
    let amount = a
        .get(1)
        .and_then(ApiArg::as_int)
        .ok_or(EffectError::BadArgs("addStat"))?;
    let sid = r
        .get_stat(stat)
        .ok_or_else(|| EffectError::UnknownStat(stat.to_string()))?;
    w.game_data.add_stat(sid, amount as i32);
    Ok(())
}

pub fn set_stat(
    w: &mut World,
    _c: &mut SceneCtx,
    r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let stat = str0(a, "setStat")?;
    let value = a
        .get(1)
        .and_then(ApiArg::as_int)
        .ok_or(EffectError::BadArgs("setStat"))?;
    let sid = r
        .get_stat(stat)
        .ok_or_else(|| EffectError::UnknownStat(stat.to_string()))?;
    w.game_data.set_stat(sid, value as i32);
    Ok(())
}

pub fn set_job_title(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.game_data.job_title = str0(a, "setJobTitle")?.to_string();
    Ok(())
}

pub fn add_desire(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.game_data.add_desire(int0(a, "addDesire")? as i32);
    Ok(())
}

pub fn set_desire(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    w.game_data.set_desire(int0(a, "setDesire")? as i32);
    Ok(())
}

pub fn advance_time(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let slots = int0(a, "advanceTime")?;
    for _ in 0..slots {
        w.game_data.advance_time_slot();
    }
    Ok(())
}

/// `advanceArc(arc, to_state)` — two string args (state validated by the gate).
pub fn advance_arc(
    w: &mut World,
    _c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let arc = str0(a, "advanceArc")?;
    let to_state = a
        .get(1)
        .and_then(ApiArg::as_str)
        .ok_or(EffectError::BadArgs("advanceArc"))?;
    w.game_data.advance_arc(arc, to_state);
    Ok(())
}

pub fn fail_red_check(
    w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    let skill = str0(a, "failRedCheck")?;
    let scene_id = c.scene_id.as_deref().unwrap_or("unknown").to_string();
    w.game_data.fail_red_check(&scene_id, skill);
    Ok(())
}
