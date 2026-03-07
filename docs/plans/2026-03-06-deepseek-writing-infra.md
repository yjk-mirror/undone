# DeepSeek Writing Infrastructure — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Build prompt-packing tooling and compact reference docs so DeepSeek can reliably generate complete scene TOML files, with Claude orchestrating creative direction and quality review.

**Architecture:** A prompt packer assembles optimal prompt files from a compact "writer core" doc + route-specific context + task payload. DeepSeek generates scene TOML (draft mode) and audits it (review mode). Claude reviews findings, validates templates, applies fixes, commits.

**Tech Stack:** Node.js (tools), Markdown (reference docs), DeepSeek API (already wired via `tools/deepseek-helper.mjs`)

**Design doc:** `docs/plans/2026-03-06-deepseek-writing-infra-design.md`

---

### Task 1: Add `tmp/` to .gitignore

Prompt files and DeepSeek output go in `tmp/`. It must exist and be ignored.

**Files:**
- Modify: `.gitignore`

**Step 1: Add tmp/ to .gitignore**

Add this line after the `.env` line in `.gitignore`:

```
tmp/
```

**Step 2: Create the directory**

Run: `mkdir -p tmp`

**Step 3: Commit**

```bash
git add .gitignore
git commit -m "chore: add tmp/ to gitignore for prompt assembly workspace"
```

---

### Task 2: Fix writing-guide.md m/f availability statement

The writing guide says `m`/`f` are not available in prose templates at all. The engine contract says they ARE available in action/NPC-action prose — just not in intro prose. Fix the guide to match reality.

**Files:**
- Modify: `docs/writing-guide.md:123-126`

**Step 1: Replace the note block**

Find lines 123-126 (the `> **Note:**` block) and replace with:

```markdown
> **Note:** `m` (male NPC) and `f` (female NPC) are available in **action and NPC-action
> prose only** — NOT in intro prose, intro_variants, or thoughts. NPC bindings are not
> established until after scene start. To vary intro prose based on NPC state, use
> `gd.npcLiking("ROLE_X")` which reads from persistent world state.
```

**Step 2: Verify no other m/f misstatements**

Run: `grep -n "not available in prose" docs/writing-guide.md`
Expected: no matches (the old statement is gone)

**Step 3: Commit**

```bash
git add docs/writing-guide.md
git commit -m "fix: correct m/f template availability in writing guide"
```

---

### Task 3: Write the writer core doc

Extract a compact (~3K token) reference from writing-guide.md, engine-contract.md, and content-schema.md. This is DeepSeek's stable system prompt prefix.

**Files:**
- Create: `docs/writer-core.md`

**Step 1: Write docs/writer-core.md**

Source material to extract from (read these first):
- `docs/writing-guide.md` — voice, anti-patterns, trait branching, transformation, content gating, template syntax, physical attributes, scene design
- `docs/engine-contract.md` — template objects, effect types, NPC bindings
- `docs/content-schema.md` — TOML format, all effect types
- `packs/base/scenes/rain_shelter.toml` — reference scene for format example

The doc must contain these sections in this exact order (stable prefix for cache):

1. **Voice** (~200 tokens): BG3 narrator reference, 2nd person present tense, American English. 5 lines max.

2. **Anti-Patterns** (~300 tokens): Every anti-pattern from writing-guide.md as a one-line bullet. No multi-paragraph explanations. Include: staccato declaratives, em-dash reveals, anaphoric repetition, over-naming, emotion announcement, heart/pulse clichés, generic NPC dialogue, passive observation chains, step-by-step narration, adjective-swap branching, AI erotic clichés, overused words list.

3. **Trait Branching Rule** (~200 tokens): The fundamental rule in one sentence ("branches must change what happens, not what adjective is used"). One bad example (3 lines). One good example (6 lines). Both from writing-guide.md.

4. **PC Traits** (~400 tokens): The trait table from writing-guide.md compressed to: `TRAIT_ID — one-sentence behavior shift`. Include all 18 personality traits. Group by: personality, attitude.

5. **NPC Personalities** (~200 tokens): The 5 core personalities + 6 modifier traits, one line each.

