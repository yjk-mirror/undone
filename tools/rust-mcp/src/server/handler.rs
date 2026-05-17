use anyhow::Result;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::analyzer::RustAnalyzerClient;
use crate::server::parameters::*;

#[derive(Clone)]
pub struct RustMcpServer {
    analyzer: Arc<Mutex<RustAnalyzerClient>>,
    tool_router: ToolRouter<RustMcpServer>,
}

impl Default for RustMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

fn ok(text: String) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text)])
}

fn err(e: impl std::fmt::Display) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

#[tool_router]
impl RustMcpServer {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(Mutex::new(RustAnalyzerClient::new())),
            tool_router: Self::tool_router(),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut analyzer = self.analyzer.lock().await;
        analyzer.start().await
    }

    #[tool(description = "Find the definition of a symbol at a given position")]
    async fn find_definition(
        &self,
        params: Parameters<FindDefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let mut analyzer = self.analyzer.lock().await;
        analyzer
            .find_definition(&p.file_path, p.line, p.character)
            .await
            .map(ok)
            .map_err(err)
    }

    #[tool(description = "Find all references to a symbol at a given position")]
    async fn find_references(
        &self,
        params: Parameters<FindReferencesParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let mut analyzer = self.analyzer.lock().await;
        analyzer
            .find_references(&p.file_path, p.line, p.character)
            .await
            .map(ok)
            .map_err(err)
    }

    #[tool(description = "Search for symbols in the workspace")]
    async fn workspace_symbols(
        &self,
        params: Parameters<WorkspaceSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let mut analyzer = self.analyzer.lock().await;
        analyzer
            .workspace_symbols(&p.query)
            .await
            .map(ok)
            .map_err(err)
    }

    #[tool(description = "Rename a symbol with scope awareness")]
    async fn rename_symbol(
        &self,
        params: Parameters<RenameSymbolParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let mut analyzer = self.analyzer.lock().await;
        analyzer
            .rename_symbol(&p.file_path, p.line, p.character, &p.new_name)
            .await
            .map(ok)
            .map_err(err)
    }
}

#[tool_handler]
impl ServerHandler for RustMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Rust MCP Server: rust-analyzer-backed navigation tools (find_definition, find_references, workspace_symbols, rename_symbol).".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
