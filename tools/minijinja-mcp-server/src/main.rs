mod server;
mod validator;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // --validate <file>: validate a template file and exit (used by PostToolUse hook)
    if args.len() == 3 && args[1] == "--validate" {
        let source = match std::fs::read_to_string(&args[2]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ERROR ?: Cannot read file: {}", e);
                std::process::exit(1);
            }
        };
        let errors = validator::validate_template(&source);
        if errors.is_empty() {
            std::process::exit(0);
        }
        for e in &errors {
            eprintln!(
                "ERROR {}: {}",
                e.line.map(|l| l.to_string()).unwrap_or_else(|| "?".into()),
                e.message
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

    tracing::info!("minijinja-mcp-server starting");

    let service = server::MiniJinjaServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
