use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize, Serialize)]
pub struct DevCommandResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
}

pub fn command_file_path() -> PathBuf {
    std::env::temp_dir().join("undone-dev-cmd.json")
}

pub fn result_file_path() -> PathBuf {
    std::env::temp_dir().join("undone-dev-result.json")
}

pub fn send_command(command_json: &str, timeout: Duration) -> Result<DevCommandResponse> {
    let command_path = command_file_path();
    let result_path = result_file_path();

    let _ = std::fs::remove_file(&result_path);
    let tmp_path = command_path.with_extension("tmp");
    std::fs::write(&tmp_path, command_json).context("write command failed")?;
    std::fs::rename(&tmp_path, &command_path).context("rename command failed")?;

    let deadline = Instant::now() + timeout;
    loop {
        if result_path.exists() {
            let result = std::fs::read_to_string(&result_path).context("read result failed")?;
            let _ = std::fs::remove_file(&result_path);
            return serde_json::from_str(&result).context("parse result failed");
        }
        if Instant::now() > deadline {
            return Err(anyhow!(
                "timeout waiting for game response. Is the game running with --dev?"
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
