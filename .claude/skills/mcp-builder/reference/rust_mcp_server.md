# Rust MCP Server Implementation Guide

## Overview

The official Rust MCP SDK (`rmcp`) provides a tokio-async implementation with procedural macros for clean tool definition. Use this for MCP servers written in Rust — particularly useful when the server needs to embed Rust libraries (e.g., `rhai`, `minijinja`) or when performance and type safety are priorities.

**Crates:** `rmcp` (core) + `rmcp-macros` (proc macros)
**Version:** 0.8.0 (crates.io)
**Source:** https://github.com/modelcontextprotocol/rust-sdk

---

## Quick Reference

### Cargo.toml
```toml
[package]
name = "my-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
rmcp = { version = "0.8", features = ["server", "transport-io"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
schemars = "0.8"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Server Naming Convention
- Crate name: `{service}-mcp-server` (e.g., `rhai-mcp-server`)
- Binary name: `{service}-mcp-server`

---

## Core Pattern

```rust
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::{Parameters, ToolRouter},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    Error as McpError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// 1. Define your input types with JsonSchema + Deserialize
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateInput {
    /// Path to the .rhai script to validate
    pub script: String,
    /// Optional engine configuration
    pub strict: Option<bool>,
}

// 2. Define your server struct
#[derive(Clone)]
pub struct MyServer {
    // shared state here (use Arc<Mutex<T>> if mutable)
}

impl MyServer {
    pub fn new() -> Self {
        Self {}
    }
}

// 3. Define tools in an impl block annotated with #[tool_router]
#[tool_router]
impl MyServer {
    #[tool(description = "Validate a script and return diagnostics")]
    async fn validate_script(
        &self,
        params: Parameters<ValidateInput>,
    ) -> Result<CallToolResult, McpError> {
        let input = params.0;
        // ... do work ...
        Ok(CallToolResult::success(vec![Content::text(
            format!("Validated: {}", input.script)
        )]))
    }
}

