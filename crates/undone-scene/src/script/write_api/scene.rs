//! `scene.*` effect mutators — scene-local flag writes (NON-persistent).
//!
//! The only non-persistent mutator group: these touch `SceneCtx::scene_flags`
//! rather than `World`, so they do not survive scene exit. Each method mirrors an
//! `apply_effect` arm in `effects.rs`, wrapped in `with_write_ctx` for
//! continue-on-error. Methods are added to the `Scene` handle defined in
//! `read_api::scene` (Rust allows inherent impls across modules in a crate).

use crate::script::context::with_write_ctx;
use crate::script::read_api::scene::Scene;

impl Scene {
    fn set_flag(&mut self, flag: &str) {
        with_write_ctx(|_world, ctx, _reg| {
            ctx.set_flag(flag.to_string());
            Ok(())
        });
    }

    fn remove_flag(&mut self, flag: &str) {
        with_write_ctx(|_world, ctx, _reg| {
            ctx.scene_flags.remove(flag);
            Ok(())
        });
    }
}

/// Register the `scene.*` effect mutators. Names are the authored effect vocabulary.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_fn("setFlag", Scene::set_flag)
        .register_fn("removeFlag", Scene::remove_flag);
}
