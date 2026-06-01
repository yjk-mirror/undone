# Writing Delegation

This document describes the repo-neutral scene writing workflow. It is for any
orchestrator, not only Claude Code.

## Roles

**Writer**

- Input: scene id, route, premise, required actions, required effects, relevant
  traits, NPC context, and any user-provided creative constraints.
- Reads: `docs/creative-direction.md`, `docs/writing-guide.md`,
  `docs/writing-samples.md`, `docs/writer-core.md`, and the target route or
  character docs.
- Output: final scene TOML at `packs/base/scenes/<scene_id>.toml`, plus a short
  note listing validations run and any unresolved creative risks.

**Reviewer**

- Input: the target scene file and the intended scene brief.
- Reads: `docs/creative-direction.md`, `docs/writing-guide.md`,
  `docs/writing-samples.md`, and `docs/review-core.md`.
- Output: findings grouped by severity, ending with `Ready`, `Needs Revision`,
  or `Significant Rework`. The reviewer does not edit files.

## Safe Input Boundary

Only send fictional content-work context to subordinate models or external
helpers:

- scene specs and briefs
- writing rules
- trait, skill, stat, scene, arc, and flag ids
- relevant fictional character and route notes
- sample prose and review targets

Do not send secrets, `.env` contents, credentials, personal data, git identity,
machine-local paths, unrelated repository code, or unrelated logs.

## Drafting Workflow

1. Confirm the user has provided a creative spec. Do not invent major scene
   direction, relationship outcomes, or explicit-content premise.
2. Write a compact JSON scene spec in `tmp/spec-<scene_id>.json`.
3. Run `node tools/spec-validate.mjs tmp/spec-<scene_id>.json`.
4. Prefer the pipeline:
   `node tools/scene-pipeline.mjs --spec-file tmp/spec-<scene_id>.json`.
5. If using individual tools, assemble context with
   `node tools/pack-prompt.mjs --spec-file tmp/spec-<scene_id>.json`, draft via
   `node tools/deepseek-helper.mjs draft`, and convert labeled prose with
   `node tools/prose-to-toml.mjs`.
6. Read the generated TOML yourself. Fix structural issues directly; do not
   accept subordinate output blindly.
7. Validate minijinja templates for all prose fields before delivery.
8. Run `node tools/prose-lint.mjs packs/base/scenes/<scene_id>.toml`.
9. Run `cargo run --bin validate-pack`.

## Review Workflow

1. Run `node tools/prose-lint.mjs <scene.toml>` first and treat findings as real.
2. Read the scene against the brief and the intro/action split.
3. Check every action for consequence, state change, or meaningful progression.
4. Check trait branches for structural differences rather than adjective swaps.
5. Check template receiver scope: `m` and `f` are action/NPC-action prose only,
   not intro prose, intro variants, or thoughts.
6. Return concrete findings with quoted evidence and fix direction.

## Validation Checklist

- Scene TOML parses.
- All templates validate.
- `prose-lint` has no Critical findings for the touched scene.
- `validate-pack` exits successfully.
- Any remaining warnings are named explicitly and classified as creative,
  design-blocked, or intentionally deferred.
