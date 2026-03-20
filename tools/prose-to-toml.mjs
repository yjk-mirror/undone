#!/usr/bin/env node

/**
 * prose-to-toml.mjs — Converts DeepSeek labeled prose + spec JSON → scene TOML.
 *
 * DeepSeek writes prose in a simple labeled format (INTRO:, ACTION: id, etc.).
 * The spec JSON provides all structural data (effects, conditions, labels).
 * This tool merges them into valid scene TOML.
 *
 * Usage:
 *   node tools/prose-to-toml.mjs --prose draft.txt --spec spec.json [--output scene.toml]
 *   node tools/prose-to-toml.mjs --test
 */

import fs from "node:fs/promises";
import process from "node:process";

// ─── Parse DeepSeek labeled output ───────────────────────────────────────────

const SECTION_HEADER = /^(INTRO|ACTION|NPC_ACTION):\s*(.*)/;

function parseDraft(text) {
  const sections = new Map(); // key: "INTRO" | "ACTION:id" | "NPC_ACTION:id"
  let currentKey = null;
  const lines = [];
  let currentLines = [];

  for (const line of text.split(/\r?\n/)) {
    const match = line.match(SECTION_HEADER);
    if (match) {
      if (currentKey !== null) {
        sections.set(currentKey, joinProse(currentLines));
      }
      const type = match[1];
      const id = match[2].trim();
      currentKey = type === "INTRO" ? "INTRO" : `${type}:${id}`;
      currentLines = [];
    } else if (currentKey !== null) {
      currentLines.push(line);
    }
  }
  if (currentKey !== null) {
    sections.set(currentKey, joinProse(currentLines));
  }

  return sections;
}

function joinProse(lines) {
  // Trim leading/trailing blank lines, preserve internal structure
  const text = lines.join("\n");
  return text.replace(/^\n+/, "").replace(/\n+$/, "");
}

// ─── TOML serialization ─────────────────────────────────────────────────────

function escapeTomlString(s) {
  return s
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\t/g, "\\t");
}

function toTomlValue(v) {
  if (typeof v === "string") return `"${escapeTomlString(v)}"`;
  if (typeof v === "number") return String(v);
  if (typeof v === "boolean") return v ? "true" : "false";
  throw new Error(`unsupported TOML value type: ${typeof v}`);
}

