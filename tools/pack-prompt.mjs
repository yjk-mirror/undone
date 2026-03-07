#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const ROUTE_DOCS = {
  workplace: {
    preset: "docs/presets/robin.md",
    arc: "docs/arcs/workplace-opening.md",
  },
  campus: {
    preset: "docs/presets/camila.md",
    arc: "docs/arcs/campus-opening.md",
  },
};

const TOKEN_WARN_BYTES = 48_000; // ~12K tokens

function usage() {
  return `Usage:
  node tools/pack-prompt.mjs [options]

Options:
  --spec-file <path>   JSON scene spec file (or read from stdin)
  --output <path>      Output path (default: tmp/prompt-<scene_id>.md)
  --help               Show this message

Scene spec JSON fields:
  scene_id (required)  e.g. "base::workplace_coffee_break"
  brief (required)     Creative direction for this scene
  route                "workplace" or "campus" — pulls preset + arc docs
  arc_state            Current arc state for context
  slot                 Schedule slot name
  traits               Array of trait IDs to branch on
  femininity_range     e.g. "10-30"
  content_level        "VANILLA", "SEXUAL", "ROUGH", "DUBCON", "NONCON"
  reference_scenes     Array of scene names (without base:: prefix) for voice calibration
`;
}

function parseArgs(argv) {
  const options = {};
  const args = [...argv];
  while (args.length > 0) {
    const arg = args.shift();
    switch (arg) {
      case "--spec-file":
        options.specFile = args.shift();
        break;
      case "--output":
        options.output = args.shift();
        break;
      case "--help":
      case "-h":
        options.help = true;
        break;
      default:
        throw new Error(`unknown argument: ${arg}`);
    }
  }
  return options;
}

async function readStdin() {
  if (process.stdin.isTTY) return "";
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

async function readFileSafe(filePath) {
  try {
    return await fs.readFile(filePath, "utf8");
  } catch (e) {
    if (e.code === "ENOENT") {
      process.stderr.write(`[pack-prompt] warning: ${filePath} not found, skipping\n`);
      return null;
    }
    throw e;
  }
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  if (options.help) {
    process.stdout.write(usage());
    return;
  }

  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const repoRoot = path.resolve(scriptDir, "..");

  const specSrc = options.specFile
    ? await fs.readFile(options.specFile, "utf8")
    : await readStdin();
  if (!specSrc.trim()) throw new Error("spec is empty");

  const spec = JSON.parse(specSrc);
  if (!spec.scene_id) throw new Error("scene_id is required");
  if (!spec.brief) throw new Error("brief is required");

  const parts = [];

  // 1. Writer core (always)
  const writerCore = await readFileSafe(path.join(repoRoot, "docs/writer-core.md"));
  if (writerCore) {
    parts.push("# Writing Rules\n\n" + writerCore.trim());
  } else {
    throw new Error("docs/writer-core.md is required but not found");
  }

  // 2. Route-specific docs
  if (spec.route && ROUTE_DOCS[spec.route]) {
    const routeInfo = ROUTE_DOCS[spec.route];
    const preset = await readFileSafe(path.join(repoRoot, routeInfo.preset));
    if (preset) parts.push("---\n\n# Preset\n\n" + preset.trim());
    const arc = await readFileSafe(path.join(repoRoot, routeInfo.arc));
    if (arc) parts.push("---\n\n# Arc\n\n" + arc.trim());
  }

  // 3. Reference scenes
  if (spec.reference_scenes && spec.reference_scenes.length > 0) {
    for (const sceneName of spec.reference_scenes) {
      const scenePath = path.join(repoRoot, `packs/base/scenes/${sceneName}.toml`);
      const sceneContent = await readFileSafe(scenePath);
      if (sceneContent) {
        parts.push(`---\n\n# Reference Scene: ${sceneName}\n\n\`\`\`toml\n${sceneContent.trim()}\n\`\`\``);
      }
    }
  }

  // 4. Task payload
  const taskLines = [`---\n\n# Task\n\nWrite a complete scene TOML file for \`${spec.scene_id}\`.`];
  if (spec.slot) taskLines.push(`**Slot:** ${spec.slot}`);
  if (spec.arc_state) taskLines.push(`**Arc state:** ${spec.arc_state}`);
  if (spec.femininity_range) taskLines.push(`**FEMININITY range:** ${spec.femininity_range}`);
  if (spec.content_level) taskLines.push(`**Content level:** ${spec.content_level}`);
  if (spec.traits && spec.traits.length > 0) {
    taskLines.push(`**Key traits to branch on:** ${spec.traits.join(", ")}`);
  }
  taskLines.push(`\n**Scene brief:**\n${spec.brief}`);
  taskLines.push(`\nOutput ONLY the complete TOML file. No commentary, no markdown fences, no explanation.`);
  parts.push(taskLines.join("\n"));

  const assembled = parts.join("\n\n");

  // Output
  const sceneIdSuffix = spec.scene_id.replace(/^base::/, "");
  const outputPath = options.output || path.join(repoRoot, `tmp/prompt-${sceneIdSuffix}.md`);

  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, assembled + "\n", "utf8");

  const bytes = Buffer.byteLength(assembled, "utf8");
  const estTokens = Math.round(bytes / 4);
  const warn = bytes > TOKEN_WARN_BYTES ? " ⚠ OVER BUDGET" : "";
  process.stderr.write(
    `[pack-prompt] ${outputPath} — ${bytes} bytes (~${estTokens} tokens)${warn}\n`
  );
}

main().catch((error) => {
  process.stderr.write(`[pack-prompt] ${error.message}\n`);
  process.exitCode = 1;
});