6. **Transformation** (~400 tokens): CisMale-only rule. FEMININITY intervals condensed to 4 ranges (0-19, 20-39, 40-59, 60+) with one sentence each. The 4 textures (insider knowledge, body unfamiliarity, social reversal, desire crossover) as bullet points. The `{% if not w.alwaysFemale() %}` pattern. "No {% else %} branches."

7. **Content Gating** (~100 tokens): The three-level pattern (LIKES_ROUGH / default / BLOCK_ROUGH). "{% else %} must be fully written."

8. **Template Objects** (~400 tokens): Compact tables for `w.`, `gd.`, `scene.`, `m.` (action prose only), `f.` (action prose only). Include all method names. Note that `m.`/`f.` are NOT available in intro prose.

9. **Scene TOML Format** (~600 tokens): One complete trimmed example. Use a shortened version of rain_shelter.toml — keep `[scene]`, `[intro]` with one trait branch and the transformation block, one action with effects and next, one NPC action. Include ALL effect type names in a compact list at the end.

10. **Scene Design Checklist** (~200 tokens): Condensed from writing-guide.md. 8-10 bullet points max.

Target: the full doc should be under 13KB (~3,200 tokens). Measure with `wc -c docs/writer-core.md` after writing.

**Step 2: Verify size**

Run: `wc -c docs/writer-core.md`
Expected: under 13000 bytes. If over, trim the longest section.

**Step 3: Commit**

```bash
git add docs/writer-core.md
git commit -m "docs: add writer-core.md — compact DeepSeek prompt prefix"
```

---

### Task 4: Write the review core doc

Extract a compact review reference for DeepSeek review mode.

**Files:**
- Create: `docs/review-core.md`

**Step 1: Read the source**

Read: `.claude/agents/writing-reviewer.md`

**Step 2: Write docs/review-core.md**

Target: ~6KB (~1,500 tokens). Contains:

1. **Role** (2 lines): You review scene prose for AI artifacts, style violations, quality issues. Return findings grouped by severity.

2. **Detection Criteria** (~800 tokens): One line per pattern, grouped by severity:
   - Critical: staccato declaratives, em-dash reveals, anaphoric repetition, over-naming, italicised coinages, scene lacks distinguishing moment, POV violations (any "she" narration), TRANS_WOMAN branches, invalid w.getXxx() accessor, missing BLOCK_ROUGH gate on dark-content traits
   - Important: emotion announcements, heart/pulse clichés, adjective-swap branches, British English, missing FEMININITY calibration, generic physical description when accessor exists, AlwaysFemale {% else %} branches, stolen player agency, trait-gated transformation insight
   - Minor: too many "You" sentence starters, passive observation chains, weak NPC dialogue, missed branching opportunity on new trait groups

3. **Output Format** (~200 tokens): For each finding: quote the text, explain why it's a problem, suggest fix direction. End with overall: Ready / Needs Revision / Significant Rework.

4. **Overused Words** (~200 tokens): The word list from writing-guide.md: "specific/specifically", "something about", "the way", "a quality/a certain", "you notice/you realize", "somehow", "deliberate/deliberately", "something shifts", "the weight of". Flag at 3+ in one scene.

**Step 3: Verify size**

Run: `wc -c docs/review-core.md`
Expected: under 6500 bytes.

**Step 4: Commit**

```bash
git add docs/review-core.md
git commit -m "docs: add review-core.md — compact DeepSeek review prompt"
```

---

### Task 5: Build the prompt packer

A Node script that assembles prompt files from a scene spec.

**Files:**
- Create: `tools/pack-prompt.mjs`

**Step 1: Write tools/pack-prompt.mjs**

The script:
- Reads a JSON scene spec from `--spec-file <path>` or stdin
- Validates required fields: `scene_id`, `brief`
- Assembles the prompt in stable order (see below)
- Writes to `--output <path>` or `tmp/prompt-<scene_id_suffix>.md`
- Prints size stats to stderr: total bytes, estimated tokens (~4 chars/token), warning if over 48KB

Route-to-doc mapping (hardcoded, simple):
```javascript
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
```

Scene file lookup: `packs/base/scenes/<name>.toml` where `<name>` is the reference scene ID with `base::` prefix stripped.

