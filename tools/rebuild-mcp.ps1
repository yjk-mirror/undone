<#
.SYNOPSIS
    Rebuild the Undone MCP server binaries.

.DESCRIPTION
    The five MCP servers (rhai, minijinja, screenshot, game-input, rust) referenced
    by .mcp.json live in tools/target/release/. That is a Cargo build-artifact
    directory, so anything that wipes target/ (a `cargo clean`, a disk sweep, or the
    DevClean-WeeklyScan scheduled task when this project has sat idle >7 days) silently
    removes the binaries. When that happens Claude Code shows the MCP servers as
    failed-to-connect with no obvious cause.

    Run this script after such a break to restore them, then restart your Claude Code
    session so the MCP servers reconnect (they only connect at session start).

.EXAMPLE
    pwsh tools/rebuild-mcp.ps1
#>

$ErrorActionPreference = 'Stop'

# Repo root = parent of this script's directory (tools/).
$toolsDir = $PSScriptRoot
$repoRoot = Split-Path $toolsDir -Parent

$expected = @(
    'rhai-mcp-server',
    'minijinja-mcp-server',
    'screenshot-mcp',
    'game-input-mcp',
    'rust-mcp'
)

Write-Host "Building MCP servers (tools workspace)..." -ForegroundColor Cyan
Push-Location $toolsDir
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "cargo build failed (exit $LASTEXITCODE)." -ForegroundColor Red
        exit 1
    }
} finally {
    Pop-Location
}

$releaseDir = Join-Path $toolsDir 'target\release'
$missing = @()
Write-Host ""
foreach ($name in $expected) {
    $exe = Join-Path $releaseDir "$name.exe"
    if (Test-Path $exe) {
        Write-Host ("  OK      {0}.exe" -f $name) -ForegroundColor Green
    } else {
        Write-Host ("  MISSING {0}.exe" -f $name) -ForegroundColor Red
        $missing += $name
    }
}

Write-Host ""
if ($missing.Count -gt 0) {
    Write-Host ("{0} binary(ies) still missing after build." -f $missing.Count) -ForegroundColor Red
    exit 1
}

Write-Host "All 5 MCP binaries present. Restart your Claude Code session to reconnect them." -ForegroundColor Green