function toTomlMultiline(text) {
  // Escape literal triple-quotes inside the text
  const escaped = text.replace(/\\\\/g, "\\\\").replace(/"""/g, '""\\"');
  return `"""\n${escaped}\n"""`;
}

function writeKeyValue(key, value, indent = "") {
  if (typeof value === "string" && value.includes("\n")) {
    return `${indent}${key} = ${toTomlMultiline(value)}`;
  }
  return `${indent}${key} = ${toTomlValue(value)}`;
}

// ─── Generate scene TOML ─────────────────────────────────────────────────────

function generateToml(spec, proseMap) {
  const lines = [];
  const warnings = [];

  // [scene] table
  lines.push("[scene]");
  lines.push(writeKeyValue("id", spec.scene_id));
  lines.push(writeKeyValue("pack", spec.pack || spec.scene_id.split("::")[0]));
  lines.push(writeKeyValue("description", spec.description || ""));
  lines.push("");

  // [intro] table
  const introProse = proseMap.get("INTRO");
  if (introProse) {
    lines.push("[intro]");
    lines.push(writeKeyValue("prose", introProse));
    lines.push("");
  } else {
    warnings.push("No INTRO section found in draft");
    lines.push("[intro]");
    lines.push(writeKeyValue("prose", "TODO: intro prose missing from draft"));
    lines.push("");
  }

  // [[intro_variants]] if spec has them
  if (spec.intro_variants && spec.intro_variants.length > 0) {
    for (const variant of spec.intro_variants) {
      lines.push("[[intro_variants]]");
      if (variant.condition) {
        lines.push(writeKeyValue("condition", variant.condition));
      }
      const key = `INTRO_VARIANT:${variant.id || variant.condition || ""}`;
      const prose = proseMap.get(key) || variant.prose || "TODO";
      lines.push(writeKeyValue("prose", prose));
      lines.push("");
    }
  }

  // [[thoughts]] if spec has them
  if (spec.thoughts && spec.thoughts.length > 0) {
    for (const thought of spec.thoughts) {
      lines.push("[[thoughts]]");
      lines.push(writeKeyValue("id", thought.id));
      if (thought.condition) {
        lines.push(writeKeyValue("condition", thought.condition));
      }
      const key = `THOUGHT:${thought.id}`;
      const prose = proseMap.get(key) || thought.prose || "TODO";
      lines.push(writeKeyValue("prose", prose));
      lines.push("");
    }
  }

  // [[actions]]
  if (spec.actions) {
    for (const action of spec.actions) {
      lines.push("[[actions]]");
      lines.push(writeKeyValue("id", action.id));
      lines.push(writeKeyValue("label", action.label));
      if (action.detail) {
        lines.push(writeKeyValue("detail", action.detail));
      }
      if (action.condition) {
        lines.push(writeKeyValue("condition", action.condition));
      }
      if (action.allow_npc_actions) {
        lines.push(writeKeyValue("allow_npc_actions", true));
      }

      const proseKey = `ACTION:${action.id}`;
      const actionProse = proseMap.get(proseKey);
      if (actionProse) {
        lines.push(writeKeyValue("prose", actionProse));
      } else if (action.prose) {
        // Action has prose in the spec (rare — for simple actions)
        lines.push(writeKeyValue("prose", action.prose));
      } else {
        warnings.push(`No prose found for action "${action.id}"`);
        lines.push(writeKeyValue("prose", `TODO: prose for ${action.id}`));
      }
      lines.push("");

      // [[actions.effects]]
      if (action.effects && action.effects.length > 0) {
        for (const effect of action.effects) {
          lines.push("  [[actions.effects]]");
          for (const [k, v] of Object.entries(effect)) {
            if (v !== null && v !== undefined) {
              lines.push(writeKeyValue(k, v, "  "));
            }
          }
          lines.push("");
        }
      }

      // [[actions.next]]
      if (action.next) {
        if (Array.isArray(action.next)) {
          for (const n of action.next) {
            lines.push("  [[actions.next]]");
            for (const [k, v] of Object.entries(n)) {
              if (v !== null && v !== undefined) {
                lines.push(writeKeyValue(k, v, "  "));
              }
            }
            lines.push("");
          }
        } else {
          lines.push("  [[actions.next]]");
          for (const [k, v] of Object.entries(action.next)) {
            if (v !== null && v !== undefined) {
              lines.push(writeKeyValue(k, v, "  "));
            }
          }
          lines.push("");
        }
      }
    }
  }

  // [[npc_actions]]
  if (spec.npc_actions) {
    for (const npcAction of spec.npc_actions) {
      lines.push("[[npc_actions]]");
      lines.push(writeKeyValue("id", npcAction.id));
      if (npcAction.condition) {
        lines.push(writeKeyValue("condition", npcAction.condition));
      }
      if (npcAction.weight !== undefined) {
        lines.push(writeKeyValue("weight", npcAction.weight));
      }

      const proseKey = `NPC_ACTION:${npcAction.id}`;
      const npcProse = proseMap.get(proseKey);
      if (npcProse) {
        lines.push(writeKeyValue("prose", npcProse));
      } else if (npcAction.prose) {
        lines.push(writeKeyValue("prose", npcAction.prose));
      } else {
        warnings.push(`No prose found for npc_action "${npcAction.id}"`);
        lines.push(writeKeyValue("prose", `TODO: prose for ${npcAction.id}`));
      }
      lines.push("");

      // [[npc_actions.effects]]
      if (npcAction.effects && npcAction.effects.length > 0) {
        for (const effect of npcAction.effects) {
          lines.push("  [[npc_actions.effects]]");
          for (const [k, v] of Object.entries(effect)) {
            if (v !== null && v !== undefined) {
              lines.push(writeKeyValue(k, v, "  "));
            }
          }
          lines.push("");
        }
      }
    }
  }

  return { toml: lines.join("\n") + "\n", warnings };
}

// ─── Self-test ───────────────────────────────────────────────────────────────

function selfTest() {
  const draft = `INTRO:
The bar is half-empty on a Tuesday.

{% if w.hasTrait("BEAUTIFUL") %}
One of the guys at the end has stopped watching the game.
{% endif %}

The coaster is waiting.

ACTION: order_drink
"Gin and tonic," you say.

She makes the drink.

ACTION: leave
You step back outside.

NPC_ACTION: offer_drink
He leans in. "Can I buy you another?"
`;

  const spec = {
    scene_id: "base::test_bar",
    pack: "base",
    description: "A test scene.",
    actions: [
      {
        id: "order_drink",
        label: "Order a drink",
        detail: "Get something to hold.",
        allow_npc_actions: true,
        effects: [
          { type: "set_scene_flag", flag: "has_drink" },
          { type: "change_alcohol", amount: 1 },
        ],
      },
      {
        id: "leave",
        label: "Leave",
        detail: "Head out.",
        next: { finish: true },
      },
    ],
    npc_actions: [
      {
        id: "offer_drink",
        condition: "scene.hasFlag('has_drink')",
        weight: 15,
        effects: [{ type: "set_scene_flag", flag: "offer_made" }],
      },
    ],
  };

  const proseMap = parseDraft(draft);
  const { toml, warnings } = generateToml(spec, proseMap);

  let ok = true;

  // Check sections exist
  if (!proseMap.has("INTRO")) {
    process.stderr.write("FAIL: INTRO not parsed\n");
    ok = false;
  }
  if (!proseMap.has("ACTION:order_drink")) {
    process.stderr.write("FAIL: ACTION:order_drink not parsed\n");
    ok = false;
  }
  if (!proseMap.has("NPC_ACTION:offer_drink")) {
    process.stderr.write("FAIL: NPC_ACTION:offer_drink not parsed\n");
    ok = false;
  }

  // Check TOML output structure
  if (!toml.includes('[scene]')) {
    process.stderr.write("FAIL: missing [scene]\n");
    ok = false;
  }
  if (!toml.includes('[intro]')) {
    process.stderr.write("FAIL: missing [intro]\n");
    ok = false;
  }
  if (!toml.includes('[[actions]]')) {
    process.stderr.write("FAIL: missing [[actions]]\n");
    ok = false;
  }
  if (!toml.includes('[[npc_actions]]')) {
    process.stderr.write("FAIL: missing [[npc_actions]]\n");
    ok = false;
  }
  if (!toml.includes('id = "base::test_bar"')) {
    process.stderr.write("FAIL: missing scene id\n");
    ok = false;
  }
  if (!toml.includes('type = "set_scene_flag"')) {
    process.stderr.write("FAIL: missing effect\n");
    ok = false;
  }
  if (!toml.includes('finish = true')) {
    process.stderr.write("FAIL: missing next finish\n");
    ok = false;
  }
  if (!toml.includes("The bar is half-empty")) {
    process.stderr.write("FAIL: intro prose not in output\n");
    ok = false;
  }
  if (!toml.includes("Gin and tonic")) {
    process.stderr.write("FAIL: action prose not in output\n");
    ok = false;
  }
  if (warnings.length > 0) {
    process.stderr.write(`FAIL: unexpected warnings: ${warnings.join(", ")}\n`);
    ok = false;
  }

  process.stderr.write(`prose-to-toml self-test: ${ok ? "PASSED" : "FAILED"}\n`);
  return ok;
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

function usage() {
  return `Usage:
  node tools/prose-to-toml.mjs --prose <draft.txt> --spec <spec.json> [--output <scene.toml>]
  node tools/prose-to-toml.mjs --test

Merges DeepSeek labeled prose output with a scene spec JSON to produce valid scene TOML.

Options:
  --prose <path>    DeepSeek draft output (labeled sections format)
  --spec <path>     Scene spec JSON (actions, effects, conditions)
  --output <path>   Output TOML path (default: stdout)
  --test            Run self-tests
`;
}

async function main() {
  const args = process.argv.slice(2);

  if (args.includes("--test")) {
    const ok = selfTest();
    process.exitCode = ok ? 0 : 1;
    return;
  }

  if (args.includes("--help") || args.includes("-h")) {
    process.stdout.write(usage());
    return;
  }

  const options = {};
  for (let i = 0; i < args.length; i++) {
    switch (args[i]) {
      case "--prose":
        options.prose = args[++i];
        break;
      case "--spec":
        options.spec = args[++i];
        break;
      case "--output":
        options.output = args[++i];
        break;
    }
  }

  if (!options.prose || !options.spec) {
    process.stderr.write("Error: --prose and --spec are required\n\n");
    process.stdout.write(usage());
    process.exitCode = 1;
    return;
  }

  const [draftText, specText] = await Promise.all([
    fs.readFile(options.prose, "utf8"),
    fs.readFile(options.spec, "utf8"),
  ]);

  const spec = JSON.parse(specText);
  if (!spec.scene_id) {
    throw new Error("spec must have scene_id");
  }

  const proseMap = parseDraft(draftText);
  const { toml, warnings } = generateToml(spec, proseMap);

  for (const w of warnings) {
    process.stderr.write(`[prose-to-toml] warning: ${w}\n`);
  }

  if (options.output) {
    await fs.writeFile(options.output, toml, "utf8");
    process.stderr.write(`[prose-to-toml] wrote ${options.output}\n`);
  } else {
    process.stdout.write(toml);
  }
}

main().catch((error) => {
  process.stderr.write(`[prose-to-toml] ${error.message}\n`);
  process.exitCode = 1;
});
