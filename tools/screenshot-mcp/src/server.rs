use std::sync::{Arc, Mutex};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::capture::SessionManager;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScreenshotInput {
    /// Partial window title to match (case-sensitive substring).
    /// Example: "Undone" matches a window titled "Undone".
    pub title: String,
}

#[derive(Clone)]
pub struct ScreenshotServer {
    tool_router: ToolRouter<Self>,
    sessions: Arc<Mutex<SessionManager>>,
}

#[tool_router]
impl ScreenshotServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            sessions: Arc::new(Mutex::new(SessionManager::new())),
        }
    }

    #[tool(description = "Capture a screenshot of a running native window by partial title match. Uses Windows Graphics Capture API — no focus stealing, no border on Windows 11, cursor excluded. Returns the screenshot as an image/png content block.")]
    async fn screenshot_window(
        &self,
        params: Parameters<ScreenshotInput>,
    ) -> Result<CallToolResult, McpError> {
        let title = params.0.title.clone();
        let sessions = Arc::clone(&self.sessions);

        // Run on a blocking thread — session creation involves WGC init,
        // and frame reads touch a Mutex that the capture thread writes to.
        let frame = tokio::task::spawn_blocking(move || {
            let mut mgr = sessions.lock().unwrap();
            mgr.capture(&title)
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // BGRA8 → RGBA8 in-place (swap B and R channels; alpha unchanged).
        let mut rgba = frame.data;
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        // Encode PNG with fast compression — agent use doesn't need small files.
        let mut png_bytes: Vec<u8> = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_bytes, frame.width, frame.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_compression(png::Compression::Fast);
            let mut writer = encoder
                .write_header()
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
            writer
                .write_image_data(&rgba)
                .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        }

        let b64 = STANDARD.encode(&png_bytes);

        Ok(CallToolResult::success(vec![Content::image(
            b64,
            "image/png",
        )]))
    }
}

#[tool_handler]
impl ServerHandler for ScreenshotServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Screenshot tool for native Windows GUI apps. \
                 Use screenshot_window(title) to capture a running window by partial title match."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
