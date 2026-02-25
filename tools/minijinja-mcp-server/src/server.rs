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

#[derive(Clone)]
pub struct MiniJinjaServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl MiniJinjaServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
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
            instructions: Some("Minijinja template validation and preview tools.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
