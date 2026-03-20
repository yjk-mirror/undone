#!/usr/bin/env node

/**
 * spec-validate.mjs — Validates scene spec JSON for structural depth.
 *
 * Catches design problems BEFORE sending to DeepSeek:
 *   - Missing effects, no persisting effects
 *   - Too few actions, no conditional actions
 *   - Missing NPC actions, thin briefs
 *   - Missing trait axes
 *
 * Usage:
 *   node tools/spec-validate.mjs <spec.json>
 *   node tools/spec-validate.mjs --test
 */

import fs from "node:fs/promises";
import process from "node:process";

// ─── Effect classification ───────────────────────────────────────────────────

const SCENE_ONLY_EFFECTS = new Set(["set_scene_flag", "remove_scene_flag"]);

const PERSISTING_EFFECTS = new Set([
  "set_game_flag", "remove_game_flag",
  "change_stress", "change_money", "change_anxiety",
  "add_arousal", "change_alcohol",
  "add_stat", "set_stat", "skill_increase",
  "add_trait", "remove_trait",
  "set_virgin", "set_player_partner", "add_player_friend",
  "set_job_title", "add_stuff", "remove_stuff",
  "add_npc_liking", "add_npc_love", "add_w_liking",
  "set_npc_flag", "add_npc_trait", "set_relationship",
  "set_npc_attraction", "set_npc_behaviour", "set_contactable",
  "add_sexual_activity", "set_npc_role",
  "advance_arc", "advance_time",
]);

// ─── Validation ──────────────────────────────────────────────────────────────

