//! `scene` (scene-local flag) read accessors. Lifted from `read_api/scene.rs`.

use undone_packs::PackRegistry;
use undone_world::World;

use crate::scene_ctx::SceneCtx;
use crate::script::api::{ApiArg, ApiError, ApiValue};

pub fn has_flag(
    _w: &World,
    _r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let flag = a
        .first()
        .and_then(ApiArg::as_str)
        .ok_or(ApiError::BadArgs { method: "hasFlag" })?;
    Ok(ApiValue::Bool(c.has_flag(flag)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_ctx::SceneCtx;
    use undone_world::test_helpers::make_test_world;

    #[test]
    fn has_flag_reads_scene_ctx() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let mut c = SceneCtx::new();
        c.set_flag("umbrella");
        assert_eq!(
            has_flag(&w, &r, &c, &[ApiArg::Str("umbrella")]).unwrap(),
            ApiValue::Bool(true)
        );
        assert_eq!(
            has_flag(&w, &r, &c, &[ApiArg::Str("nope")]).unwrap(),
            ApiValue::Bool(false)
        );
    }
}
