use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::RgbaImage;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::capture;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScreenshotInput {
    /// Partial window title to match (case-sensitive substring).
    /// Example: "Undone" matches a window titled "Undone".
    pub title: String,
}

#[derive(Clone)]
pub struct ScreenshotServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ScreenshotServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Capture a screenshot of a running native window by partial title match. Uses Windows Graphics Capture API — no focus stealing, no border on Windows 11, cursor excluded. Returns the screenshot as an image/png content block.")]
    async fn screenshot_window(
        &self,
        params: Parameters<ScreenshotInput>,
    ) -> Result<CallToolResult, McpError> {
        let title = params.0.title.clone();

        // Run the blocking WGC capture on a dedicated thread so we don't block tokio.
        let frame = tokio::task::spawn_blocking(move || capture::capture_window(&title))
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // BGRA8 → RGBA8 (swap B and R channels; alpha unchanged).
        let rgba: Vec<u8> = frame
            .data
            .chunks_exact(4)
            .flat_map(|p| [p[2], p[1], p[0], p[3]])
            .collect();

        let img = RgbaImage::from_raw(frame.width, frame.height, rgba)
            .ok_or_else(|| McpError::internal_error("image dimensions mismatch", None))?;

        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

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
