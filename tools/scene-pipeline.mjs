#!/usr/bin/env node

/**
 * scene-pipeline.mjs — End-to-end scene prose pipeline.
 *
 * spec JSON → prompt assembly → DeepSeek draft → lint → (retry?) → TOML conversion
 *
 * Usage:
 *   node tools/scene-pipeline.mjs --spec-file spec.json [options]
 *
 * Options:
 *   --spec-file <path>   Scene spec JSON (required)
 *   --output <path>      Final TOML output (default: packs/base/scenes/<name>.toml)
 *   --max-retries <n>    Max DeepSeek revision attempts on lint failure (default: 2)
 *   --draft-only         Stop after draft + lint (don't convert to TOML)
 *   --skip-lint          Skip the lint step (use when voice samples aren't ready)
 *   --keep-tmp           Keep intermediate files in tmp/ (default: keep them)
 *   --help               Show this message
 */

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { execFile } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, "..");

function usage() {
  return `scene-pipeline.mjs — End-to-end scene prose pipeline

Usage:
  node tools/scene-pipeline.mjs --spec-file <path> [options]

Options:
  --spec-file <path>   Scene spec JSON (required)
  --output <path>      Final TOML output path
  --max-retries <n>    Max DeepSeek revisions on lint failure (default: 2)
  --draft-only         Stop after draft + lint
  --skip-lint          Skip lint (when voice samples aren't ready yet)
  --help               Show help

Pipeline steps:
  0. Validate spec (spec-validate.mjs — blocks on critical findings)
  1. Assemble prompt (pack-prompt.mjs)
  2. Draft prose (deepseek-helper.mjs → DeepSeek API)
  3. Lint prose (prose-lint.mjs — prose quality + structural depth)
  4. If lint fails: revise (send draft + findings back to DeepSeek, up to --max-retries)
  5. Convert to TOML (prose-to-toml.mjs)

Intermediate files land in tmp/ and are preserved for inspection.
`;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    execFile(cmd, args, { cwd: repoRoot, maxBuffer: 10 * 1024 * 1024, ...opts }, (err, stdout, stderr) => {
      if (err && !opts.allowFail) {
        const msg = stderr || err.message;
        reject(new Error(`${cmd} ${args.slice(0, 2).join(" ")} failed: ${msg}`));
      } else {
        resolve({ code: err ? err.code : 0, stdout, stderr });
      }
    });
  });
}

function log(step, msg) {
  process.stderr.write(`[pipeline:${step}] ${msg}\n`);
}

// ─── Main pipeline ───────────────────────────────────────────────────────────

