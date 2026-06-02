//! `scene.*` write accessors (scene-local, NON-persistent). Lifted from
//! `write_api/scene.rs`.

use undone_packs::PackRegistry;
use undone_world::World;

use crate::effects::EffectError;
use crate::scene_ctx::SceneCtx;
use crate::script::api::ApiArg;

fn str0<'a>(a: &[ApiArg<'a>], m: &'static str) -> Result<&'a str, EffectError> {
    a.first()
        .and_then(ApiArg::as_str)
        .ok_or(EffectError::BadArgs(m))
}

pub fn set_flag(
    _w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    c.set_flag(str0(a, "setFlag")?.to_string());
    Ok(())
}

pub fn remove_flag(
    _w: &mut World,
    c: &mut SceneCtx,
    _r: &PackRegistry,
    a: &[ApiArg],
) -> Result<(), EffectError> {
    c.scene_flags.remove(str0(a, "removeFlag")?);
    Ok(())
}
