use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::validator;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateTemplateInput {
    /// The Minijinja/Jinja2 template source to validate.
    pub source: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenderTemplateInput {
    /// The Minijinja/Jinja2 template source to render.
    pub source: String,
    /// JSON object to use as the template context. Example: {"name": "Alice"}
    pub context_json: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateProseInput {
    /// The Minijinja prose template source to validate against the game method surface.
    pub source: String,
}

#[derive(Clone)]
pub struct MiniJinjaServer {
    tool_router: ToolRouter<Self>,
    /// Base-pack registry, for content-id resolution in the prose method-surface gate.
    registry: Arc<undone_packs::PackRegistry>,
}

#[tool_router]
impl MiniJinjaServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            registry: Arc::new(load_probe_registry()),
        }
    }

    #[tool(
        description = "Validate a game PROSE template against the real method surface — exactly the load-time prose gate the game loader runs. Catches minijinja syntax errors AND unknown/mis-contexted methods (a write mutator or condition-only checkSkill used in prose) and unknown content ids (a typo'd trait/skill). Single- and double-quoted ids are both accepted. Returns a JSON array of diagnostics; empty means valid."
    )]
    async fn jinja_validate_prose(
        &self,
        params: Parameters<ValidateProseInput>,
    ) -> Result<CallToolResult, McpError> {
        let errors = validator::validate_prose(&params.0.source, &self.registry);
        let json = serde_json::to_string_pretty(&errors)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Parse a Minijinja/Jinja2 template and return syntax errors. Returns an empty array if valid. Use this after writing or editing a .j2 or .jinja file."
    )]
    async fn jinja_validate_template(
        &self,
        params: Parameters<ValidateTemplateInput>,
    ) -> Result<CallToolResult, McpError> {
        let errors = validator::validate_template(&params.0.source);
        let json = serde_json::to_string_pretty(&errors)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Render a Minijinja template with a JSON context object. Returns the rendered string, or an error with line number if rendering fails. Useful for testing templates against example data."
    )]
    async fn jinja_render_preview(
        &self,
        params: Parameters<RenderTemplateInput>,
    ) -> Result<CallToolResult, McpError> {
        match validator::render_template(&params.0.source, &params.0.context_json) {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => {
                let json = serde_json::to_string_pretty(&e)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                Ok(CallToolResult::error(vec![Content::text(json)]))
            }
        }
    }

    #[tool(
        description = "List all built-in Minijinja filters available in templates. Returns a JSON array of filter names."
    )]
    async fn jinja_list_filters(&self) -> Result<CallToolResult, McpError> {
        let filters = validator::list_builtin_filters();
        let json = serde_json::to_string_pretty(&filters)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[tool_handler]
impl ServerHandler for MiniJinjaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Minijinja template validation, preview, and prose method-surface (game \
                 prose gate) tools."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Load the base-pack registry for prose content-id validation. Tries
/// `$UNDONE_PACKS_DIR` then `./packs` (the server is launched from the repo root).
/// On any failure, returns an empty registry so the server still starts (id
/// validation degrades, method/context checks still work).
fn load_probe_registry() -> undone_packs::PackRegistry {
    let candidate = std::env::var("UNDONE_PACKS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("packs"));
    match undone_packs::load_packs(&candidate) {
        Ok((registry, _metas)) => registry,
        Err(e) => {
            tracing::warn!(
                "minijinja-mcp-server: could not load probe pack from {}: {e}; \
                 prose content-id validation will be limited",
                candidate.display()
            );
            undone_packs::PackRegistry::new()
        }
    }
}