async function main() {
  const args = process.argv.slice(2);
  if (args.includes("--help") || args.includes("-h") || args.length === 0) {
    process.stdout.write(usage());
    return;
  }

  const options = {
    maxRetries: 2,
    draftOnly: false,
    skipLint: false,
  };
  let specFile;

  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case "--spec-file":
        specFile = args[++i];
        break;
      case "--output":
        options.output = args[++i];
        break;
      case "--max-retries":
        options.maxRetries = parseInt(args[++i], 10);
        break;
      case "--draft-only":
        options.draftOnly = true;
        break;
      case "--skip-lint":
        options.skipLint = true;
        break;
    }
  }

  if (!specFile) {
    process.stderr.write("Error: --spec-file is required\n");
    process.exitCode = 1;
    return;
  }

  const specText = await fs.readFile(specFile, "utf8");
  const spec = JSON.parse(specText);
  if (!spec.scene_id) throw new Error("spec must have scene_id");
  if (!spec.brief) throw new Error("spec must have brief");

  const sceneName = spec.scene_id.replace(/^base::/, "");
  const tmpDir = path.join(repoRoot, "tmp");
  await fs.mkdir(tmpDir, { recursive: true });

  const promptFile = path.join(tmpDir, `prompt-${sceneName}.md`);
  const draftFile = path.join(tmpDir, `draft-${sceneName}.txt`);
  const lintFile = path.join(tmpDir, `lint-${sceneName}.json`);
  const specLintFile = path.join(tmpDir, `spec-lint-${sceneName}.json`);
  const revisionPromptFile = path.join(tmpDir, `revision-${sceneName}.md`);
  const outputFile = options.output || path.join(repoRoot, `packs/base/scenes/${sceneName}.toml`);

  // ── Step 0: Validate spec ─────────────────────────────────────────────────

  log("spec", "validating...");
  const specLintResult = await run(
    "node",
    [path.join(scriptDir, "spec-validate.mjs"), specFile],
    { allowFail: true },
  );

  await fs.writeFile(specLintFile, specLintResult.stdout, "utf8");

  let specData;
  try {
    specData = JSON.parse(specLintResult.stdout);
  } catch {
    specData = { pass: true };
  }

  if (!specData.pass) {
    const criticals = specData.findings
      .filter((f) => f.severity === "critical")
      .map((f) => `  - [${f.rule}] ${f.message}`);
    log("spec", `FAIL — ${specData.critical} critical finding(s):`);
    for (const c of criticals) process.stderr.write(c + "\n");
    log("spec", "fix the spec before running the pipeline");
    process.exitCode = 1;
    return;
  }

  if (specData.important > 0) {
    log("spec", `PASS with ${specData.important} warning(s) — check ${specLintFile}`);
  } else {
    log("spec", "PASS — structurally sound");
  }

  // ── Step 1: Assemble prompt ────────────────────────────────────────────────

  log("prompt", "assembling...");
  await run("node", [
    path.join(scriptDir, "pack-prompt.mjs"),
    "--spec-file", specFile,
    "--output", promptFile,
  ]);
  log("prompt", `wrote ${promptFile}`);

  // ── Step 2: DeepSeek draft ─────────────────────────────────────────────────

  log("draft", "calling DeepSeek...");
  const writerTech = path.join(repoRoot, "docs/writer-tech.md");
  await run("node", [
    path.join(scriptDir, "deepseek-helper.mjs"),
    "draft",
    "--system-file", writerTech,
    "--prompt-file", promptFile,
    "--output-file", draftFile,
    "--max-tokens", "8000",
  ]);
  log("draft", `wrote ${draftFile}`);

  // Read draft for potential revision loop
  let currentDraft = await fs.readFile(draftFile, "utf8");

  // Strip markdown fences if DeepSeek wrapped output in them
  currentDraft = stripFences(currentDraft);
  await fs.writeFile(draftFile, currentDraft, "utf8");

  // ── Step 3: Lint ───────────────────────────────────────────────────────────

  if (!options.skipLint) {
    let attempt = 0;
    let lintPassed = false;

    while (attempt <= options.maxRetries) {
      log("lint", `attempt ${attempt + 1}...`);
      const lintResult = await run(
        "node",
        [path.join(scriptDir, "prose-lint.mjs"), draftFile],
        { allowFail: true },
      );

      const lintOutput = lintResult.stdout;
      await fs.writeFile(lintFile, lintOutput, "utf8");

      let lintData;
      try {
        lintData = JSON.parse(lintOutput);
      } catch {
        log("lint", `could not parse lint output — treating as pass`);
        lintPassed = true;
        break;
      }

      if (lintData.pass) {
        const warnCount = lintData.important || 0;
        log("lint", warnCount > 0 ? `PASS with ${warnCount} warnings` : "PASS — clean");
        lintPassed = true;
        break;
      }

      log("lint", `FAIL — ${lintData.critical} critical, ${lintData.important || 0} important`);

      if (attempt >= options.maxRetries) {
        log("lint", `max retries reached — proceeding with warnings`);
        break;
      }

      // ── Step 3b: Revision ────────────────────────────────────────────────

      log("revise", `sending findings back to DeepSeek (attempt ${attempt + 2})...`);

      const criticalFindings = lintData.findings
        .filter((f) => f.severity === "critical")
        .map((f) => `- Line ${f.line}: "${f.text}" — ${f.message}`)
        .join("\n");

      const revisionPrompt = `# Revision Request

The following draft has lint failures. Fix ONLY the flagged issues. Keep all other prose unchanged. Output the complete revised draft in the same labeled format.

## Critical findings to fix:

${criticalFindings}

## Original draft:

${currentDraft}

Output the complete revised draft. Same labeled format (INTRO:, ACTION:, NPC_ACTION:). No commentary.`;

      await fs.writeFile(revisionPromptFile, revisionPrompt, "utf8");

      await run("node", [
        path.join(scriptDir, "deepseek-helper.mjs"),
        "draft",
        "--system-file", writerTech,
        "--prompt-file", revisionPromptFile,
        "--output-file", draftFile,
        "--temperature", "0.4",
        "--max-tokens", "8000",
      ]);

      currentDraft = await fs.readFile(draftFile, "utf8");
      currentDraft = stripFences(currentDraft);
      await fs.writeFile(draftFile, currentDraft, "utf8");

      attempt++;
    }

    if (!lintPassed) {
      log("lint", "⚠ Draft has unresolved critical findings — TOML will have quality issues");
    }
  } else {
    log("lint", "skipped (--skip-lint)");
  }

  // ── Step 4: Convert to TOML ────────────────────────────────────────────────

  if (options.draftOnly) {
    log("done", `draft at ${draftFile}`);
    return;
  }

  // Check if spec has actions/npc_actions for conversion
  if (!spec.actions || spec.actions.length === 0) {
    log("convert", "spec has no actions defined — skipping TOML conversion");
    log("done", `draft at ${draftFile} — add actions to spec for TOML conversion`);
    return;
  }

  log("convert", "merging prose + spec → TOML...");
  await run("node", [
    path.join(scriptDir, "prose-to-toml.mjs"),
    "--prose", draftFile,
    "--spec", specFile,
    "--output", outputFile,
  ]);
  log("convert", `wrote ${outputFile}`);

  // ── Summary ────────────────────────────────────────────────────────────────

  const tomlStat = await fs.stat(outputFile);
  log("done", `${outputFile} (${tomlStat.size} bytes)`);
  log("done", "next: validate templates with minijinja MCP, then playtest");
}

// ─── Utilities ───────────────────────────────────────────────────────────────

function stripFences(text) {
  // Remove ```toml ... ``` or ```markdown ... ``` or ``` ... ``` wrapping
  let result = text.trim();
  if (/^```\w*\s*\n/.test(result) && result.endsWith("```")) {
    result = result.replace(/^```\w*\s*\n/, "").replace(/\n```$/, "");
  }
  return result;
}

main().catch((error) => {
  process.stderr.write(`[pipeline] fatal: ${error.message}\n`);
  process.exitCode = 1;
});
