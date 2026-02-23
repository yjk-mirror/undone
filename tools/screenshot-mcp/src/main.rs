mod capture;
mod server;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("screenshot-mcp starting");

    let service = server::ScreenshotServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("server error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