Prompt assembly order:
```
# Writing Rules

<contents of docs/writer-core.md>

---

# Preset: <route name>

<contents of preset doc>

---

# Arc: <route name>

<contents of arc doc>

---

# Reference Scene

<contents of reference scene TOML file, if specified>

---

# Task

Write a complete scene TOML file for `<scene_id>`.

**Slot:** <slot>
**Arc state:** <arc_state>
**FEMININITY range:** <femininity_range>
**Content level:** <content_level>
**Key traits to branch on:** <comma-separated traits>

**Scene brief:**
<brief text>

Output ONLY the complete TOML file. No commentary, no markdown fences, no explanation.
```

Spec fields `route`, `arc_state`, `slot`, `traits`, `femininity_range`, `content_level`, `reference_scenes` are all optional. The packer only includes sections when data is present. `docs/writer-core.md` is always included.

```javascript
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
```

**Step 2: Verify it runs**

Run: `node tools/pack-prompt.mjs --help`
Expected: prints usage text without errors.

**Step 3: Commit**

```bash
git add tools/pack-prompt.mjs
git commit -m "feat: add pack-prompt.mjs — prompt assembly for DeepSeek writing"
```

---

### Task 6: Integration test — rain_shelter reproduction

Test the full pipeline against a known-good scene.

**Files:**
- Create: `tmp/spec-rain-shelter.json` (not committed)

**Step 1: Write a test spec**

Create `tmp/spec-rain-shelter.json`:
```json
{
  "scene_id": "base::rain_shelter",
  "route": "workplace",
  "slot": "free_time",
  "brief": "Caught in sudden rain, you duck into a bus shelter on Clement Ave. A man is already there — mid-twenties, decent jacket, compact umbrella. He looks up when you step in. The scene must have: (1) trait branches that change what the player DOES (SHY takes the far end, POSH positions herself precisely, CUTE laughs and accidentally speaks to him, BITCHY doesn't acknowledge his adjustment), (2) a transformation block where you recognize the male gaze because you used to do it, (3) world texture independent of the player (the rain, a bus that doesn't stop, a woman losing her umbrella across the street), (4) an NPC action where he offers his umbrella, (5) player choices: wait it out, make a run for it, accept umbrella (if offered). The umbrella offer should branch on BEAUTIFUL vs PLAIN — different delivery speed and intent. Set stress +3 if she runs. Set game flag RAIN_SHELTER_VISITED.",
  "traits": ["SHY", "POSH", "CUTE", "BITCHY", "FLIRTY", "BEAUTIFUL", "PLAIN"],
  "femininity_range": "10-30",
  "content_level": "VANILLA"
}
```

**Step 2: Run the packer**

Run: `node tools/pack-prompt.mjs --spec-file tmp/spec-rain-shelter.json`
Expected: creates `tmp/prompt-rain_shelter.md`, prints size stats to stderr.

**Step 3: Inspect the assembled prompt**

Read `tmp/prompt-rain_shelter.md` and verify:
- Writer core is first
- Preset doc (robin.md) is present
- Arc doc (workplace-opening.md) is present
- Task section at the end with the brief
- Total under 48KB

**Step 4: Run DeepSeek draft**

Run: `node tools/deepseek-helper.mjs draft --system-file docs/writer-core.md --prompt-file tmp/prompt-rain_shelter.md --output-file tmp/draft-rain_shelter.toml --json`

Check stderr for cache metrics. Save the JSON output for comparison.

**Step 5: Compare output against real rain_shelter.toml**

Read both files. Check:
- Is the TOML structurally valid?
- Does it have `[scene]`, `[intro]`, `[[actions]]`, `[[npc_actions]]`?
- Are trait branches structurally different (not adjective swaps)?
- Is the transformation block inside `{% if not w.alwaysFemale() %}`?
- Is it second-person present tense throughout?
- Any AI-isms (staccato closers, em-dash reveals, over-naming)?

**Step 6: Run DeepSeek review**

Run: `node tools/deepseek-helper.mjs review --system-file docs/review-core.md --prompt-file tmp/draft-rain_shelter.toml --output-file tmp/review-rain_shelter.md`

Read the review. Note findings.

**Step 7: Validate templates**

Extract each prose field from the draft TOML and run through `mcp__minijinja__jinja_validate_template`.

**Step 8: Document findings**

Note what DeepSeek got right and wrong. This informs Task 7 (iterate on writer-core.md).