function validate(spec) {
  const findings = [];

  function add(severity, rule, message) {
    findings.push({ severity, rule, message });
  }

  // ── Required fields ────────────────────────────────────────────────────────

  if (!spec.scene_id) {
    add("critical", "missing_scene_id", "scene_id is required");
  }

  if (!spec.brief) {
    add("critical", "missing_brief", "brief is required — what is this scene ABOUT?");
  } else if (spec.brief.length < 80) {
    add("critical", "thin_brief",
      `brief is ${spec.brief.length} chars — needs specific creative direction (min 80). ` +
      "What happens? Who's there? What traits matter? What's at stake?");
  }

  if (!spec.description) {
    add("important", "missing_description", "no description — add a one-line setting");
  }

  // ── Trait axes ─────────────────────────────────────────────────────────────

  if (!spec.traits || spec.traits.length === 0) {
    add("critical", "no_traits",
      "no trait axes — DeepSeek won't know what to branch on. " +
      "Pick 2-4 traits that genuinely change what happens in this scene.");
  } else if (spec.traits.length === 1) {
    add("important", "single_trait",
      "only 1 trait axis — commit to 2-4 for depth. " +
      "Different traits should open different paths, not just color the same path.");
  } else if (spec.traits.length > 6) {
    add("important", "too_many_traits",
      `${spec.traits.length} trait axes is too many — pick 2-4 that matter HERE. ` +
      "Going deep on a few > going shallow on many.");
  }

  // ── Actions ────────────────────────────────────────────────────────────────

  const actions = spec.actions || [];

  if (actions.length === 0) {
    add("critical", "no_actions", "scene has no actions — players need choices");
    return buildResult(findings);
  }

  if (actions.length < 2) {
    add("critical", "single_action",
      "only 1 action — a scene with one choice is not a choice. Add at least one alternative.");
  }

  // Per-action checks
  let actionsWithEffects = 0;
  let actionsWithPersisting = 0;
  let actionsWithCondition = 0;
  let actionsWithTransition = 0;
  let actionsWithNpcAllowed = 0;
  const allPersistingTypes = new Set();
  const effectlessActions = [];

  for (const action of actions) {
    if (!action.id) {
      add("critical", "action_missing_id", "an action is missing its id");
      continue;
    }
    if (!action.label) {
      add("critical", "action_missing_label", `action "${action.id}" has no label`);
    }
    if (!action.detail) {
      add("important", "action_missing_detail",
        `action "${action.id}" has no detail text — players need context for each choice`);
    }

    // Effects
    const effects = action.effects || [];
    if (effects.length === 0) {
      effectlessActions.push(action.id);
    } else {
      actionsWithEffects++;
      for (const effect of effects) {
        if (PERSISTING_EFFECTS.has(effect.type)) {
          allPersistingTypes.add(effect.type);
          actionsWithPersisting++;
          break; // count action once
        }
      }
    }

    // Condition
    if (action.condition) actionsWithCondition++;

    // Next
    if (action.next) {
      const nextObj = Array.isArray(action.next) ? action.next[0] : action.next;
      if (nextObj && nextObj.scene) actionsWithTransition++;
    }

    // NPC
    if (action.allow_npc_actions) actionsWithNpcAllowed++;
  }

  // Effectless actions
  if (effectlessActions.length > 0) {
    add("critical", "effectless_actions",
      `${effectlessActions.length} action(s) have no effects: [${effectlessActions.join(", ")}]. ` +
      "Every choice must have consequences — at minimum a flag or stat change.");
  }

  // No persisting effects anywhere
  if (actionsWithPersisting === 0 && actions.length > 0) {
    add("critical", "no_persisting_effects",
      "no action has a persisting effect (game_flag, stat, npc_liking, trait, skill, arc). " +
      "The scene has no lasting impact — nothing carries forward to future scenes. " +
      "Add at least one game_flag, stat change, or npc_liking effect.");
  }

  // No conditional actions (but more than 2 actions)
  if (actionsWithCondition === 0 && actions.length > 2) {
    add("important", "no_conditional_actions",
      "no actions have conditions — all choices available from the start. " +
      "Consider gating later choices behind scene flags (scene progression).");
  }

  // ── NPC actions ────────────────────────────────────────────────────────────

  const npcActions = spec.npc_actions || [];

  if (npcActions.length === 0 && actionsWithNpcAllowed === 0) {
    add("important", "no_npc_actions",
      "no NPC actions and no action allows them. " +
      "The world should act on the player — consider adding NPC behavior that gates choices.");
  }

  if (npcActions.length > 0) {
    for (const npc of npcActions) {
      if (!npc.id) add("critical", "npc_missing_id", "NPC action missing id");
      const npcEffects = npc.effects || [];
      if (npcEffects.length === 0) {
        add("important", "npc_no_effects",
          `NPC action "${npc.id}" has no effects — NPC actions should change the scene state`);
      }
    }
  }

  // ── Scene progression check ────────────────────────────────────────────────

  // Check if there's a "round" system: actions that set scene_flags which gate later actions
  const flagsSet = new Set();
  const flagsChecked = new Set();

  for (const action of actions) {
    for (const effect of action.effects || []) {
      if (effect.type === "set_scene_flag") flagsSet.add(effect.flag);
    }
    if (action.condition) {
      const flagMatches = action.condition.match(/hasFlag\(['"](\w+)['"]\)/g);
      if (flagMatches) {
        for (const m of flagMatches) {
          const flag = m.match(/['"](\w+)['"]/)[1];
          flagsChecked.add(flag);
        }
      }
    }
  }

  for (const npc of npcActions) {
    for (const effect of npc.effects || []) {
      if (effect.type === "set_scene_flag") flagsSet.add(effect.flag);
    }
    if (npc.condition) {
      const flagMatches = npc.condition.match(/hasFlag\(['"](\w+)['"]\)/g);
      if (flagMatches) {
        for (const m of flagMatches) {
          const flag = m.match(/['"](\w+)['"]/)[1];
          flagsChecked.add(flag);
        }
      }
    }
  }

  // Flags set but never checked = orphaned progression
  const orphanedFlags = [...flagsSet].filter((f) => !flagsChecked.has(f));
  if (orphanedFlags.length > 0 && flagsSet.size > 1) {
    add("important", "orphaned_scene_flags",
      `scene flags set but never checked: [${orphanedFlags.join(", ")}]. ` +
      "If a flag gates a later action, add it to that action's condition. " +
      "If it's for another scene, make it a game_flag instead.");
  }

  // ── Summary stats ──────────────────────────────────────────────────────────

  const stats = {
    actions: actions.length,
    actions_with_effects: actionsWithEffects,
    actions_with_persisting: actionsWithPersisting,
    actions_with_condition: actionsWithCondition,
    actions_with_transition: actionsWithTransition,
    npc_actions: npcActions.length,
    persisting_effect_types: [...allPersistingTypes],
    traits: spec.traits || [],
    scene_flags_set: [...flagsSet],
    scene_flags_checked: [...flagsChecked],
  };

  return buildResult(findings, stats);
}

function buildResult(findings, stats = {}) {
  const hasCritical = findings.some((f) => f.severity === "critical");
  return {
    pass: !hasCritical,
    total: findings.length,
    critical: findings.filter((f) => f.severity === "critical").length,
    important: findings.filter((f) => f.severity === "important").length,
    findings,
    stats,
  };
}

// ─── Self-test ───────────────────────────────────────────────────────────────

function selfTest() {
  let passed = 0;
  let failed = 0;

  function check(name, spec, expectPass, expectRule) {
    const result = validate(spec);
    let ok = true;

    if (result.pass !== expectPass) {
      process.stderr.write(`FAIL: ${name} — expected pass=${expectPass}, got ${result.pass}\n`);
      if (result.findings.length > 0) {
        process.stderr.write(`  findings: ${result.findings.map((f) => f.rule).join(", ")}\n`);
      }
      ok = false;
    }
    if (expectRule && !result.findings.some((f) => f.rule === expectRule)) {
      process.stderr.write(`FAIL: ${name} — expected rule ${expectRule} not found\n`);
      ok = false;
    }

    if (ok) passed++;
    else failed++;
  }

  // Good spec
  check("valid spec passes", {
    scene_id: "base::test",
    brief: "A test scene where the player encounters a stranger in a laundromat. SHY avoids, CONFIDENT engages.",
    description: "Late-night laundromat.",
    traits: ["SHY", "CONFIDENT", "BEAUTIFUL"],
    actions: [
      {
        id: "wait", label: "Wait", detail: "Stay put.",
        allow_npc_actions: true,
        effects: [{ type: "set_scene_flag", flag: "waited" }],
      },
      {
        id: "leave", label: "Leave", detail: "Head out.",
        effects: [{ type: "change_stress", amount: -2 }],
        next: { finish: true },
      },
      {
        id: "talk", label: "Talk to him", detail: "Start a conversation.",
        condition: "scene.hasFlag('approached')",
        effects: [
          { type: "add_npc_liking", npc: "m", delta: 1 },
          { type: "set_game_flag", flag: "LAUNDROMAT_MET" },
        ],
        next: { finish: true },
      },
    ],
    npc_actions: [
      {
        id: "approach", condition: "scene.hasFlag('waited')",
        weight: 10, effects: [{ type: "set_scene_flag", flag: "approached" }],
      },
    ],
  }, true, null);

  // Missing brief
  check("missing brief fails", {
    scene_id: "base::test",
    actions: [{ id: "a", label: "A", effects: [{ type: "change_stress", amount: 1 }] }],
  }, false, "missing_brief");

  // Thin brief
  check("thin brief fails", {
    scene_id: "base::test",
    brief: "A bar scene.",
    actions: [
      { id: "a", label: "A", effects: [{ type: "change_stress", amount: 1 }] },
      { id: "b", label: "B", effects: [{ type: "change_stress", amount: -1 }] },
    ],
    traits: ["SHY"],
  }, false, "thin_brief");

  // No actions
  check("no actions fails", {
    scene_id: "base::test",
    brief: "A scene with no choices which is very boring and should not exist in the game at all.",
    traits: ["SHY", "CONFIDENT"],
  }, false, "no_actions");

  // Effectless action
  check("effectless action fails", {
    scene_id: "base::test",
    brief: "A scene where one action has no consequences which means it goes nowhere and is pointless.",
    traits: ["SHY", "CONFIDENT"],
    actions: [
      { id: "a", label: "Do nothing", detail: "...", effects: [] },
      { id: "b", label: "Leave", detail: "...", effects: [{ type: "change_stress", amount: -1 }] },
    ],
  }, false, "effectless_actions");

  // No persisting effects
  check("scene-only effects fails", {
    scene_id: "base::test",
    brief: "A scene where nothing persists beyond the scene itself which means it has no lasting impact.",
    traits: ["SHY", "CONFIDENT"],
    actions: [
      { id: "a", label: "A", detail: "...", effects: [{ type: "set_scene_flag", flag: "x" }] },
      { id: "b", label: "B", detail: "...", effects: [{ type: "set_scene_flag", flag: "y" }] },
    ],
  }, false, "no_persisting_effects");

  // No traits
  check("no traits fails", {
    scene_id: "base::test",
    brief: "A scene without trait axes which means DeepSeek won't branch and everything is shallow.",
    actions: [
      { id: "a", label: "A", detail: "...", effects: [{ type: "set_game_flag", flag: "X" }] },
      { id: "b", label: "B", detail: "...", effects: [{ type: "change_stress", amount: -1 }] },
    ],
  }, false, "no_traits");

  // No NPC actions (important, not critical)
  check("no NPC actions is important", {
    scene_id: "base::test",
    brief: "A solo scene where nothing external happens to the player which is fine for evening_home type scenes.",
    traits: ["SHY", "CONFIDENT"],
    actions: [
      { id: "a", label: "A", detail: "...", effects: [{ type: "set_game_flag", flag: "X" }] },
      { id: "b", label: "B", detail: "...", effects: [{ type: "change_stress", amount: -1 }] },
    ],
  }, true, "no_npc_actions"); // passes (important, not critical)

  process.stderr.write(`\nspec-validate self-test: ${passed} passed, ${failed} failed\n`);
  return failed === 0;
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

async function main() {
  const args = process.argv.slice(2);

  if (args.includes("--test")) {
    const ok = selfTest();
    process.exitCode = ok ? 0 : 1;
    return;
  }

  if (args.includes("--help") || args.includes("-h")) {
    process.stdout.write(`Usage:
  node tools/spec-validate.mjs <spec.json>   Validate a scene spec
  node tools/spec-validate.mjs --test        Run self-tests

Validates scene spec JSON for structural depth before sending to DeepSeek.
Output: JSON report to stdout.
Exit code: 0 if pass, 1 if critical findings.
`);
    return;
  }

  const specFile = args[0];
  if (!specFile) {
    process.stderr.write("Error: provide a spec JSON file path\n");
    process.exitCode = 1;
    return;
  }

  const specText = await fs.readFile(specFile, "utf8");
  const spec = JSON.parse(specText);
  const result = validate(spec);

  process.stdout.write(JSON.stringify(result, null, 2) + "\n");

  if (!result.pass) {
    process.stderr.write(
      `[spec-validate] FAIL — ${result.critical} critical, ${result.important} important\n`,
    );
    process.exitCode = 1;
  } else if (result.important > 0) {
    process.stderr.write(
      `[spec-validate] PASS with warnings — ${result.important} important\n`,
    );
  } else {
    process.stderr.write("[spec-validate] PASS — structurally sound\n");
  }
}

main().catch((error) => {
  process.stderr.write(`[spec-validate] ${error.message}\n`);
  process.exitCode = 1;
});
