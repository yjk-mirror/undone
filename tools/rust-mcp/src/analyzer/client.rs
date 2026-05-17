use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;

use crate::analyzer::protocol::*;

fn get_rust_analyzer_path() -> String {
    std::env::var("RUST_ANALYZER_PATH").unwrap_or_else(|_| {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let path = format!("{home}/.cargo/bin/rust-analyzer");
        if !std::path::Path::new(&path).exists() {
            let with_exe = format!("{path}.exe");
            if std::path::Path::new(&with_exe).exists() {
                return with_exe;
            }
        }
        path
    })
}

pub struct RustAnalyzerClient {
    process: Option<Child>,
    request_id: u64,
    initialized: bool,
}

impl Default for RustAnalyzerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAnalyzerClient {
    pub fn new() -> Self {
        Self {
            process: None,
            request_id: 0,
            initialized: false,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let rust_analyzer_path = get_rust_analyzer_path();
        let child = tokio::process::Command::new(&rust_analyzer_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.process = Some(child);
        self.initialize().await?;
        Ok(())
    }

    async fn initialize(&mut self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let root_uri = format!("file://{}", current_dir.display());

        let init_params = json!({
            "processId": null,
            "clientInfo": { "name": "rust-mcp-server", "version": "0.1.0" },
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "definition": { "dynamicRegistration": false },
                    "references": { "dynamicRegistration": false },
                    "publishDiagnostics": { "relatedInformation": true }
                },
                "workspace": {
                    "symbol": { "dynamicRegistration": false }
                }
            },
            // Headless tool provider — don't behave like an IDE.
            "initializationOptions": {
                "cachePriming": { "enable": false },
                "lru": { "capacity": 32 },
                "numThreads": 2,
                // Empty overrideCommand disables background cargo-check.
                "check": { "overrideCommand": [] },
                "cargo": { "allTargets": false }
            }
        });

        let _response = self
            .send_request_internal("initialize", init_params)
            .await?;
        self.send_notification("initialized", json!({})).await?;
        self.initialized = true;
        Ok(())
    }

    async fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.send_message(&notification).await
    }

    async fn send_request_internal(&mut self, method: &str, params: Value) -> Result<Value> {
        self.request_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        });
        self.send_message(&request).await?;
        self.read_response(self.request_id).await
    }

    async fn send_message(&mut self, message: &Value) -> Result<()> {
        let content = message.to_string();
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        if let Some(child) = &mut self.process {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(header.as_bytes()).await?;
                stdin.write_all(content.as_bytes()).await?;
                stdin.flush().await?;
            }
        }
        Ok(())
    }

    async fn read_response(&mut self, expected_id: u64) -> Result<Value> {
        let Some(child) = &mut self.process else {
            return Err(anyhow::anyhow!("rust-analyzer process not started"));
        };
        let Some(stdout) = child.stdout.as_mut() else {
            return Err(anyhow::anyhow!("rust-analyzer stdout unavailable"));
        };
        let mut reader = BufReader::new(stdout);

        loop {
            let mut content_length: Option<usize> = None;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).await?;
                if line == "\r\n" {
                    break;
                }
                if let Some(stripped) = line.strip_prefix("Content-Length:") {
                    content_length = Some(stripped.trim().parse()?);
                }
            }

            let Some(length) = content_length else {
                continue;
            };
            let mut content = vec![0u8; length];
            reader.read_exact(&mut content).await?;
            let response: Value = serde_json::from_slice(&content)?;
            if response.get("id").and_then(Value::as_u64) == Some(expected_id) {
                return Ok(response);
            }
        }
    }

    fn check_initialized(&self) -> Result<()> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Client not initialized"));
        }
        Ok(())
    }

    pub async fn find_definition(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<String> {
        self.check_initialized()?;
        let params = create_text_document_position_params(file_path, line, character);
        let response = self
            .send_request_internal("textDocument/definition", params)
            .await?;
        Ok(format!("Definition response: {response}"))
    }

    pub async fn find_references(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
    ) -> Result<String> {
        self.check_initialized()?;
        let params = create_references_params(file_path, line, character);
        let response = self
            .send_request_internal("textDocument/references", params)
            .await?;
        Ok(format!("References response: {response}"))
    }

    pub async fn workspace_symbols(&mut self, query: &str) -> Result<String> {
        self.check_initialized()?;
        let params = create_workspace_symbol_params(query);
        let response = self
            .send_request_internal("workspace/symbol", params)
            .await?;
        Ok(format!("Workspace symbols response: {response}"))
    }

    pub async fn rename_symbol(
        &mut self,
        file_path: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<String> {
        self.check_initialized()?;
        let params = create_rename_params(file_path, line, character, new_name);
        let response = self
            .send_request_internal("textDocument/rename", params)
            .await?;
        Ok(format!("Rename response: {response}"))
    }
}
