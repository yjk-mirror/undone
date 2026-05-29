//! `register_write_api()` + effect mutator calls.
//!
//! Registered ONLY on the effect engine, so a mutating call inside a condition
//! is not resolvable on the condition engine (surfaced as a load error by the
//! Task-6 gate). Each mutator wraps its body in `context::with_write_ctx`, which
//! preserves the engine's best-effort *continue-on-error* semantics: a failing
//! mutator records an error and the call-list keeps running.
//!
//! The authored effect vocabulary (one statement per call, no control flow):
//! - `w.*`     player + player-relationship writes
//! - `gd.*`    game-data writes (flags, stats, arc, time, job, red-check)
//! - `scene.*` scene-local flag writes (the only NON-persistent mutators)
//! - `npc("m"|"f"|role).*` NPC writes
//!
//! Method bodies are a faithful port of the `apply_effect` arms in `effects.rs`,
//! reusing its `step_*` / `parse_*` / `resolve_npc_ref` helpers. `player` is the
//! worked reference; the other groups follow the identical pattern.

pub mod game_data;
pub mod npc;
pub mod player;
pub mod scene;

/// Register every effect mutator onto the effect `engine`.
pub fn register_write_api(engine: &mut rhai::Engine) {
    player::register(engine);
    game_data::register(engine);
    scene::register(engine);
    npc::register(engine);
}
