//! The Rhai scripting layer.
//!
//! Conditions and effects authored in pack TOML are compiled to `Arc<rhai::AST>`
//! once at pack load (the direct analog of the pre-parsed `undone_expr::Expr` they
//! replace) and evaluated many times at runtime. Game state is exposed to scripts
//! ONLY through a curated set of registered receiver methods (`w`, `gd`, `m`, `f`,
//! `role`, `scene` for reads; `w.*`/`npc(x).*`/`scene.*` mutators for effects) —
//! never by exposing `World` directly.

pub mod compiled;
pub mod context;
pub mod engine;
pub mod read_api;
pub mod validate;
pub mod write_api;

pub use compiled::{compile_condition, compile_effect, CompiledScript, ScriptError};
pub use engine::{build_engines, ScriptEngines};
