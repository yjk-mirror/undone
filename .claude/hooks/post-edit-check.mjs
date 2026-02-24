#!/usr/bin/env node
/**
 * PostToolUse hook — fires after every Edit or Write.
 * Checks the edited file with the appropriate validator.
 * Writes diagnostics to stderr so Claude sees them immediately.
 * Never blocks Claude — silently exits on any hook error.
 *
 * No hardcoded paths. Self-locates via import.meta.url so it works
 * on both Linux and Windows without modification.
 */

import { existsSync } from 'fs';
import { extname } from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join, resolve } from 'path';
import { platform } from 'os';

const __dirname = dirname(fileURLToPath(import.meta.url));
// This script lives in .claude/hooks/ — two levels up is the repo root.
const GAME_ROOT = resolve(__dirname, '..', '..');
const ext = platform() === 'win32' ? '.exe' : '';
const TOOLS_RELEASE = join(GAME_ROOT, 'tools', 'target', 'release');
const RHAI_BIN  = join(TOOLS_RELEASE, `rhai-mcp-server${ext}`);
const JINJA_BIN = join(TOOLS_RELEASE, `minijinja-mcp-server${ext}`);

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
