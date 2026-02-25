#!/usr/bin/env bash
# Cross-platform MCP server launcher.
#
# Usage: bash tools/mcp-launcher.sh <server-binary-name>
#
# Resolves the binary path from this script's own location (tools/),
# appends .exe on Windows, and exec's into it — replacing the shell
# process entirely (zero memory overhead from the launcher).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_NAME="${1:?mcp-launcher: missing server name argument}"

case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) EXT=".exe" ;;
  *)                     EXT="" ;;
esac

BINARY="${SCRIPT_DIR}/target/release/${SERVER_NAME}${EXT}"

if [[ ! -x "$BINARY" ]]; then
  echo "mcp-launcher: binary not found or not executable: $BINARY" >&2
  exit 1
fi

exec "$BINARY"
