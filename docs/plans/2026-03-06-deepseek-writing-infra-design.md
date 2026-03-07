# DeepSeek Writing Infrastructure — Design

Date: 2026-03-06

## Goal

Build the infrastructure so DeepSeek does the bulk of scene prose generation reliably
and cheaply, with Claude orchestrating direction and quality review.

## Architecture

```
User/Opus (creative direction + scene specs)
    ↓
pack-prompt.mjs (assembles optimal prompt — no LLM)
    ↓
deepseek-helper.mjs draft (generates complete scene TOML)
    ↓
deepseek-helper.mjs review (self-audit against writing rules)
    ↓
Opus/Sonnet (apply fixes, validate templates, commit)
```

## Key Artifacts

### 1. Writer Core (`docs/writer-core.md`)

Compact extraction (~3K tokens) from writing-guide.md + engine-contract.md +
content-schema.md. This is the stable system prompt prefix that DeepSeek caches
after the first call.

Contents:
- Voice spec (BG3 narrator, 2nd person present, American English)
- Anti-pattern checklist (terse, one line each)
- Trait branching rule + one good/bad example
- PC trait reference table (compact)
- NPC personality table (compact)
- Transformation rules (CisMale-only, FEMININITY intervals condensed, 4 textures)
- Content gating pattern
- Template object reference (w./gd./scene./m./f. methods)
- One complete reference scene TOML (rain_shelter trimmed)
- Effect types list

### 2. Review Core (`docs/review-core.md`)

Compact extraction (~1.5K tokens) for DeepSeek review mode:
- Detection criteria (one line per pattern)
- Severity definitions
- Output format spec

### 3. Prompt Packer (`tools/pack-prompt.mjs`)

Node script. Input: JSON scene spec. Output: assembled prompt file.

Scene spec format:
```json
{
  "scene_id": "base::workplace_coffee_break",
  "route": "workplace",
  "arc_state": "working",
  "slot": "work",
  "brief": "Coffee break in the office kitchen...",
  "traits": ["SHY", "AMBITIOUS", "OBJECTIFYING"],
  "npcs": [],
  "femininity_range": "10-30",
  "content_level": "VANILLA",
  "reference_scenes": ["workplace_first_day"]
}
```

What the packer does:
1. Reads `docs/writer-core.md` (always — stable prefix)
2. Looks up `route` → pulls preset doc + arc doc
3. If `reference_scenes` specified, reads those scene TOMLs
4. Appends brief as task payload
5. Writes to `tmp/prompt-<scene_id>.md`
6. Warns if total exceeds 12K tokens (~48KB)

Prompt structure (stable order for DeepSeek cache):
```
[writer-core.md]           ← cached after first call
---
[preset doc]               ← cached if same route
[arc doc]                  ← cached if same arc
---
[reference scene TOML]     ← voice calibration
---
TASK: Write scene `base::scene_id`
[brief, constraints, traits, femininity range, content level]
```

### 4. Doc Fixes

- writing-guide.md: fix m/f availability statement to match engine-contract
  (available in action/NPC-action prose, not intro prose)
- scene-writer.md: thin to orchestrator prompt (~100 lines), reference
  writer-core.md instead of duplicating rules
- writing-reviewer.md: thin similarly, reference review-core.md
- Annotate stale audit finding (cache reporting already implemented)

## Testing Plan

### Phase 1: Reproduction test
- Pick rain_shelter (known-good scene)
- Write a spec as if it didn't exist
- Run packer → inspect assembled prompt
- Run deepseek draft → inspect output
- Compare against real rain_shelter.toml
- Run deepseek review → check findings
- Iterate on writer-core.md based on what DeepSeek gets wrong

### Phase 2: New scene test
- Write a spec for a scene that doesn't exist yet
- Run full pipeline
- Validate templates with minijinja MCP
- Review output quality manually

### Phase 3: Iterate
- Adjust writer-core.md based on test results
- Adjust prompt structure if needed
- Verify cache-hit rates from helper output

## Real Session Workflow

```
Opus: "Write these 5 scenes" + creative specs
  ↓
For each scene (parallelizable):
  1. pack-prompt.mjs builds prompt
  2. deepseek-helper.mjs draft produces scene TOML
  3. deepseek-helper.mjs review audits draft
  4. minijinja validates templates
  5. Opus/Sonnet reviews findings, applies fixes
  6. Commit
```

## What's Deferred

- Embedding/vector retrieval (keyword matching sufficient for current scale)
- NPC profile cards (only Robin and Camila exist)
- Local response caching/dedup (premature)
- Restructuring writing-guide.md itself (stays as full human reference)
