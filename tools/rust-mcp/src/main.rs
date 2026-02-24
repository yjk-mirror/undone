use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use rust_mcp::server::RustMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the rust-analyzer integration
    let mut rust_server = RustMcpServer::new();
    rust_server.start().await?;

    // Note: stdout is the MCP JSON-RPC channel — all logging goes to stderr
    eprintln!("Starting Rust MCP Server");
    eprintln!("Server running on stdio transport...");

    // Start the MCP server using the ServiceExt trait
    let service = rust_server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
