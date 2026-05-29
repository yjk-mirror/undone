//! The `scene` receiver — scene-local flag reads.
//!
//! Mirrors the `Receiver::Scene` arm in `undone_expr::eval` (`eval_call_bool`),
//! preserving the exact method name and argument shape so existing condition
//! strings work verbatim.

use crate::script::context::with_read_ctx;

type RhaiResult<T> = Result<T, Box<rhai::EvalAltResult>>;

/// Zero-sized `scene` handle; reads the thread-local evaluation context.
#[derive(Clone)]
pub struct Scene;

impl Scene {
    // ── bool methods (eval_call_bool, Receiver::Scene) ───────────────────────

    fn has_flag(&mut self, flag: &str) -> RhaiResult<bool> {
        with_read_ctx(|_world, _reg, ctx| Ok(ctx.has_flag(flag)))
    }
}

/// Register the `Scene` type and its methods. Names match the authored condition
/// syntax (`scene.hasFlag(...)`) exactly.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<Scene>()
        // bool
        .register_fn("hasFlag", Scene::has_flag);
}
