# Ironclad Engine Hardening Plan

Date: 2026-03-07
Baseline commit: `72e90d4`

## Goal

Reach a state where content writing can proceed against an engine contract that is hard to violate silently, easy to validate, and well-covered by automated checks.

This is not a feature sprint for new content. It is a hardening sprint.

Primary rule:

- prefer validated contracts over runtime assumptions

## Current Baseline

The readiness sprint already established:

- pack/bootstrap fail-fast validation
- startup validation for scenes, schedules, manifest entry scenes, and char-creation trait dependencies
- a written engine contract in `docs/engine-contract.md`
- a readiness matrix in `docs/audits/2026-03-07-engine-readiness-matrix.md`
- DeepSeek subordinate tooling in `tools/deepseek-helper.mjs`

The remaining work is not "basic engine missing." It is hardening, diagnostics, and end-to-end correctness.

## Exit Criteria

Before content writing begins in earnest, the repo should satisfy all of these:

1. Save/resume behavior is covered end to end, not just by format/migration tests.
2. Runtime authoring failures are visible enough for QA to catch quickly.
3. `validate-pack` catches the important classes of content breakage before runtime.
4. The current NPC context contract is either proven sufficient for writing scope or replaced with a stronger one.
5. Verification is repeatable from the repo root with a short, explicit command set.

## Workstreams

### 1. Save / Resume Hardening

Target:

- prove that loading a save recreates a clean runtime and resumes safely from persisted world state

Tasks:

- add end-to-end tests that:
  - start from a real `PreGameState`
  - create or simulate a playable world transition
  - save and reload through `undone_save::save_game` + UI load helpers
  - verify no opening-scene replay
  - verify no stale scene stack / queued runtime state leaks through load
  - verify scheduler decisions still reflect persisted flags/arc state after reload
- add at least one regression test around save compatibility with expanded registries

Done when:

- save/resume semantics in `docs/engine-contract.md` are directly exercised by tests, not inferred from lower-level units

### 2. Runtime Diagnostics Hardening

Target:

- make content/runtime failures obvious during QA instead of only visible in logs

Tasks:

- audit current failure paths for:
  - condition evaluation failures
  - template render failures
  - scene/effect execution failures
- decide which failures should remain log-only and which should surface in UI/debug output
- add tests around chosen behavior
- document the final visibility rules in `docs/engine-contract.md`

Done when:

- a bad scene/template/condition during playtesting is quick to identify without reading code

### 3. Authoring Validation Hardening

Target:

- catch more content problems in `validate-pack` and startup, before runtime

Tasks:

- audit `validate-pack` against current failure classes and warnings
- decide which current warnings should stay warnings versus become errors
- consider adding checks for:
  - duplicate or structurally suspicious content IDs
  - malformed schedule/scene authoring patterns not already covered
  - content that is reachable but leaves no durable state when that is unintended
- keep the validator aligned with `docs/content-schema.md`

Done when:

- `validate-pack` is the default trust gate for content authors, not just a partial checker

### 4. NPC Context Contract Decision

Target:

- remove ambiguity around active-NPC semantics before more scene writing depends on them

Tasks:

- review current `m` / `f` contract in `docs/engine-contract.md`
- decide whether one active male + one active female is enough for the next writing phase
- if yes:
  - add tests and authoring guidance that make the limitation explicit
- if no:
  - design the replacement before writing scenes that would depend on it

Done when:

- writers are not guessing how multi-NPC scenes are supposed to bind recipients

### 5. Verification and Handoff Discipline

Target:

- make future sessions deterministic and low-friction

Tasks:

- keep `HANDOFF.md`, `docs/engine-contract.md`, and the readiness matrix aligned
- record the final command set for verification
- avoid reopening already-closed readiness items as if they were still pending

Verification baseline:

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run --bin validate-pack`
- `node tools/deepseek-helper.mjs --help`

## Recommended Order

1. Save / Resume Hardening
2. Runtime Diagnostics Hardening
3. Authoring Validation Hardening
4. NPC Context Contract Decision
5. Final docs sync and verification

## Non-Goals For This Plan

- no new real content writing
- no broad UI redesign unless required for correctness
- no replacement of the orchestrating model with DeepSeek
- no speculative feature expansion unrelated to contract safety
