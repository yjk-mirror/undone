//! Compiled-script type + the load-time error taxonomy.
//!
//! The load-time compile/validate gate (`compile_condition` / `compile_effect`)
//! is added in Task 6.

use std::sync::Arc;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("script compile error in {context}: {message}\n  source: {source_text}")]
    Compile {
        context: String,
        message: String,
        source_text: String,
    },
    #[error("unknown content id '{id}' ({kind}) in {context}\n  source: {source_text}")]
    UnknownId {
        context: String,
        kind: String,
        id: String,
        source_text: String,
    },
    #[error("script runtime error in {context}: {message}")]
    Runtime { context: String, message: String },
}

/// A compiled condition or effect script. The AST is the direct analog of the
/// pre-parsed `undone_expr::Expr` it replaces: compiled once at pack load,
/// evaluated many times at runtime.
#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub ast: Arc<rhai::AST>,
    pub source: String,
}
