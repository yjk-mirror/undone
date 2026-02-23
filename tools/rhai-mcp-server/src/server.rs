use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    Error as McpError,
};
use rhai::Engine;
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
    engine: Arc<Engine>,
}

#[tool_router]
impl RhaiServer {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        // Prevent infinite-loop DoS on rhai_get_diagnostics — generous but bounded.
        engine.set_max_operations(500_000);
        Self {
            tool_router: Self::tool_router(),
            engine: Arc::new(engine),
        }
    }

    #[tool(description = "Fast syntax-only check of a Rhai source string. Returns a JSON array of diagnostics with line/column. Empty array means valid. Does not evaluate the script.")]
    async fn rhai_check_syntax(
        &self,
        params: Parameters<SourceInput>,
    ) -> Result<CallToolResult, McpError> {
        let diags = validator::check_syntax(&params.0.source);
        let json = serde_json::to_string_pretty(&diags)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "Validate a Rhai script file at the given absolute path. Returns a JSON array of diagnostics with line/column. Empty array means valid. Use this after writing or editing a .rhai file.")]
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

    #[tool(description = "Run a Rhai source string through the engine to catch runtime errors that syntax-checking misses (e.g. undefined variable references, type mismatches). Returns a JSON array of diagnostics.")]
    async fn rhai_get_diagnostics(
        &self,
        params: Parameters<SourceInput>,
    ) -> Result<CallToolResult, McpError> {
        let diags = validator::validate_with_engine(&params.0.source, &self.engine);
        let json = serde_json::to_string_pretty(&diags)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "List all functions registered in the Rhai engine, including built-ins. Returns a JSON array of function signatures. Use this to check what functions are available before calling them in a script.")]
    async fn rhai_list_registered_api(
        &self,
    ) -> Result<CallToolResult, McpError> {
        let fns = validator::list_registered_functions(&self.engine);
        let json = serde_json::to_string_pretty(&fns)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
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
