#[cfg(target_os = "windows")]
mod capture;
#[cfg(target_os = "windows")]
mod server;

use anyhow::Result;
#[cfg(target_os = "windows")]
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("screenshot-mcp starting");

    #[cfg(target_os = "windows")]
    {
        let service = server::ScreenshotServer::new()
            .serve(stdio())
            .await
            .inspect_err(|e| tracing::error!("server error: {:?}", e))?;

        service.waiting().await?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("screenshot-mcp is Windows-only (uses Windows Graphics Capture API).");
        std::process::exit(1);
    }

    Ok(())
}
