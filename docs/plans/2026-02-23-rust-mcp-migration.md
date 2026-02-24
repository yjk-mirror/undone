# Rust MCP Migration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate the `rust-mcp` server from `~/.claude/mcp-servers/rust-mcp/` into `tools/rust-mcp/` so it builds cross-platform as part of the tools workspace.

**Context:** Source has been copied to `tools/rust-mcp/src/` and `Cargo.toml` + workspace membership are already set up. The code needs porting from rmcp 0.2.1 to rmcp 0.8 (matching the other 4 MCP servers in the workspace).

**Architecture:** The rust-mcp is a thin wrapper around a long-lived `rust-analyzer` LSP subprocess. It exposes 20 MCP tools (find_definition, rename_symbol, etc.) via rmcp. The code is pure Rust with no platform-specific dependencies — `get_rust_analyzer_path()` already handles HOME vs USERPROFILE and .exe detection.

---

### API Differences: rmcp 0.2 → 0.8

These are the **only** breaking changes. Everything else compiles as-is.

| What | rmcp 0.2 (old) | rmcp 0.8 (new) |
|------|----------------|-----------------|
| Parameters import | `handler::server::tool::Parameters` | `handler::server::wrapper::Parameters` |
| Error type | `model::ErrorData as McpError` | `Error as McpError` |
| ServerInfo | explicit `protocol_version`, `server_info: Implementation::from_build_env()` | `..Default::default()` |
| Tool param style | `Parameters(MyParams { field1, field2 })` destructuring | `params: Parameters<MyParams>` then `params.0.field1` |
| Tool return error | `Result<CallToolResult, McpError>` with `McpError` = `ErrorData` | `Result<CallToolResult, McpError>` with `McpError` = `rmcp::Error` |

**Reference implementation:** `tools/rhai-mcp-server/src/server.rs` — copy this pattern exactly.

---

### Task 1: Fix handler.rs imports and ServerHandler impl

**File:** `tools/rust-mcp/src/server/handler.rs`

**Step 1:** Replace the import block (lines 1-10):

Old:
```rust
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ErrorData as McpError, *},
    tool, tool_handler, tool_router,
};
```

New:
```rust
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    Error as McpError,
};
```

**Step 2:** Replace the `ServerHandler` impl (bottom of file):

Old:
```rust
#[tool_handler]
impl ServerHandler for RustMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Rust MCP Server providing rust-analyzer integration...".to_string()),
        }
    }
}
```

New:
```rust
#[tool_handler]
impl ServerHandler for RustMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Rust MCP Server providing rust-analyzer integration for idiomatic Rust development tools.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
```

---

### Task 2: Fix all 20 tool method signatures in handler.rs

Every `#[tool]` method needs its parameter pattern changed. The pattern is mechanical:

Old style (every method):
```rust
#[tool(description = "...")]
async fn find_definition(
    &self,
    Parameters(FindDefinitionParams {
        file_path,
        line,
        character,
    }): Parameters<FindDefinitionParams>,
) -> Result<CallToolResult, McpError> {
    let args = serde_json::json!({
        "file_path": file_path,
        "line": line,
        "character": character
    });
    ...
}
```

New style:
```rust
#[tool(description = "...")]
async fn find_definition(
    &self,
    params: Parameters<FindDefinitionParams>,
) -> Result<CallToolResult, McpError> {
    let p = &params.0;
    let args = serde_json::json!({
        "file_path": p.file_path,
        "line": p.line,
        "character": p.character
    });
    ...
}
```

**All 20 methods to update** (same mechanical transformation for each):
1. `find_definition` — fields: file_path, line, character
2. `find_references` — fields: file_path, line, character
3. `get_diagnostics` — fields: file_path
4. `workspace_symbols` — fields: query
5. `rename_symbol` — fields: file_path, line, character, new_name
6. `format_code` — fields: file_path
7. `analyze_manifest` — fields: manifest_path
8. `run_cargo_check` — fields: workspace_path
9. `extract_function` — fields: file_path, start_line, start_character, end_line, end_character, function_name
10. `generate_struct` — fields: struct_name, fields, derives, file_path
11. `generate_enum` — fields: enum_name, variants, derives, file_path
12. `generate_trait_impl` — fields: trait_name, struct_name, file_path
13. `generate_tests` — fields: target_function, file_path, test_cases
14. `inline_function` — fields: file_path, line, character
15. `change_signature` — fields: file_path, line, character, new_signature
16. `organize_imports` — fields: file_path
17. `apply_clippy_suggestions` — fields: file_path
18. `validate_lifetimes` — fields: file_path
19. `get_type_hierarchy` — fields: file_path, line, character
20. `suggest_dependencies` — fields: query, workspace_path
21. `create_module` — fields: module_name, module_path, is_public
22. `move_items` — fields: source_file, target_file, item_names

