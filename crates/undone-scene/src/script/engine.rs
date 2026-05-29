//! `ScriptEngines` (two-engine scaffold) + `build_engines()` + eval helpers.
//!
//! The scaffold (`ScriptEngines`, `build_engines`) lands in Task 3 and the eval
//! helpers (`eval_bool` / `eval_int` / `eval_string`) in Task 4. For now this
//! file hosts the Task-2 borrow-bridging SPIKE bench that justified the decision
//! documented in `context.rs`.

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::script::context::{with_read_ctx, GdA, ReadCtxGuard};
    use undone_world::test_helpers::make_test_world;

    const ITERS: usize = 10_000;

    /// Candidate B receiver: zero-sized; reads the thread-local context.
    #[derive(Clone)]
    struct GdB;

    impl GdB {
        fn week(&mut self) -> Result<i64, Box<rhai::EvalAltResult>> {
            with_read_ctx(|world, _reg, _ctx| Ok(world.game_data.week as i64))
        }
    }

    #[test]
    fn spike_binding() {
        let mut world = make_test_world();
        world.game_data.week = 1;
        let registry = undone_packs::PackRegistry::new();
        let ctx = undone_expr::SceneCtx::new();

        // ── Candidate A: pointer carried inside the scope-injected handle ──
        let mut engine_a = rhai::Engine::new();
        engine_a
            .register_type::<GdA>()
            .register_fn("week", GdA::week);
        let ast_a = engine_a.compile("gd.week() == 1").unwrap();

        let start_a = Instant::now();
        for _ in 0..ITERS {
            let mut scope = rhai::Scope::new();
            scope.push(
                "gd",
                GdA {
                    world: &world as *const _,
                },
            );
            let got: bool = engine_a
                .eval_ast_with_scope(&mut scope, &ast_a)
                .expect("candidate A eval");
            assert!(got);
        }
        let elapsed_a = start_a.elapsed();

        // ── Candidate B: ZST handle + thread-local context (chosen path) ──
        let mut engine_b = rhai::Engine::new();
        engine_b
            .register_type::<GdB>()
            .register_fn("week", GdB::week);
        let ast_b = engine_b.compile("gd.week() == 1").unwrap();

        let start_b = Instant::now();
        for _ in 0..ITERS {
            let _guard = ReadCtxGuard::install(&world, &registry, &ctx);
            let mut scope = rhai::Scope::new();
            scope.push("gd", GdB);
            let got: bool = engine_b
                .eval_ast_with_scope(&mut scope, &ast_b)
                .expect("candidate B eval");
            assert!(got);
        }
        let elapsed_b = start_b.elapsed();

        println!(
            "spike_binding ({ITERS} evals): candidate A (handle ptr) = {elapsed_a:?}, \
             candidate B (thread-local) = {elapsed_b:?}"
        );
        // Both must be correct; the decision (B) is recorded in context.rs and is
        // driven by ergonomics since the two are within noise on this bench.
    }
}
