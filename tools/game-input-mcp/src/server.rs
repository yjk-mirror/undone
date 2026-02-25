use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::input;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartGameInput {
    /// Working directory for `cargo run --release`. Typically the game workspace root.
    pub working_dir: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StopGameInput {
    /// Process name to kill (e.g. "undone.exe").
    pub exe_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IsGameRunningInput {
    /// Process name to check (e.g. "undone.exe").
    pub exe_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PressKeyInput {
    /// Partial window title to match (case-sensitive substring).
    /// Example: "Undone" matches a window titled "Undone".
    pub title: String,
    /// Key to press. Supported: "1"-"9", "enter", "tab", "escape", "space".
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ClickInput {
    /// Partial window title to match (case-sensitive substring).
    pub title: String,
    /// X coordinate relative to the window's client area.
    pub x: i32,
    /// Y coordinate relative to the window's client area.
    pub y: i32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScrollInput {
    /// Partial window title to match (case-sensitive substring).
    pub title: String,
    /// X coordinate relative to the window's client area.
    pub x: i32,
    /// Y coordinate relative to the window's client area.
    pub y: i32,
    /// Scroll delta in ticks. Positive = scroll up, negative = scroll down.
    /// One tick is one notch of the mouse wheel.
    pub delta: i32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HoverInput {
    /// Partial window title to match (case-sensitive substring).
    pub title: String,
    /// X coordinate relative to the window's client area.
    pub x: i32,
    /// Y coordinate relative to the window's client area.
    pub y: i32,
}

#[derive(Clone)]
pub struct GameInputServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl GameInputServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Press a key in a running native window without stealing focus or moving the cursor. Posts WM_KEYDOWN + WM_KEYUP via PostMessage. Supported keys: \"1\"-\"9\", \"enter\", \"tab\", \"escape\", \"space\"."
    )]
    async fn press_key(
        &self,
        params: Parameters<PressKeyInput>,
    ) -> Result<CallToolResult, McpError> {
        let title = &params.0.title;
        let key = &params.0.key;

        let hwnd = input::find_window(title)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        input::press_key(hwnd, key)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Pressed '{}' in window matching '{}'",
            key, title
        ))]))
    }

    #[tool(
        description = "Click at a position in a running native window without stealing focus or moving the cursor. Posts WM_LBUTTONDOWN + WM_LBUTTONUP via PostMessage at window-client-relative coordinates."
    )]
    async fn click(
        &self,
        params: Parameters<ClickInput>,
    ) -> Result<CallToolResult, McpError> {
        let title = &params.0.title;
        let x = params.0.x;
        let y = params.0.y;

        let hwnd = input::find_window(title)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        input::click(hwnd, x, y)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Clicked at ({}, {}) in window matching '{}'",
            x, y, title
        ))]))
    }

    #[tool(
        description = "Scroll the mouse wheel in a running native window without stealing focus. Posts WM_MOUSEWHEEL via PostMessage. Use negative delta to scroll down, positive to scroll up."
    )]
    async fn scroll(
        &self,
        params: Parameters<ScrollInput>,
    ) -> Result<CallToolResult, McpError> {
        let title = &params.0.title;
        let x = params.0.x;
        let y = params.0.y;
        let delta = params.0.delta;

        let hwnd = input::find_window(title)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        input::scroll(hwnd, x, y, delta)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let direction = if delta > 0 { "up" } else { "down" };
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Scrolled {} by {} ticks at ({}, {}) in window matching '{}'",
            direction,
            delta.abs(),
            x,
            y,
            title
        ))]))
    }

    #[tool(
        description = "Move the mouse cursor to a position in a running native window without stealing focus. Posts WM_MOUSEMOVE via PostMessage. Use this to trigger hover effects on UI elements."
    )]
    async fn hover(
        &self,
        params: Parameters<HoverInput>,
    ) -> Result<CallToolResult, McpError> {
        let title = &params.0.title;
        let x = params.0.x;
        let y = params.0.y;

        let hwnd = input::find_window(title)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        input::hover(hwnd, x, y)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Hovered at ({}, {}) in window matching '{}'",
            x, y, title
        ))]))
    }

    #[tool(
        description = "Check if a game process is running. Returns PID if found, or 'not running'."
    )]
    async fn is_game_running(
        &self,
        params: Parameters<IsGameRunningInput>,
    ) -> Result<CallToolResult, McpError> {
        let exe_name = &params.0.exe_name;

        let result = tokio::task::spawn_blocking({
            let exe = exe_name.clone();
            move || input::find_process(&exe)
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        match result {
            Some(pid) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Running (PID {})",
                pid
            ))])),
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "'{}' is not running",
                exe_name
            ))])),
        }
    }

    #[tool(
        description = "Start the game by running `cargo run --release` in the given working directory. Returns immediately after spawning — the game runs independently. Returns the PID of the cargo process."
    )]
    async fn start_game(
        &self,
        params: Parameters<StartGameInput>,
    ) -> Result<CallToolResult, McpError> {
        let working_dir = params.0.working_dir.clone();

        let child = std::process::Command::new("cargo")
            .args(["run", "--release", "--bin", "undone"])
            .current_dir(&working_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| McpError::internal_error(format!("failed to spawn cargo: {}", e), None))?;

        let pid = child.id();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Game building and launching (cargo PID {}). The game window will appear once compilation finishes. Use is_game_running to check.",
            pid
        ))]))
    }

    #[tool(
        description = "Stop the game process by killing it. Finds the process by exe name and terminates it."
    )]
    async fn stop_game(
        &self,
        params: Parameters<StopGameInput>,
    ) -> Result<CallToolResult, McpError> {
        let exe_name = &params.0.exe_name;

        let pid = tokio::task::spawn_blocking({
            let exe = exe_name.clone();
            move || input::find_process(&exe)
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        match pid {
            Some(pid) => {
                tokio::task::spawn_blocking(move || input::kill_process(pid))
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Stopped '{}' (PID {})",
                    exe_name, pid
                ))]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "'{}' is not running",
                exe_name
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for GameInputServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Game input tool for sending keystrokes, clicks, scroll, and hover to native \
                 Windows GUI apps without stealing focus. Use press_key(title, key) for keyboard \
                 input, click(title, x, y) for mouse clicks, scroll(title, x, y, delta) for \
                 mouse wheel, and hover(title, x, y) for mouse move/hover effects. Also provides \
                 game lifecycle tools: start_game(working_dir) to build and launch, \
                 stop_game(exe_name) to kill the process, and is_game_running(exe_name) to check \
                 if it's running and get the PID."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