Do NOT commit test artifacts — they live in `tmp/` which is gitignored.

---

### Task 7: Iterate on writer-core.md based on test results

Based on Task 6 findings, adjust the writer core to fix whatever DeepSeek got wrong.

**Files:**
- Modify: `docs/writer-core.md`

**Step 1: Identify failure patterns**

From Task 6 comparison, list what DeepSeek got wrong. Common expected issues:
- Wrong TOML structure (missing fields, wrong nesting)
- Template syntax errors (wrong method names, missing endif)
- Adjective-swap branches despite instructions
- AI-isms that slipped through
- Wrong effect type names

**Step 2: Adjust writer-core.md**

For each failure pattern:
- If structural: add/clarify the TOML format section
- If template syntax: add/correct the template object reference
- If prose quality: strengthen the relevant anti-pattern bullet
- If missing: add the missing rule

**Step 3: Re-run the pipeline**

Repeat Task 6 steps 4-7 with the updated writer-core.md. Iterate until the output is structurally correct and prose quality is acceptable.

**Step 4: Commit the improved writer-core.md**

```bash
git add docs/writer-core.md
git commit -m "docs: refine writer-core.md after DeepSeek test pass"
```

---

### Task 8: Test with a new scene

Test with a scene spec for something that doesn't exist yet — no reference to compare against.

**Files:**
- Create: `tmp/spec-new-scene.json` (not committed)

**Step 1: Write a spec for an unwritten scene**

Pick a plausible scene from the workplace arc (e.g., `base::workplace_lunch_break`). Write a creative brief.

**Step 2: Run the full pipeline**

```bash
node tools/pack-prompt.mjs --spec-file tmp/spec-new-scene.json
node tools/deepseek-helper.mjs draft --system-file docs/writer-core.md --prompt-file tmp/prompt-workplace_lunch_break.md --output-file tmp/draft-workplace_lunch_break.toml --json
node tools/deepseek-helper.mjs review --system-file docs/review-core.md --prompt-file tmp/draft-workplace_lunch_break.toml --output-file tmp/review-workplace_lunch_break.md
```

**Step 3: Validate templates with minijinja**

Extract prose fields and validate with `mcp__minijinja__jinja_validate_template`.

**Step 4: Review quality manually**

Check the output against writing-guide standards. Is it good enough to use in a real session with light editing?

**Step 5: Verify cache metrics**

Check the stderr output from both DeepSeek calls. On the second call (review), the writer-core prefix should show cache hits. If not, investigate prompt ordering.

**Step 6: Note findings for future iteration**

Document what worked and what needs adjustment.

---

### Task 9: Thin the scene-writer agent

Strip duplicated rules from scene-writer.md. Make it an orchestrator that references writer-core.md and calls the tooling.

**Files:**
- Modify: `.claude/agents/scene-writer.md`

**Step 1: Read the current agent**

Read: `.claude/agents/scene-writer.md` (currently ~300 lines)

**Step 2: Rewrite to orchestrator shape**

The new scene-writer.md should be ~100 lines. Structure:

```markdown
---
name: scene-writer
description: Writes Undone scene TOML files. Orchestrates DeepSeek for bulk generation, validates output, shapes into final TOML.
tools: Read, Glob, Grep, Write, Edit, Bash, mcp__minijinja__jinja_validate_template, mcp__minijinja__jinja_render_preview
mcpServers:
  minijinja:
model: sonnet
---

You write scenes for **Undone**. Your primary tool is DeepSeek for bulk prose generation.
You orchestrate, review, and validate — DeepSeek does the heavy writing.

## Workflow

1. Read `docs/creative-direction.md` and `docs/writing-guide.md` if not read this session
2. Get a scene spec from the user (scene_id, route, brief, traits, etc.)
3. Write the spec as JSON to `tmp/spec-<name>.json`
4. Run: `node tools/pack-prompt.mjs --spec-file tmp/spec-<name>.json`
5. Run: `node tools/deepseek-helper.mjs draft --system-file docs/writer-core.md --prompt-file tmp/prompt-<name>.md --output-file tmp/draft-<name>.toml`
6. Run: `node tools/deepseek-helper.mjs review --system-file docs/review-core.md --prompt-file tmp/draft-<name>.toml --output-file tmp/review-<name>.md`
7. Read the review findings
8. Read the draft TOML
9. Fix Critical/Important findings
10. Validate all prose fields with `mcp__minijinja__jinja_validate_template`
11. Write the final TOML to `packs/base/scenes/<name>.toml`

## Key Rules (full rules in docs/writing-guide.md)

- Always second-person present tense — no "she" narration
- CisMale-only: transformation content in `{% if not w.alwaysFemale() %}`, no {% else %}
- Trait branches must change what HAPPENS, not what adjective describes it
- m./f. available in action prose only — NOT in intro prose
- Every scene needs an inciting situation, 1-3 genuine choices, at least one lasting consequence
- Do not write scenes without a creative spec from the user

## DeepSeek Safety

Only send fictional content to DeepSeek. Never send secrets, file paths, git identity, or unrelated repo data.

## After Writing

Check against the Scene Authorship Checklist in `docs/writing-guide.md`.
```

