use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::input;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartGameInput {
    /// Working directory for `cargo run --release`. Typically the game workspace root.
    pub working_dir: String,
    /// Launch with `--dev --quick` so the dev panel and IPC are immediately available.
    pub dev_mode: bool,
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DevCommandInput {
    /// Raw JSON command payload written to the game's dev IPC command file.
    pub command_json: String,
    /// How long to wait for the game to respond before returning a timeout.
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetGameStateInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetRuntimeStateInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct JumpToSceneInput {
    /// Scene ID to jump to, for example `base::coffee_shop`.
    pub scene_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetGameStatInput {
    /// Supported: money, stress, anxiety, femininity.
    pub stat: String,
    pub value: i32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetGameFlagInput {
    pub flag: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AdvanceTimeInput {
    /// Number of weeks to advance.
    pub weeks: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetNpcLikingInput {
    /// NPC name (case-insensitive), e.g. "Jake".
    pub npc_name: String,
    /// Liking level: Neutral, Ok, Like, or Close.
    pub level: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetAllNpcLikingInput {
    /// Liking level to set for all NPCs: Neutral, Ok, Like, or Close.
    pub level: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChooseActionInput {
    /// Stable visible action id returned by get_runtime_state.
    pub action_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContinueSceneInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetTabInput {
    /// Supported tabs: game, saves, settings, dev.
    pub tab: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetWindowSizeInput {
    /// Requested window width in logical pixels.
    pub width: f64,
    /// Requested window height in logical pixels.
    pub height: f64,
}

#[derive(Clone)]
pub struct GameInputServer {
    tool_router: ToolRouter<Self>,
}

fn runtime_state_payload() -> String {
    json!({
        "command": "get_runtime_state"
    })
    .to_string()
}

fn choose_action_payload(action_id: &str) -> String {
    json!({
        "command": "choose_action",
        "action_id": action_id,
    })
    .to_string()
}

fn continue_scene_payload() -> String {
    json!({
        "command": "continue_scene"
    })
    .to_string()
}

fn set_tab_payload(tab: &str) -> String {
    json!({
        "command": "set_tab",
        "tab": tab,
    })
    .to_string()
}

fn set_window_size_payload(width: f64, height: f64) -> String {
    json!({
        "command": "set_window_size",
        "width": width,
        "height": height,
    })
    .to_string()
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

        let hwnd =
            input::find_window(title).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        input::press_key(hwnd, key).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Pressed '{}' in window matching '{}'",
            key, title
        ))]))
    }

    #[tool(
        description = "Click at a position in a running native window without stealing focus or moving the cursor. Posts WM_LBUTTONDOWN + WM_LBUTTONUP via PostMessage at window-client-relative coordinates."
    )]
    async fn click(&self, params: Parameters<ClickInput>) -> Result<CallToolResult, McpError> {
        let title = &params.0.title;
        let x = params.0.x;
        let y = params.0.y;

        let hwnd =
            input::find_window(title).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        input::click(hwnd, x, y).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Clicked at ({}, {}) in window matching '{}'",
            x, y, title
        ))]))
    }

    #[tool(
        description = "Scroll the mouse wheel in a running native window without stealing focus. Posts WM_MOUSEWHEEL via PostMessage. Use negative delta to scroll down, positive to scroll up."
    )]
    async fn scroll(&self, params: Parameters<ScrollInput>) -> Result<CallToolResult, McpError> {
        let title = &params.0.title;
        let x = params.0.x;
        let y = params.0.y;
        let delta = params.0.delta;

        let hwnd =
            input::find_window(title).map_err(|e| McpError::internal_error(e.to_string(), None))?;

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
    async fn hover(&self, params: Parameters<HoverInput>) -> Result<CallToolResult, McpError> {
        let title = &params.0.title;
        let x = params.0.x;
        let y = params.0.y;

        let hwnd =
            input::find_window(title).map_err(|e| McpError::internal_error(e.to_string(), None))?;

        input::hover(hwnd, x, y).map_err(|e| McpError::internal_error(e.to_string(), None))?;

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
        description = "Start the game by running `cargo run --release` in the given working directory. When dev_mode is true it launches with `--dev --quick` so the dev panel and IPC are available immediately."
    )]
    async fn start_game(
        &self,
        params: Parameters<StartGameInput>,
    ) -> Result<CallToolResult, McpError> {
        let working_dir = params.0.working_dir.clone();
        let dev_mode = params.0.dev_mode;

        let mut command = std::process::Command::new("cargo");
        command.args(["run", "--release", "--bin", "undone"]);
        if dev_mode {
            command.args(["--", "--dev", "--quick"]);
        }

        let child = command
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
        description = "Send a raw dev command JSON payload to a running Undone game launched with --dev. Returns the JSON response from the game."
    )]
    async fn dev_command(
        &self,
        params: Parameters<DevCommandInput>,
    ) -> Result<CallToolResult, McpError> {
        let timeout_ms = params.0.timeout_ms.unwrap_or(2000);
        let command_path = std::env::temp_dir().join("undone-dev-cmd.json");
        let result_path = std::env::temp_dir().join("undone-dev-result.json");

        let _ = std::fs::remove_file(&result_path);
        let tmp_path = command_path.with_extension("tmp");
        std::fs::write(&tmp_path, &params.0.command_json)
            .map_err(|e| McpError::internal_error(format!("write command failed: {e}"), None))?;
        std::fs::rename(&tmp_path, &command_path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            McpError::internal_error(format!("rename command failed: {e}"), None)
        })?;

        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            if result_path.exists() {
                let result = std::fs::read_to_string(&result_path).map_err(|e| {
                    McpError::internal_error(format!("read result failed: {e}"), None)
                })?;
                let _ = std::fs::remove_file(&result_path);
                return Ok(CallToolResult::success(vec![Content::text(result)]));
            }
            if std::time::Instant::now() > deadline {
                return Ok(CallToolResult::success(vec![Content::text(
                    r#"{"success": false, "message": "Timeout waiting for game response. Is the game running with --dev?"}"#,
                )]));
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    #[tool(description = "Get the current game state from a running Undone game in dev mode.")]
    async fn get_game_state(
        &self,
        _params: Parameters<GetGameStateInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "get_state"
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(
        description = "Get the current player-visible runtime state from a running Undone game in dev mode."
    )]
    async fn get_runtime_state(
        &self,
        _params: Parameters<GetRuntimeStateInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: runtime_state_payload(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Jump to a specific scene in a running Undone game in dev mode.")]
    async fn jump_to_scene(
        &self,
        params: Parameters<JumpToSceneInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "jump_to_scene",
                "scene_id": params.0.scene_id,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Choose a visible action by stable id in a running Undone game in dev mode.")]
    async fn choose_action(
        &self,
        params: Parameters<ChooseActionInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: choose_action_payload(&params.0.action_id),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Continue the runtime after a scene finishes in a running Undone game in dev mode.")]
    async fn continue_scene(
        &self,
        _params: Parameters<ContinueSceneInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: continue_scene_payload(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Switch the active app tab in a running Undone game in dev mode.")]
    async fn set_tab(
        &self,
        params: Parameters<SetTabInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: set_tab_payload(&params.0.tab),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Resize the running Undone window in dev mode to an exact width and height.")]
    async fn set_window_size(
        &self,
        params: Parameters<SetWindowSizeInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: set_window_size_payload(params.0.width, params.0.height),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Set a dev-editable stat in a running Undone game in dev mode.")]
    async fn set_game_stat(
        &self,
        params: Parameters<SetGameStatInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "set_stat",
                "stat": params.0.stat,
                "value": params.0.value,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Set a game flag in a running Undone game in dev mode.")]
    async fn set_game_flag(
        &self,
        params: Parameters<SetGameFlagInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "set_flag",
                "flag": params.0.flag,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Remove a game flag in a running Undone game in dev mode.")]
    async fn remove_game_flag(
        &self,
        params: Parameters<SetGameFlagInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "remove_flag",
                "flag": params.0.flag,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Advance the game clock by N weeks in a running Undone game in dev mode.")]
    async fn advance_time(
        &self,
        params: Parameters<AdvanceTimeInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "advance_time",
                "weeks": params.0.weeks,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Set an NPC's liking level in a running Undone game in dev mode.")]
    async fn set_npc_liking(
        &self,
        params: Parameters<SetNpcLikingInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "set_npc_liking",
                "npc_name": params.0.npc_name,
                "level": params.0.level,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(
        description = "Set all NPCs' liking level at once in a running Undone game in dev mode."
    )]
    async fn set_all_npc_liking(
        &self,
        params: Parameters<SetAllNpcLikingInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "set_all_npc_liking",
                "level": params.0.level,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
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
                 game lifecycle tools: start_game(working_dir, dev_mode) to build and launch, \
                 stop_game(exe_name) to kill the process, is_game_running(exe_name) to check \
                 if it's running and get the PID, and dev-mode IPC helpers such as \
                 get_game_state(), get_runtime_state(), jump_to_scene(scene_id), \
                 choose_action(action_id), continue_scene(), set_tab(tab), \
                 set_window_size(width, height), \
                 set_game_stat(stat, value), \
                 set_game_flag(flag), remove_game_flag(flag), advance_time(weeks), \
                 set_npc_liking(npc_name, level), and set_all_npc_liking(level)."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        choose_action_payload, continue_scene_payload, runtime_state_payload,
        set_tab_payload, set_window_size_payload,
    };
    use serde_json::json;

    #[test]
    fn runtime_state_payload_uses_runtime_command_name() {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&runtime_state_payload()).unwrap(),
            json!({ "command": "get_runtime_state" })
        );
    }

    #[test]
    fn choose_action_payload_includes_stable_action_id() {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&choose_action_payload("wait")).unwrap(),
            json!({ "command": "choose_action", "action_id": "wait" })
        );
    }

    #[test]
    fn continue_scene_payload_uses_continue_command_name() {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&continue_scene_payload()).unwrap(),
            json!({ "command": "continue_scene" })
        );
    }

    #[test]
    fn set_tab_payload_includes_requested_tab() {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&set_tab_payload("dev")).unwrap(),
            json!({ "command": "set_tab", "tab": "dev" })
        );
    }

    #[test]
    fn set_window_size_payload_uses_resize_command_name() {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&set_window_size_payload(1800.0, 1000.0))
                .unwrap(),
            json!({
                "command": "set_window_size",
                "width": 1800.0,
                "height": 1000.0
            })
        );
    }
}
