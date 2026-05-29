//! `register_read_api()` + read receiver handles (`W`, `Gd`, `M`, `F`, `Role`, `Scene`).
//!
//! Each receiver is a zero-sized handle type defined in its own submodule. The
//! method bodies read the thread-local evaluation context installed by
//! `context::ReadCtxGuard` via `context::with_read_ctx`, and are a faithful port
//! of the read methods dispatched in `undone_expr::eval` — same method names and
//! argument shapes, so existing condition strings work verbatim.
//!
//! `player` (the `W` receiver) is the worked reference; the other receivers
//! follow the identical pattern.

pub mod female_npc;
pub mod game_data;
pub mod male_npc;
pub mod player;
pub mod role;
pub mod scene;

/// Register every read receiver type and its methods onto `engine`.
///
/// Registered on BOTH the condition engine and the effect engine so effect
/// scripts can branch on reads.
pub fn register_read_api(engine: &mut rhai::Engine) {
    player::register(engine);
    game_data::register(engine);
    male_npc::register(engine);
    female_npc::register(engine);
    role::register(engine);
    scene::register(engine);
}
