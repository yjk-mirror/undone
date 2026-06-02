//! Lifted write (effect mutator) accessors — fns over `&mut World` / `&mut SceneCtx`.
//!
//! Bodies are mechanical lifts of the `write_api/*` `with_write_ctx` closures; the
//! thread-local plumbing and continue-on-error sink now live in the Rhai adapter.
//! Each fn returns `Result<(), EffectError>`, preserving the exact error vocabulary.

pub mod game_data;
pub mod npc;
pub mod player;
pub mod scene;