Remove all duplicated sections: the full anti-patterns list, the full trait tables, the full template syntax reference, the full TOML format spec, the full physical attribute reference, the full content gating docs. These now live in writer-core.md (for DeepSeek) and writing-guide.md (for the agent's own reference via Read tool).

**Step 3: Commit**

```bash
git add .claude/agents/scene-writer.md
git commit -m "refactor: thin scene-writer agent to orchestrator shape"
```

---

### Task 10: Thin the writing-reviewer agent

Same treatment as scene-writer — strip duplicated rules, reference review-core.md.

**Files:**
- Modify: `.claude/agents/writing-reviewer.md`

**Step 1: Read the current agent**

Read: `.claude/agents/writing-reviewer.md` (currently ~288 lines)

**Step 2: Rewrite to reference review-core.md**

The new writing-reviewer.md should be ~80 lines. Structure:

```markdown
---
name: writing-reviewer
description: Reviews Undone scene prose for AI-isms, style guide violations, quality issues. Read-only — reports, does not edit.
tools: Read, Glob, Grep, mcp__minijinja__jinja_validate_template
mcpServers:
  minijinja:
model: sonnet
---

You review scene prose for **Undone**. You catch AI artifacts, writing guide violations, and quality issues. You read and report. You do not edit.

## Before Reviewing

Read these files to calibrate:
- `docs/creative-direction.md` — creative bible
- `docs/writing-guide.md` — the complete standard you enforce
- `docs/writing-samples.md` — reference voice
- The scene(s) under review

## Detection Criteria

Read `docs/review-core.md` for the complete detection checklist. The key severity levels:

**Critical** — must fix before commit:
- AI-isms (staccato, em-dash reveals, over-naming, anaphoric repetition)
- POV violations (any "she" narration in prose)
- TRANS_WOMAN branches, invalid w.getXxx() accessors
- Missing BLOCK_ROUGH gate on dark-content traits

**Important** — should fix:
- Emotion announcements, heart/pulse clichés, adjective-swap branches
- British English, missing FEMININITY calibration
- AlwaysFemale {% else %} branches, stolen player agency

**Minor** — polish:
- Sentence starter variety, weak NPC dialogue, missed branching opportunities

## DeepSeek Second Opinion

When useful, run: `node tools/deepseek-helper.mjs review --system-file docs/review-core.md --prompt-file <scene> --output-file tmp/ds-review-<name>.md`

Treat DeepSeek's output as input to your review, not as the final word.

## Output Format

For each finding: quote the text, explain the problem, suggest fix direction.
End with: **Ready** / **Needs Revision** / **Significant Rework**
```

**Step 3: Commit**

```bash
git add .claude/agents/writing-reviewer.md
git commit -m "refactor: thin writing-reviewer agent to reference review-core.md"
```

---

### Task 11: Annotate stale audit finding

The writing-agent-tooling-audit says cache reporting is missing. It's already implemented. Mark it.

**Files:**
- Modify: `docs/audits/2026-03-07-writing-agent-tooling-audit.md`

**Step 1: Find the cache reporting section**

Read: `docs/audits/2026-03-07-writing-agent-tooling-audit.md`
Find the section about "prompt cache hit/miss reporting" (around line 105 and 145).

**Step 2: Add resolution annotations**

Add `✅ Already implemented in deepseek-helper.mjs (lines 260-266)` next to the cache reporting items.

**Step 3: Commit**

```bash
git add docs/audits/2026-03-07-writing-agent-tooling-audit.md
git commit -m "docs: annotate stale cache-reporting audit finding as resolved"
```

---

### Task 12: Update docs and HANDOFF.md

Update project docs to reflect the new infrastructure.

**Files:**
- Modify: `docs/deepseek-writing-tool.md`
- Modify: `HANDOFF.md`

**Step 1: Update deepseek-writing-tool.md**

Add the prompt packer workflow to the existing doc. Add a section:

```markdown
## Prompt Packer

`tools/pack-prompt.mjs` assembles optimal prompt files from scene specs.

### Usage

```bash
node tools/pack-prompt.mjs --spec-file tmp/spec-scene.json
```

See the script's `--help` for full options and spec format.

### Full Pipeline

```bash
# 1. Pack the prompt
node tools/pack-prompt.mjs --spec-file tmp/spec.json

# 2. Generate draft
node tools/deepseek-helper.mjs draft --system-file docs/writer-core.md \
  --prompt-file tmp/prompt-scene.md --output-file tmp/draft-scene.toml

# 3. Review
node tools/deepseek-helper.mjs review --system-file docs/review-core.md \
  --prompt-file tmp/draft-scene.toml --output-file tmp/review-scene.md
```
```

**Step 2: Update HANDOFF.md**

Add to Current State:
- DeepSeek writing infrastructure built: writer-core.md, review-core.md, pack-prompt.mjs
- Scene-writer and writing-reviewer agents thinned to orchestrator shape
- Full pipeline tested against rain_shelter reproduction and new scene generation

Add to Session Log with timestamp.

**Step 3: Commit**

```bash
git add docs/deepseek-writing-tool.md HANDOFF.md
git commit -m "docs: update HANDOFF and deepseek tool docs for new writing infrastructure"
```

---

### Task 13: Acceptance tests — full pipeline verification

Verify the complete pipeline works end-to-end from spec to validated TOML.

**Acceptance Criteria:**
- `pack-prompt.mjs --help` prints usage without error
- `pack-prompt.mjs` with a valid spec produces a prompt file under 48KB
- `pack-prompt.mjs` with missing `scene_id` errors with clear message
- `pack-prompt.mjs` with missing `brief` errors with clear message
- `pack-prompt.mjs` with `route: "workplace"` includes robin.md and workplace-opening.md content
- `pack-prompt.mjs` with `reference_scenes: ["rain_shelter"]` includes rain_shelter.toml content
- `pack-prompt.mjs` with no route still produces a valid prompt (writer-core + task only)
- `deepseek-helper.mjs draft` with the packed prompt produces TOML output
- `deepseek-helper.mjs review` with the draft produces findings
- The draft TOML has valid minijinja templates (validated by MCP tool)
- Cache metrics appear in deepseek-helper stderr output

**Step 1: Run acceptance tests**

Run each command and verify output:

```bash
# Help
node tools/pack-prompt.mjs --help

# Missing scene_id
echo '{"brief":"test"}' | node tools/pack-prompt.mjs

# Missing brief
echo '{"scene_id":"base::test"}' | node tools/pack-prompt.mjs

# Minimal spec (no route)
echo '{"scene_id":"base::test","brief":"A test scene."}' | node tools/pack-prompt.mjs --output tmp/test-minimal.md

# Workplace route
echo '{"scene_id":"base::test","route":"workplace","brief":"Test."}' | node tools/pack-prompt.mjs --output tmp/test-workplace.md

# With reference scene
echo '{"scene_id":"base::test","route":"workplace","brief":"Test.","reference_scenes":["rain_shelter"]}' | node tools/pack-prompt.mjs --output tmp/test-ref.md
```

Verify each output: errors print clearly, prompt files contain expected sections, sizes are reasonable.

**Step 2: Verify end-to-end with DeepSeek**

Run the rain_shelter reproduction test (Task 6) one final time to confirm everything works cleanly.

**Step 3: Verify template validation**

Extract at least one prose block from the DeepSeek draft and validate with minijinja MCP.

---

## Execution handoff

```
Use `ops:executing-plans` to implement the plan at `docs/plans/2026-03-06-deepseek-writing-infra.md`
```