// 4. Implement ServerHandler
#[tool_handler]
impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("My MCP server description".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

// 5. main.rs — wire up stdio transport
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr only — never stdout (stdout is the MCP channel)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let service = MyServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
```

---

## Input Parameters

Use `Parameters<T>` where `T: JsonSchema + DeserializeOwned`. The macro generates the MCP JSON Schema automatically from your struct.

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagnosticsInput {
    /// Absolute path to the file to check
    pub file_path: String,

    /// Maximum number of diagnostics to return (default: 50)
    #[schemars(range(min = 1, max = 500))]
    pub limit: Option<u32>,

    /// Output format: "json" or "markdown"
    pub format: Option<String>,
}
```

Document each field with `///` doc comments — they become the field descriptions in the MCP schema.

---

## Tool Return Values

### Text content
```rust
Ok(CallToolResult::success(vec![
    Content::text("Your result here")
]))
```

### JSON content
```rust
let result = serde_json::to_string_pretty(&my_struct)?;
Ok(CallToolResult::success(vec![
    Content::text(result)
]))
```

### Error result (tool-level, not protocol-level)
```rust
Ok(CallToolResult::error(vec![
    Content::text(format!("Error: {}", e))
]))
```

### Protocol-level error (use sparingly — prefer tool-level errors above)
```rust
Err(McpError::invalid_params("Missing required field: script"))
```

---

## Structured Output with `Json<T>`

For tools returning typed structured data:

```rust
use rmcp::handler::server::tool::Json;

#[derive(Serialize, JsonSchema)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<DiagnosticError>,
    pub warnings: Vec<DiagnosticWarning>,
}

#[tool(description = "Validate and return structured diagnostics")]
async fn validate(
    &self,
    params: Parameters<ValidateInput>,
) -> Result<Json<ValidationResult>, McpError> {
    // ...
    Ok(Json(ValidationResult { valid: true, errors: vec![], warnings: vec![] }))
}
```

---

## Shared State

For servers that need mutable state, use `Arc<Mutex<T>>`:

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RhaiServer {
    engine: Arc<rhai::Engine>,  // immutable — no mutex needed
    cache: Arc<Mutex<HashMap<String, ParseResult>>>,  // mutable
}

impl RhaiServer {
    pub fn new() -> Self {
        let mut engine = rhai::Engine::new();
        // register custom functions...
        Self {
            engine: Arc::new(engine),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
```

---

## Tool Annotations

The `#[tool]` macro accepts:

```rust
#[tool(
    name = "rhai_validate_script",          // override tool name (defaults to fn name)
    description = "Validate a Rhai script", // required
)]
async fn validate_script(...) -> Result<CallToolResult, McpError> { ... }
```

Set annotations in `get_info()` or per-tool via the `ToolDef` type. For read-only tools, document this in the description — "This tool does not modify any state."

---

## Transport: stdio vs HTTP

### stdio (use for all project-local MCP servers)
```rust
use rmcp::transport::stdio;

let service = MyServer::new().serve(stdio()).await?;
```

### Streamable HTTP (for multi-client or remote scenarios)
```rust
// See counter_streamhttp.rs example in rust-sdk repo
```

All Undone project MCP servers use **stdio** — they run as subprocesses of Claude Code.

---

## Logging

**Critical**: Never write to stdout. stdout is the MCP JSON-RPC channel.

```rust
// Correct — stderr only
tracing_subscriber::fmt()
    .with_writer(std::io::stderr)
    .with_ansi(false)  // disable ANSI colours in logs
    .init();

// In tool handlers:
tracing::debug!("Validating script: {}", path);
tracing::error!("Parse failed: {:?}", e);
```

---

## Project Structure

```
{service}-mcp-server/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs          # stdio transport setup, tracing init
    ├── server.rs        # Server struct, #[tool_router] impl, ServerHandler impl
    ├── tools/           # One module per tool domain (optional for larger servers)
    │   ├── mod.rs
    │   └── validate.rs
    └── types.rs         # Shared input/output types (Deserialize + JsonSchema)
```

For small servers (< 5 tools), `main.rs` + `server.rs` is sufficient.

---

## Testing

```bash
# Build
cargo build

# Lint
cargo clippy -- -D warnings

# Test with MCP Inspector (requires Node)
npx @modelcontextprotocol/inspector cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

---

## Registering with Claude Code (.mcp.json)

```json
{
  "mcpServers": {
    "rhai": {
      "type": "stdio",
      "command": "cargo",
      "args": ["run", "--manifest-path", "tools/rhai-mcp-server/Cargo.toml", "--release"],
      "env": {}
    }
  }
}
```

Or after `cargo build --release`:
```json
{
  "mcpServers": {
    "rhai": {
      "type": "stdio",
      "command": "./tools/rhai-mcp-server/target/release/rhai-mcp-server",
      "args": [],
      "env": {}
    }
  }
}
```

---

## Quality Checklist

### Implementation
- [ ] All tool methods in a `#[tool_router]` impl block
- [ ] All tool methods have `#[tool(description = "...")]`
- [ ] ServerHandler implemented with `#[tool_handler]`
- [ ] `get_info()` returns correct capabilities (at minimum `enable_tools()`)
- [ ] All input structs derive `JsonSchema + Deserialize`
- [ ] All fields documented with `///` doc comments
- [ ] Tool names follow `{service}_{action}` pattern (snake_case)

### Correctness
- [ ] No writes to stdout (use stderr for logs)
- [ ] `tracing_subscriber` configured with `.with_writer(std::io::stderr)`
- [ ] All errors returned as `CallToolResult::error(...)` not panics
- [ ] Mutex guards dropped before any await points

### Build
- [ ] `cargo build` succeeds
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Binary runs and responds to `npx @modelcontextprotocol/inspector`
