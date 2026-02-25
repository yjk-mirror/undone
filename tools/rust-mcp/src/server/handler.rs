use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::analyzer::RustAnalyzerClient;
use crate::server::parameters::*;
use crate::tools::execute_tool;

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

impl RustMcpServer {
    /// Dispatch a tool call to the analyzer layer and convert the result to MCP format.
    async fn dispatch(&self, tool_name: &str, args: Value) -> Result<CallToolResult, McpError> {
        let mut analyzer = self.analyzer.lock().await;
        match execute_tool(tool_name, args, &mut analyzer).await {
            Ok(result) => {
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.get("text") {
                        return Ok(CallToolResult::success(vec![Content::text(
                            text.as_str().unwrap_or("No result"),
                        )]));
                    }
                }
                Ok(CallToolResult::success(vec![Content::text("No result")]))
            }
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }
}

#[tool_router]
impl RustMcpServer {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(Mutex::new(RustAnalyzerClient::new())),
            tool_router: Self::tool_router(),
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        let mut analyzer = self.analyzer.lock().await;
        analyzer.start().await
    }

    #[tool(description = "Find the definition of a symbol at a given position")]
    async fn find_definition(
        &self,
        params: Parameters<FindDefinitionParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "find_definition",
            serde_json::json!({ "file_path": p.file_path, "line": p.line, "character": p.character }),
        )
        .await
    }

    #[tool(description = "Find all references to a symbol at a given position")]
    async fn find_references(
        &self,
        params: Parameters<FindReferencesParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "find_references",
            serde_json::json!({ "file_path": p.file_path, "line": p.line, "character": p.character }),
        )
        .await
    }

    #[tool(description = "Get compiler diagnostics for a file")]
    async fn get_diagnostics(
        &self,
        params: Parameters<GetDiagnosticsParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "get_diagnostics",
            serde_json::json!({ "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Search for symbols in the workspace")]
    async fn workspace_symbols(
        &self,
        params: Parameters<WorkspaceSymbolsParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch("workspace_symbols", serde_json::json!({ "query": p.query }))
            .await
    }

    #[tool(description = "Rename a symbol with scope awareness")]
    async fn rename_symbol(
        &self,
        params: Parameters<RenameSymbolParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "rename_symbol",
            serde_json::json!({ "file_path": p.file_path, "line": p.line, "character": p.character, "new_name": p.new_name }),
        )
        .await
    }

    #[tool(description = "Apply rustfmt formatting to a file")]
    async fn format_code(
        &self,
        params: Parameters<FormatCodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "format_code",
            serde_json::json!({ "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Parse and analyze Cargo.toml file")]
    async fn analyze_manifest(
        &self,
        params: Parameters<AnalyzeManifestParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "analyze_manifest",
            serde_json::json!({ "manifest_path": p.manifest_path }),
        )
        .await
    }

    #[tool(description = "Execute cargo check and parse errors")]
    async fn run_cargo_check(
        &self,
        params: Parameters<RunCargoCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "run_cargo_check",
            serde_json::json!({ "workspace_path": p.workspace_path }),
        )
        .await
    }

    #[tool(description = "Extract selected code into a new function")]
    async fn extract_function(
        &self,
        params: Parameters<ExtractFunctionParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "extract_function",
            serde_json::json!({
                "file_path": p.file_path,
                "start_line": p.start_line, "start_character": p.start_character,
                "end_line": p.end_line, "end_character": p.end_character,
                "function_name": p.function_name
            }),
        )
        .await
    }

    #[tool(description = "Generate a struct with specified fields and derives")]
    async fn generate_struct(
        &self,
        params: Parameters<GenerateStructParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "generate_struct",
            serde_json::json!({ "struct_name": p.struct_name, "fields": p.fields, "derives": p.derives, "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Generate an enum with specified variants and derives")]
    async fn generate_enum(
        &self,
        params: Parameters<GenerateEnumParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "generate_enum",
            serde_json::json!({ "enum_name": p.enum_name, "variants": p.variants, "derives": p.derives, "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Generate a trait implementation for a struct")]
    async fn generate_trait_impl(
        &self,
        params: Parameters<GenerateTraitImplParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "generate_trait_impl",
            serde_json::json!({ "trait_name": p.trait_name, "struct_name": p.struct_name, "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Generate unit tests for a function")]
    async fn generate_tests(
        &self,
        params: Parameters<GenerateTestsParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "generate_tests",
            serde_json::json!({ "target_function": p.target_function, "file_path": p.file_path, "test_cases": p.test_cases }),
        )
        .await
    }

    #[tool(description = "Inline a function call at specified position")]
    async fn inline_function(
        &self,
        params: Parameters<InlineFunctionParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "inline_function",
            serde_json::json!({ "file_path": p.file_path, "line": p.line, "character": p.character }),
        )
        .await
    }

    #[tool(description = "Change the signature of a function")]
    async fn change_signature(
        &self,
        params: Parameters<ChangeSignatureParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "change_signature",
            serde_json::json!({ "file_path": p.file_path, "line": p.line, "character": p.character, "new_signature": p.new_signature }),
        )
        .await
    }

    #[tool(description = "Organize and sort import statements in a file")]
    async fn organize_imports(
        &self,
        params: Parameters<OrganizeImportsParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "organize_imports",
            serde_json::json!({ "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Apply clippy lint suggestions to improve code quality")]
    async fn apply_clippy_suggestions(
        &self,
        params: Parameters<ApplyClippySuggestionsParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "apply_clippy_suggestions",
            serde_json::json!({ "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Validate and suggest lifetime annotations")]
    async fn validate_lifetimes(
        &self,
        params: Parameters<ValidateLifetimesParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "validate_lifetimes",
            serde_json::json!({ "file_path": p.file_path }),
        )
        .await
    }

    #[tool(description = "Get type hierarchy for a symbol at specified position")]
    async fn get_type_hierarchy(
        &self,
        params: Parameters<GetTypeHierarchyParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "get_type_hierarchy",
            serde_json::json!({ "file_path": p.file_path, "line": p.line, "character": p.character }),
        )
        .await
    }

    #[tool(description = "Suggest crate dependencies based on code patterns")]
    async fn suggest_dependencies(
        &self,
        params: Parameters<SuggestDependenciesParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "suggest_dependencies",
            serde_json::json!({ "query": p.query, "workspace_path": p.workspace_path }),
        )
        .await
    }

    #[tool(description = "Create a new Rust module with optional visibility")]
    async fn create_module(
        &self,
        params: Parameters<CreateModuleParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "create_module",
            serde_json::json!({ "module_name": p.module_name, "module_path": p.module_path, "is_public": p.is_public }),
        )
        .await
    }

    #[tool(description = "Move code items from one file to another")]
    async fn move_items(
        &self,
        params: Parameters<MoveItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = &params.0;
        self.dispatch(
            "move_items",
            serde_json::json!({ "source_file": p.source_file, "target_file": p.target_file, "item_names": p.item_names }),
        )
        .await
    }
}

#[tool_handler]
impl ServerHandler for RustMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Rust MCP Server providing rust-analyzer integration for idiomatic Rust development tools.".into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
