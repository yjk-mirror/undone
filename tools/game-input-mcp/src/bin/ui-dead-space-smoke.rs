#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use anyhow::{anyhow, bail, Context};
    use game_input_mcp::dev_client::{send_command, DevCommandResponse};
    use game_input_mcp::input;
    use game_input_mcp::ui_audit::{
        assert_no_runtime_change, assert_runtime_change, assert_tab, summarize_runtime,
        RuntimeSummary,
    };
    use serde_json::json;
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    const WINDOW_TITLE: &str = "Undone";
    const WINDOW_WIDTH: f64 = 1200.0;
    const WINDOW_HEIGHT: f64 = 800.0;
    const TITLE_BAR_DEAD_SPACE_X: i32 = 60;
    const TITLE_BAR_Y: i32 = 20;
    const STORY_PANEL_LEFT_X: i32 = 280;
    const CONTINUE_DEAD_SPACE_X: i32 = STORY_PANEL_LEFT_X + 40;
    const CONTINUE_BUTTON_CENTER_X: i32 = STORY_PANEL_LEFT_X + 460;
    const BOTTOM_BAR_Y: i32 = 760;

    struct ChildGuard(Child);

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn send_json(payload: serde_json::Value) -> anyhow::Result<DevCommandResponse> {
        let response = send_command(&payload.to_string(), Duration::from_secs(3))?;
        if response.success {
            Ok(response)
        } else {
            bail!(response.message)
        }
    }

    fn runtime_summary() -> anyhow::Result<RuntimeSummary> {
        let response = send_json(json!({ "command": "get_runtime_state" }))?;
        let value = serde_json::to_value(response).context("serialize runtime response")?;
        summarize_runtime(&value)
    }

    fn send_runtime_command(payload: serde_json::Value) -> anyhow::Result<RuntimeSummary> {
        let response = send_json(payload)?;
        let value = serde_json::to_value(response).context("serialize runtime response")?;
        summarize_runtime(&value)
    }

    fn wait_for_window(timeout: Duration) -> anyhow::Result<windows::Win32::Foundation::HWND> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(hwnd) = input::find_window(WINDOW_TITLE) {
                return Ok(hwnd);
            }
            if Instant::now() > deadline {
                bail!("timed out waiting for the Undone window");
            }
            sleep(Duration::from_millis(100));
        }
    }

    fn wait_for_runtime(timeout: Duration) -> anyhow::Result<RuntimeSummary> {
        let deadline = Instant::now() + timeout;
        loop {
            match runtime_summary() {
                Ok(summary) => return Ok(summary),
                Err(_) if Instant::now() <= deadline => sleep(Duration::from_millis(100)),
                Err(err) => return Err(err),
            }
        }
    }

    fn play_until_continue() -> anyhow::Result<RuntimeSummary> {
        for _ in 0..32 {
            let current = runtime_summary()?;
            if current.awaiting_continue {
                return Ok(current);
            }
            let action_id = current
                .visible_action_ids
                .first()
                .cloned()
                .ok_or_else(|| anyhow!("runtime never reached a continue state"))?;
            let after = send_runtime_command(json!({
                "command": "choose_action",
                "action_id": action_id,
            }))?;
            if after.awaiting_continue {
                return Ok(after);
            }
        }

        bail!("runtime did not reach awaiting_continue within 32 actions")
    }

    fn build_release(root: &Path) -> anyhow::Result<()> {
        let status = Command::new("cargo")
            .args(["build", "--release", "--bin", "undone"])
            .current_dir(root)
            .status()
            .context("failed to start cargo build")?;
        if status.success() {
            Ok(())
        } else {
            bail!("cargo build --release --bin undone failed")
        }
    }

    fn launch_game(root: &Path) -> anyhow::Result<ChildGuard> {
        let exe = root.join("target").join("release").join("undone.exe");
        let child = Command::new(&exe)
            .args(["--dev", "--quick"])
            .current_dir(root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to launch {}", exe.display()))?;
        Ok(ChildGuard(child))
    }

    let root = std::env::current_dir().context("failed to read current directory")?;
    let _ = std::fs::remove_file(std::env::temp_dir().join("undone-dev-cmd.json"));
    let _ = std::fs::remove_file(std::env::temp_dir().join("undone-dev-result.json"));

    if input::find_process("undone.exe")?.is_some() {
        bail!("undone.exe is already running; close it before running ui-dead-space-smoke");
    }

    build_release(&root)?;
    let _child = launch_game(&root)?;
    let hwnd = wait_for_window(Duration::from_secs(30))?;
    let _ = wait_for_runtime(Duration::from_secs(15))?;

    let _ = send_runtime_command(json!({
        "command": "set_window_size",
        "width": WINDOW_WIDTH,
        "height": WINDOW_HEIGHT,
    }))?;
    let _ = send_runtime_command(json!({
        "command": "set_tab",
        "tab": "game",
    }))?;
    let before_title = runtime_summary()?;

    input::click(hwnd, TITLE_BAR_DEAD_SPACE_X, TITLE_BAR_Y)?;
    sleep(Duration::from_millis(250));
    let after_title = runtime_summary()?;
    assert_tab("game", &after_title)?;
    assert_no_runtime_change("title bar dead space", &before_title, &after_title)?;

    let before_continue = play_until_continue()?;
    input::click(hwnd, CONTINUE_DEAD_SPACE_X, BOTTOM_BAR_Y)?;
    sleep(Duration::from_millis(250));
    let after_dead_click = runtime_summary()?;
    assert_no_runtime_change(
        "continue bar dead space",
        &before_continue,
        &after_dead_click,
    )?;

    input::click(hwnd, CONTINUE_BUTTON_CENTER_X, BOTTOM_BAR_Y)?;
    sleep(Duration::from_millis(250));
    let after_continue_click = runtime_summary()?;
    assert_runtime_change(
        "visible continue button click",
        &before_continue,
        &after_continue_click,
    )?;

    println!("ui-dead-space-smoke passed");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("ui-dead-space-smoke is Windows-only")
}
