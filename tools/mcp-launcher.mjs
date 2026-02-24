#!/usr/bin/env node
/**
 * Cross-platform MCP server launcher.
 *
 * Usage: node tools/mcp-launcher.mjs <server-binary-name>
 *
 * Resolves the binary path from this script's own location (tools/),
 * appends .exe on Windows, and pipes stdio through. No hardcoded paths.
 * Works on Linux and Windows without any per-machine configuration.
 */
import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join, resolve } from 'path';
import { platform } from 'os';

const __dirname = dirname(fileURLToPath(import.meta.url));
// This script lives in tools/ — one level up is the repo root.
const toolsDir = resolve(__dirname);
const serverName = process.argv[2];

if (!serverName) {
  process.stderr.write('mcp-launcher: missing server name argument\n');
  process.exit(1);
}

const ext = platform() === 'win32' ? '.exe' : '';
const binaryPath = join(toolsDir, 'target', 'release', `${serverName}${ext}`);

const proc = spawn(binaryPath, [], {
  stdio: 'inherit',
  env: process.env,
});

proc.on('error', (err) => {
  process.stderr.write(`mcp-launcher: failed to start '${binaryPath}': ${err.message}\n`);
  process.exit(1);
});

proc.on('exit', (code, signal) => {
  process.exit(code ?? (signal ? 1 : 0));
});
