use rhai::Engine;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;

use crate::validator;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SourceInput {
    /// The Rhai source code to check.
    pub source: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FilePathInput {
    /// Absolute path to the .rhai file.
    pub file_path: String,
}

#[derive(Clone)]
pub struct RhaiServer {
    tool_router: ToolRouter<Self>,
    /// Probe registry loaded from the base pack, so condition/effect validation
    /// resolves the same content ids the game loader does. Empty if the pack
    /// can't be found (validation then degrades to syntax + method checks).
    registry: Arc<undone_packs::PackRegistry>,
}

/// A bounded validation engine, built fresh per call. NOT stored on the handler:
/// the handler must stay Send+Sync for rmcp, but rhai is built here without the
/// `sync` feature (see Cargo.toml), so an `Engine` is not Send+Sync. Construction
/// is cheap and these are infrequent authoring-time calls.
fn validation_engine() -> Engine {
    let mut engine = Engine::new();
    // Prevent infinite-loop DoS on rhai_get_diagnostics — generous but bounded.
    engine.set_max_operations(500_000);
    engine
}

#[tool_router]
impl RhaiServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            registry: Arc::new(load_probe_registry()),
        }
    }

    #[tool(
        description = "Fast syntax-only check of a Rhai source string. Returns a JSON array of diagnostics with line/column. Empty array means valid. Does not evaluate the script."
    )]
    async fn rhai_check_syntax(
        &self,
        params: Parameters<SourceInput>,
    ) -> Result<CallToolResult, McpError> {
        let diags = validator::check_syntax(&params.0.source);
        let json = serde_json::to_string_pretty(&diags)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Validate a Rhai script file at the given absolute path. Returns a JSON array of diagnostics with line/column. Empty array means valid. Use this after writing or editing a .rhai file."
    )]
    async fn rhai_validate_script(
        &self,
        params: Parameters<FilePathInput>,
    ) -> Result<CallToolResult, McpError> {
        let path = std::path::Path::new(&params.0.file_path);
        let diags = validator::validate_file(path);
        let json = serde_json::to_string_pretty(&diags)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Run a Rhai source string through the engine to catch runtime errors that syntax-checking misses (e.g. undefined variable references, type mismatches). Returns a JSON array of diagnostics."
    )]
    async fn rhai_get_diagnostics(
        &self,
        params: Parameters<SourceInput>,
    ) -> Result<CallToolResult, McpError> {
        let diags = validator::validate_with_engine(&params.0.source, &validation_engine());
        let json = serde_json::to_string_pretty(&diags)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Validate a game CONDITION string against the real engine + base-pack registry — exactly the load-time gate the game loader runs. Catches syntax errors, unknown methods, unknown content ids (e.g. a typo'd trait/skill), and effect mutators used in a condition. Returns a JSON array of diagnostics; empty means valid."
    )]
    async fn rhai_validate_condition(
        &self,
        params: Parameters<SourceInput>,
    ) -> Result<CallToolResult, McpError> {
        let diags = validator::validate_game_condition(&params.0.source, &self.registry);
        let json = serde_json::to_string_pretty(&diags)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Validate a game EFFECT call-list string against the real engine + base-pack registry — the same load-time gate the game loader runs. Catches syntax errors, unknown mutators, unknown content ids, and out-of-range step deltas. Returns a JSON array of diagnostics; empty means valid."
    )]
    async fn rhai_validate_effect(
        &self,
        params: Parameters<SourceInput>,
    ) -> Result<CallToolResult, McpError> {
        let diags = validator::validate_game_effect(&params.0.source, &self.registry);
        let json = serde_json::to_string_pretty(&diags)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "List all functions registered in the Rhai engine, including built-ins. Returns a JSON array of function signatures. Use this to check what functions are available before calling them in a script."
    )]
    async fn rhai_list_registered_api(&self) -> Result<CallToolResult, McpError> {
        let fns = validator::list_registered_functions(&validation_engine());
        let json = serde_json::to_string_pretty(&fns)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

/// Load the base-pack registry for content-id validation. Tries `$UNDONE_PACKS_DIR`
/// then `./packs` (the server is launched from the repo root). On any failure,
/// returns an empty registry so the server still starts (id validation degrades).
fn load_probe_registry() -> undone_packs::PackRegistry {
    let candidate = std::env::var("UNDONE_PACKS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("packs"));
    match undone_packs::load_packs(&candidate) {
        Ok((registry, _metas)) => registry,
        Err(e) => {
            tracing::warn!(
                "rhai-mcp-server: could not load probe pack from {}: {e}; \
                 content-id validation will be limited",
                candidate.display()
            );
            undone_packs::PackRegistry::new()
        }
    }
}

#[tool_handler]
impl ServerHandler for RhaiServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Rhai script validation and API introspection tools.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
