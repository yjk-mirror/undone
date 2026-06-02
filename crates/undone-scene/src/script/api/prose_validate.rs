//! Prose load gate — public entry point.
//!
//! The implementation lives in `script::validate` (it reuses the tokenizer, call
//! extractor, and content-id resolver that drive the condition/effect gate). This
//! module re-exports it at the path the design and the authoring-tool MCP server
//! (`minijinja-mcp-server`) consume: `script::api::prose_validate::validate_prose`.

pub use crate::script::validate::validate_prose;
