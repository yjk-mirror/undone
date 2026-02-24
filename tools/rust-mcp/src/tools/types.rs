use anyhow::Result;
use serde_json::Value;

use crate::analyzer::RustAnalyzerClient;

pub struct ToolResult {
    pub content: Vec<serde_json::Map<String, Value>>,
}

pub async fn execute_tool(
    name: &str,
    args: Value,
    analyzer: &mut RustAnalyzerClient,
) -> Result<ToolResult> {
    match name {
        "find_definition" => crate::tools::analysis::find_definition_impl(args, analyzer).await,
        "find_references" => crate::tools::analysis::find_references_impl(args, analyzer).await,
        "get_diagnostics" => crate::tools::analysis::get_diagnostics_impl(args, analyzer).await,
        "workspace_symbols" => {
            crate::tools::navigation::workspace_symbols_impl(args, analyzer).await
        }
        "rename_symbol" => crate::tools::refactoring::rename_symbol_impl(args, analyzer).await,
        "extract_function" => {
            crate::tools::refactoring::extract_function_impl(args, analyzer).await
        }
        "format_code" => crate::tools::formatting::format_code_impl(args, analyzer).await,
        "analyze_manifest" => crate::tools::cargo::analyze_manifest_impl(args, analyzer).await,
        "run_cargo_check" => crate::tools::cargo::run_cargo_check_impl(args, analyzer).await,
        "generate_struct" => crate::tools::generation::generate_struct_impl(args, analyzer).await,
        "generate_enum" => crate::tools::generation::generate_enum_impl(args, analyzer).await,
        "generate_trait_impl" => {
            crate::tools::generation::generate_trait_impl_impl(args, analyzer).await
        }
        "generate_tests" => crate::tools::generation::generate_tests_impl(args, analyzer).await,
        "inline_function" => crate::tools::refactoring::inline_function_impl(args, analyzer).await,
        "change_signature" => {
            crate::tools::refactoring::change_signature_impl(args, analyzer).await
        }
        "organize_imports" => {
            crate::tools::refactoring::organize_imports_impl(args, analyzer).await
        }
        "apply_clippy_suggestions" => {
            crate::tools::quality::apply_clippy_suggestions_impl(args, analyzer).await
        }
        "validate_lifetimes" => {
            crate::tools::quality::validate_lifetimes_impl(args, analyzer).await
        }
        "get_type_hierarchy" => {
            crate::tools::advanced::get_type_hierarchy_impl(args, analyzer).await
        }
        "suggest_dependencies" => {
            crate::tools::advanced::suggest_dependencies_impl(args, analyzer).await
        }
        "create_module" => crate::tools::advanced::create_module_impl(args, analyzer).await,
        "move_items" => crate::tools::advanced::move_items_impl(args, analyzer).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    }
}
