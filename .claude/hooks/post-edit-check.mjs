#!/usr/bin/env node
/**
 * PostToolUse hook — fires after every Edit or Write.
 * Checks the edited file with the appropriate validator.
 * Writes diagnostics to stderr so Claude sees them immediately.
 * Never blocks Claude — silently exits on any hook error.
 */

import { existsSync } from 'fs';
import { extname } from 'path';
import { execSync } from 'child_process';

const TOOLS_ROOT = 'C:/Users/YJK/dev/mirror/undone-tools';
const GAME_ROOT  = 'C:/Users/YJK/dev/mirror/undone';

const RHAI_BIN   = `${TOOLS_ROOT}/target/release/rhai-mcp-server.exe`;
const JINJA_BIN  = `${TOOLS_ROOT}/target/release/minijinja-mcp-server.exe`;

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => { input += chunk; });
process.stdin.on('end', () => {
  try {
    const payload = JSON.parse(input);
    const filePath = payload?.tool_input?.file_path;

    if (!filePath || !existsSync(filePath)) process.exit(0);

    const ext = extname(filePath).toLowerCase();

    if (ext === '.rs')                          checkRust(filePath);
    else if (ext === '.rhai')                   checkBinary(RHAI_BIN,  filePath, 'rhai');
    else if (ext === '.j2' || ext === '.jinja') checkBinary(JINJA_BIN, filePath, 'jinja');

    process.exit(0);
  } catch {
    process.exit(0); // never block Claude
  }
});

function checkRust(filePath) {
  try {
    const out = execSync(
      `cargo check --manifest-path "${GAME_ROOT}/Cargo.toml" --message-format=short 2>&1`,
      { encoding: 'utf8', timeout: 30000, cwd: GAME_ROOT }
    );
    if (out.trim()) process.stderr.write(`[cargo check]\n${out}\n`);
  } catch (e) {
    const msg = (e.stdout || '') + (e.stderr || '');
    if (msg.trim()) process.stderr.write(`[cargo check]\n${msg}\n`);
  }
}

function checkBinary(binaryPath, filePath, label) {
  if (!existsSync(binaryPath)) return; // binary not built — skip silently
  try {
    execSync(`"${binaryPath}" --validate "${filePath}"`, {
      encoding: 'utf8', timeout: 10000
    });
  } catch (e) {
    if (e.stderr?.trim()) {
      process.stderr.write(`[${label} check] ${filePath}\n${e.stderr}\n`);
    }
  }
}
