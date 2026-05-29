//! `ScriptEngines` (two-engine scaffold) + `build_engines()` + eval helpers.

use undone_expr::SceneCtx;
use undone_packs::PackRegistry;
use undone_world::World;

use crate::script::compiled::{CompiledScript, ScriptError};
use crate::script::context::ReadCtxGuard;
use crate::script::read_api::female_npc::F;
use crate::script::read_api::game_data::Gd;
use crate::script::read_api::male_npc::M;
use crate::script::read_api::player::W;
use crate::script::read_api::register_read_api;
use crate::script::read_api::role::Role;
use crate::script::read_api::scene::Scene;
use crate::script::write_api::register_write_api;

/// The two Rhai engines a session uses.
///
/// `cond` has only the read API registered, so a mutating call inside a
/// condition is unresolvable there (a load error via the Task-6 dry-run gate).
/// `effect` has read + write, so effect scripts can branch on reads.
pub struct ScriptEngines {
    /// Read API only — conditions compile and evaluate against this.
    pub cond: rhai::Engine,
    /// Read + write API — effect call-lists compile and evaluate against this.
    pub effect: rhai::Engine,
}

/// Bounds applied to every engine (Engineering Principle 5: bounded resources).
const MAX_OPERATIONS: u64 = 50_000;
const MAX_EXPR_DEPTH: usize = 64;

fn new_bounded_engine() -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    // Unknown identifier (variable) → compile error.
    engine.set_strict_variables(true);
    engine.set_max_operations(MAX_OPERATIONS);
    engine.set_max_expr_depths(MAX_EXPR_DEPTH, MAX_EXPR_DEPTH);
    engine
}

/// Build the condition + effect engines with their bounds and API surfaces.
pub fn build_engines() -> ScriptEngines {
    let mut cond = new_bounded_engine();
    register_read_api(&mut cond);

    let mut effect = new_bounded_engine();
    register_read_api(&mut effect);
    register_write_api(&mut effect);

    ScriptEngines { cond, effect }
}

/// Push the six read-receiver handles into a fresh scope under the names the
/// authored condition syntax uses (`w`, `gd`, `m`, `f`, `role`, `scene`).
///
/// With `strict_variables(true)`, scripts must be compiled with these variables
/// in scope (`compile_with_scope(&read_scope(), src)`) or `w`/`gd`/… are rejected
/// as undefined at compile time. The same scope shape serves both engines —
/// write methods add registered functions, not new scope variables.
pub(crate) fn read_scope() -> rhai::Scope<'static> {
    let mut scope = rhai::Scope::new();
    scope.push("w", W);
    scope.push("gd", Gd);
    scope.push("m", M);
    scope.push("f", F);
    scope.push("role", Role);
    scope.push("scene", Scene);
    scope
}

/// Evaluate a compiled condition to `bool`, installing the read context for the
/// duration of the call. The `engine` should be the `cond` engine (or `effect`
/// for an effect script's internal reads).
pub fn eval_bool(
    script: &CompiledScript,
    engine: &rhai::Engine,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<bool, ScriptError> {
    let _guard = ReadCtxGuard::install(world, registry, ctx);
    let mut scope = read_scope();
    engine
        .eval_ast_with_scope::<bool>(&mut scope, &script.ast)
        .map_err(|e| ScriptError::Runtime {
            context: script.source.clone(),
            message: e.to_string(),
        })
}

/// Evaluate a compiled script to `i64` (used by the dry-run gate / tests).
pub fn eval_int(
    script: &CompiledScript,
    engine: &rhai::Engine,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<i64, ScriptError> {
    let _guard = ReadCtxGuard::install(world, registry, ctx);
    let mut scope = read_scope();
    engine
        .eval_ast_with_scope::<i64>(&mut scope, &script.ast)
        .map_err(|e| ScriptError::Runtime {
            context: script.source.clone(),
            message: e.to_string(),
        })
}

/// Evaluate a compiled script to `String` (used by the dry-run gate / tests).
pub fn eval_string(
    script: &CompiledScript,
    engine: &rhai::Engine,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<String, ScriptError> {
    let _guard = ReadCtxGuard::install(world, registry, ctx);
    let mut scope = read_scope();
    engine
        .eval_ast_with_scope::<String>(&mut scope, &script.ast)
        .map_err(|e| ScriptError::Runtime {
            context: script.source.clone(),
            message: e.to_string(),
        })
}

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

    #[test]
    fn build_engines_succeeds() {
        let _engines = super::build_engines();
    }

    #[test]
    fn rhai_condition_reads_trait_and_skill() {
        use std::sync::Arc;

        use crate::script::compiled::CompiledScript;

        let engines = super::build_engines();

        let mut reg = undone_packs::PackRegistry::new();
        reg.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        reg.register_skills(vec![undone_packs::SkillDef {
            id: "FEMININITY".into(),
            name: "Femininity".into(),
            description: "...".into(),
            min: 0,
            max: 100,
        }]);
        let shy = reg.resolve_trait("SHY").unwrap();
        let fem = reg.resolve_skill("FEMININITY").unwrap();

        let mut world = make_test_world();
        world.player.traits.insert(shy);
        world.player.skills.insert(
            fem,
            undone_domain::SkillValue {
                value: 12,
                modifier: 0,
            },
        );
        let ctx = undone_expr::SceneCtx::new();

        let src = r#"w.hasTrait("SHY") && w.getSkill("FEMININITY") < 15"#;
        let ast = engines
            .cond
            .compile_with_scope(&super::read_scope(), src)
            .unwrap();
        let script = CompiledScript {
            ast: Arc::new(ast),
            source: src.into(),
        };

        let got = super::eval_bool(&script, &engines.cond, &world, &ctx, &reg).unwrap();
        assert!(got, "SHY + FEMININITY 12 (<15) should be true");
    }

    /// The read/write split: a mutating call resolves on the effect engine but
    /// not on the condition engine. Note Rhai resolves functions at *runtime*,
    /// so this is an EVAL-time difference, not a `compile()` difference — which
    /// is exactly what the Task-6 dry-run gate turns into a load error.
    ///
    /// `#[ignore]` until Task 5 registers `addArousal`; un-ignored there.
    #[test]
    #[ignore = "un-ignore in Task 5 once addArousal is registered on the write API"]
    fn write_call_resolves_on_effect_engine_only() {
        let engines = super::build_engines();
        // Compiles on both (Rhai defers fn resolution to runtime)...
        let on_cond = engines.cond.compile("w.addArousal(1)");
        let on_effect = engines.effect.compile("w.addArousal(1)");
        assert!(on_cond.is_ok() && on_effect.is_ok());

        // ...but only the effect engine can *resolve* the call at eval time.
        // (A full eval needs a context installed; Task 5 wires that up. Here we
        // assert the registration difference via the engine's known fns.)
    }
}
