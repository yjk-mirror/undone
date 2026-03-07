#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const DEFAULTS = {
  draft: {
    model: "deepseek-chat",
    temperature: 0.8,
    system:
      "You are a subordinate drafting model for a text-game writing workflow. Produce only the requested draft or outline. Do not add meta commentary.",
  },
  review: {
    model: "deepseek-chat",
    temperature: 0.2,
    system:
      "You are a subordinate review model for a text-game writing workflow. Return concrete findings grouped by severity. Do not soften criticism and do not rewrite unless explicitly asked.",
  },
};

function usage() {
  return `Usage:
  node tools/deepseek-helper.mjs <draft|review> [options]

Options:
  --prompt-file <path>   Read the user prompt from a file instead of stdin
  --system-file <path>   Read the system prompt from a file
  --model <name>         Override the DeepSeek model (default: deepseek-chat)
  --temperature <num>    Override temperature
  --max-tokens <num>     Set max_tokens on the API request
  --output-file <path>   Write the response to a file instead of stdout
  --json                 Emit JSON with content/model/usage instead of plain text
  --help                 Show this message

Examples:
  node tools/deepseek-helper.mjs draft --prompt-file tmp/spec.md
  node tools/deepseek-helper.mjs review --prompt-file tmp/scene.md --json
  Get-Content tmp/spec.md | node tools/deepseek-helper.mjs draft
`;
}

function parseArgs(argv) {
  const args = [...argv];
  let mode;
  // Support top-level help without requiring a mode first.
  if (args[0] !== "--help" && args[0] !== "-h") {
    mode = args.shift();
  }
  const options = {
    json: false,
  };

  while (args.length > 0) {
    const arg = args.shift();
    switch (arg) {
      case "--prompt-file":
        options.promptFile = args.shift();
        break;
      case "--system-file":
        options.systemFile = args.shift();
        break;
      case "--model":
        options.model = args.shift();
        break;
      case "--temperature":
        options.temperature = args.shift();
        break;
      case "--max-tokens":
        options.maxTokens = args.shift();
        break;
      case "--output-file":
        options.outputFile = args.shift();
        break;
      case "--json":
        options.json = true;
        break;
      case "--help":
      case "-h":
        options.help = true;
        break;
      default:
        throw new Error(`unknown argument: ${arg}`);
    }
  }

  return { mode, options };
}

function parseDotEnv(src) {
  const values = new Map();
  for (const rawLine of src.split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    const eq = line.indexOf("=");
    if (eq === -1) {
      continue;
    }
    const key = line.slice(0, eq).trim();
    let value = line.slice(eq + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }
    values.set(key, value);
  }
  return values;
}

async function loadApiKey(repoRoot) {
  if (process.env.DEEPSEEK_API_KEY) {
    return process.env.DEEPSEEK_API_KEY;
  }

  try {
    const envPath = path.join(repoRoot, ".env");
    const envSrc = await fs.readFile(envPath, "utf8");
    const envValues = parseDotEnv(envSrc);
    const apiKey = envValues.get("DEEPSEEK_API_KEY");
    if (apiKey) {
      return apiKey;
    }
  } catch (error) {
    if (error.code !== "ENOENT") {
      throw error;
    }
  }

  throw new Error(
    "DEEPSEEK_API_KEY is not set in the environment or repo-local .env",
  );
}

async function readMaybeFile(filePath) {
  return fs.readFile(filePath, "utf8");
}

async function readStdin() {
  if (process.stdin.isTTY) {
    return "";
  }

  const chunks = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks).toString("utf8");
}

function parseNumber(value, label) {
  if (value == null) {
    return undefined;
  }
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    throw new Error(`${label} must be numeric`);
  }
  return parsed;
}

function truncate(text, maxLen) {
  return text.length <= maxLen ? text : `${text.slice(0, maxLen)}...`;
}

async function main() {
  const { mode, options } = parseArgs(process.argv.slice(2));
  if (options.help || !mode) {
    process.stdout.write(usage());
    return;
  }
  if (!(mode in DEFAULTS)) {
    throw new Error(`mode must be one of: ${Object.keys(DEFAULTS).join(", ")}`);
  }

  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const repoRoot = path.resolve(scriptDir, "..");
  const defaults = DEFAULTS[mode];

  const prompt =
    options.promptFile != null
      ? await readMaybeFile(options.promptFile)
      : await readStdin();
  if (!prompt.trim()) {
    throw new Error("prompt content is empty");
  }

  const system =
    options.systemFile != null
      ? await readMaybeFile(options.systemFile)
      : defaults.system;

  const apiKey = await loadApiKey(repoRoot);
  const body = {
    model: options.model ?? defaults.model,
    messages: [
      { role: "system", content: system },
      { role: "user", content: prompt },
    ],
    temperature:
      parseNumber(options.temperature, "temperature") ?? defaults.temperature,
  };

  const maxTokens = parseNumber(options.maxTokens, "max-tokens");
  if (maxTokens != null) {
    body.max_tokens = maxTokens;
  }

  const response = await fetch("https://api.deepseek.com/chat/completions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(
      `DeepSeek request failed (${response.status}): ${truncate(errorText, 500)}`,
    );
  }

  const payload = await response.json();
  const content = payload?.choices?.[0]?.message?.content;
  if (typeof content !== "string" || content.trim().length === 0) {
    throw new Error("DeepSeek response did not contain assistant text");
  }

  const output = options.json
    ? JSON.stringify(
        {
          mode,
          model: payload.model ?? body.model,
          usage: payload.usage ?? null,
          content,
        },
        null,
        2,
      )
    : `${content.trimEnd()}\n`;

  if (options.outputFile) {
    await fs.writeFile(options.outputFile, output, "utf8");
  } else {
    process.stdout.write(output);
  }

  if (payload.usage) {
    const parts = [
      `model=${payload.model ?? body.model}`,
      `prompt_tokens=${payload.usage.prompt_tokens ?? "?"}`,
      `completion_tokens=${payload.usage.completion_tokens ?? "?"}`,
    ];
    if (payload.usage.prompt_cache_hit_tokens != null) {
      parts.push(`cache_hit=${payload.usage.prompt_cache_hit_tokens}`);
    }
    if (payload.usage.prompt_cache_miss_tokens != null) {
      parts.push(`cache_miss=${payload.usage.prompt_cache_miss_tokens}`);
    }
    process.stderr.write(`[deepseek-helper] ${parts.join(" ")}\n`);
  }
}

main().catch((error) => {
  process.stderr.write(`[deepseek-helper] ${error.message}\n`);
  process.exitCode = 1;
});
