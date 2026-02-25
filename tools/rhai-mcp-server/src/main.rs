mod server;
mod validator;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // --validate <file>: syntax-check a file and exit (used by PostToolUse hook)
    if args.len() == 3 && args[1] == "--validate" {
        let path = std::path::Path::new(&args[2]);
        let diags = validator::validate_file(path);
        if diags.is_empty() {
            std::process::exit(0);
        }
        for d in &diags {
            eprintln!(
                "ERROR {}:{}: {}",
                d.line.unwrap_or(0),
                d.column.unwrap_or(0),
                d.message
            );
        }
        std::process::exit(1);
    }

    // Default: MCP server mode
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("rhai-mcp-server starting");

    let service = server::RhaiServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
