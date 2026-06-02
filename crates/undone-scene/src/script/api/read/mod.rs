//! Lifted read accessors — pure fns over borrowed engine state.
//!
//! Each fn here is the single body that BOTH the Rhai read receivers (via
//! `rhai_bind`) and the Minijinja prose views (via `minijinja_bind`) call, replacing
//! the former per-backend duplication. The bodies are mechanical lifts of the
//! `read_api/*` closures; value-drift decisions (notably `getName` → `effective_name`)
//! are baked in here and recorded in `table.rs`.

pub mod game_data;
pub mod npc;
pub mod player;
pub mod role;
pub mod scene;