**After editing:** `cd tools && cargo fmt -p rust-mcp && cargo check -p rust-mcp 2>&1 | tail -20`

---

### Task 3: Fix main.rs crate name

**File:** `tools/rust-mcp/src/main.rs`

The package was renamed from `rustmcp` to `rust-mcp`. Cargo converts hyphens to underscores for crate names, so the import must change:

Old:
```rust
use rustmcp::server::RustMcpServer;
```

New:
```rust
use rust_mcp::server::RustMcpServer;
```

---

### Task 4: Fix parameters.rs — clean up derives

**File:** `tools/rust-mcp/src/server/parameters.rs`

The rmcp 0.8 `#[tool]` macro requires parameter types to derive `JsonSchema`. Currently they only import `schemars` via rmcp's re-export. Change the import and derives:

Old:
```rust
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindDefinitionParams { ... }
```

New:
```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindDefinitionParams { ... }
```

Apply to ALL param structs in the file. This is because rmcp 0.8 re-exports schemars differently — using the crate directly (which is already in Cargo.toml deps) is more reliable.

**After editing:** `cd tools && cargo check -p rust-mcp 2>&1 | tail -20`

---

### Task 5: Build release binary and verify

```bash
cd tools && cargo build --release -p rust-mcp 2>&1 | tail -10
```

Expected: clean build, binary at `tools/target/release/rust-mcp.exe` (Windows) or `tools/target/release/rust-mcp` (Linux).

Verify the binary starts:
```bash
echo '{}' | timeout 3 tools/target/release/rust-mcp 2>&1 || true
```
(It should print stderr about starting, then timeout — that's correct for an MCP server expecting JSON-RPC.)

---

### Task 6: Add to .mcp.json

**File:** `.mcp.json`

Add the `rust` entry:
```json
"rust": {
  "type": "stdio",
  "command": "node",
  "args": ["tools/mcp-launcher.mjs", "rust-mcp"],
  "env": {}
}
```

---

### Task 7: Update HANDOFF.md and CLAUDE.md

**HANDOFF.md changes:**
- Remove the "WINDOWS SESSION — do this first" block from Next Action
- Update MCPs note: rust MCP restored
- Add session log entry

**CLAUDE.md changes:**
- In the MCP Tools table, restore the `rust` MCP tools (find_references, find_definition, workspace_symbols, rename_symbol) — they were documented but then noted as removed pending migration
- Remove the note about rust MCP being removed

---

### Task 8: Verify existing tools still build

```bash
cd tools && cargo build --release 2>&1 | tail -10
```

All 5 servers must build cleanly. The game workspace must also be unaffected:
```bash
cargo test 2>&1 | tail -10
```

---

### Task 9: Commit

Stage and commit all changes:
```bash
git add tools/rust-mcp/ tools/Cargo.toml .mcp.json HANDOFF.md CLAUDE.md
git commit -m "tooling: migrate rust-mcp into repo, port to rmcp 0.8"
```

---

## Notes

- The `edition = "2024"` was changed to `"2021"` in the new Cargo.toml to match all other tools in the workspace. The code is compatible with both.
- The package name was changed from `rustmcp` to `rust-mcp` to match the naming convention of other tools (rhai-mcp-server, minijinja-mcp-server, etc.). The `[[bin]]` name is `rust-mcp` so the binary will be `rust-mcp.exe` / `rust-mcp`, matching what mcp-launcher.mjs expects.
- The crate name change means `use rustmcp::` in main.rs must become `use rust_mcp::` (Cargo converts hyphens to underscores for crate names).
- The tools/ workspace already has rmcp 0.8.5 in its lockfile — no new dependency resolution needed for rmcp itself.
