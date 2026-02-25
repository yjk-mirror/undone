# Engineering Audit — 2026-02-25

**Scope:** All game code (7 crates) + pack data files. 29 commits since prolific-session plan.
**Baseline:** 204 tests passing, zero clippy warnings, zero compiler errors.
**Method:** 6 parallel review agents (domain+world, packs+expr, scene+save, UI, pack data, cargo toolchain).

---

## Summary

| Severity | Count | Actionable now |
|----------|-------|----------------|
| Critical | 8     | 8              |
| Important | 19   | 15             |
| Minor    | 18    | deferred       |

Overall the codebase is well-structured. Dependency direction is clean across all 7 crates. No hardcoded content IDs in the core engine (scene/scheduler/effects). Load-time validation is systematic. Test coverage is strong at 204 tests with meaningful coverage of edge cases.

The critical findings are real bugs or principle violations that should be fixed before the next content pass.

---

## Critical Findings

### C1. `once_only` flag is never set — once-only scenes fire repeatedly
**Crate:** undone-ui (`left_panel.rs:204`, `lib.rs:257`)
**Issue:** `pick_next()` returns `PickResult` with `once_only: bool`, but the caller drops it unused. The `ONCE_<scene_id>` game flag is never set. Scenes marked `once_only = true` in `schedule.toml` will repeat indefinitely.
**Fix:** After `pick_next` returns a result with `once_only == true`, set the game flag before starting the scene.

### C2. `choose_action` does not re-check conditions on the chosen action
**Crate:** undone-scene (`engine.rs:253`)
**Issue:** Actions whose condition evaluates to `false` (hidden from UI) can still be executed if the caller sends their ID. No condition guard in `choose_action`.
**Fix:** Re-evaluate the action's condition in `choose_action` before executing. Return silently or emit `ErrorOccurred` if it fails.

### C3. Scheduler load failure is silent — game goes blank after first scene
**Crate:** undone-ui (`game_state.rs:115`)
**Issue:** `load_schedule` failure falls back to `Scheduler::empty()` with only an `eprintln!`. After the opening scene finishes, `pick_next` returns `None` and the player is stuck at a blank story panel.
**Fix:** Treat scheduler load failure the same as scene load failure — set `init_error` and return via `failed_pre`.

### C4. `ArcDef.initial_state` is declared, populated in TOML, never consumed
**Crate:** undone-packs (`data.rs:114`, `arcs.toml`)
**Issue:** Both arcs set `initial_state = "arrived"` but no code reads this field. Pack authors who rely on it will see no effect silently.
**Fix:** Wire `initial_state` into `new_game()` via the registry, or remove the field from TOML until implemented.

### C5. Spawner produces 3 NPCs when `male_count < 3`
**Crate:** undone-packs (`spawner.rs:71`)
**Issue:** `REQUIRED_PERSONALITIES` always starts with 3 entries. When `male_count` is 0, 1, or 2, the `while` loop never fires and all 3 are spawned. The throwaway world requests `male_count: 0` and gets 3 NPCs.
**Fix:** `personality_ids.truncate(config.male_count)` before the iteration loop.

### C6. Hardcoded `FEMININITY`, `TRANS_WOMAN`, `ALWAYS_FEMALE`, `NOT_TRANSFORMED` in engine code
**Crate:** undone-packs (`char_creation.rs:88-114`)
**Issue:** Engineering Principle 2 violation. The trait injections use `if let Ok(...)` which silently skips if the trait is absent.
**Fix:** Define required-skills/required-traits in the pack manifest and validate at load time. Or document as a known design compromise and make the silent skips into loud warnings.

### C7. Dead `From<&NpcCore>` impl with Debug-format bug
**Crate:** undone-ui (`lib.rs:131`)
**Issue:** Never called. Would produce `PersonalityId(Spur { ... })` as the personality string if invoked. False interface.
**Fix:** Remove entirely.

### C8. FEMININITY skill `min=-100` contradicts documentation and is not enforced
**Pack data:** `skills.toml`
**Issue:** `SkillIncrease` in effects.rs does not clamp to declared min/max. CLAUDE.md says range is 0–100+.
**Fix:** Either enforce clamping in the engine, or change `min = 0` in skills.toml and document that min/max are advisory.

---

## Important Findings

### I1. `validate_effects` does not validate `AddStat`/`SetStat`/`FailRedCheck` stat/skill names at load time
**Crate:** undone-scene (`loader.rs:274`)
**Principle:** No silent defaults for content errors.

### I2. `validate_trait_conflicts` not called in `validate-pack` binary
**Crate:** undone-packs (`validate_pack.rs`)
**Impact:** Offline pack validation tool misses conflict errors.

### I3. Category IDs in conditions not validated at load time — `inCategory('TYPO')` silently returns false
**Crate:** undone-expr (`eval.rs:293`)
**Principle:** Fail fast, fail loud.

### I4. Spawner uses hardcoded race list, ignores pack-loaded `registry.races()`
**Crate:** undone-packs (`spawner.rs:38`)
**Principle:** Data-driven over code-driven.

### I5. Parser recursion has no depth limit (`parse_not`, parenthesized sub-expressions)
**Crate:** undone-expr (`parser.rs:111`)
**Principle:** Bounded resources.

### I6. `stat_defs` name/description discarded at registration — no stat validation possible
**Crate:** undone-packs (`registry.rs:86`)

### I7. `ThoughtAdded`/`ErrorOccurred` events bypass the 200-paragraph cap
**Crate:** undone-ui (`lib.rs:459`)

### I8. `ThoughtAdded` unconditionally scrolls, violating the new-scene-top invariant
**Crate:** undone-ui (`lib.rs:469`)

### I9. Saves panel scroll missing `shrink_to_fit()` + `flex_basis(0.0)` — scrolling may not activate
**Crate:** undone-ui (`saves_panel.rs:332`)

### I10. Hardcoded content trait IDs in UI (`BLOCK_ROUGH`, `LIKES_ROUGH`, `BEAUTIFUL`, `PLAIN`)
**Crate:** undone-ui (`char_creation.rs:889`)

### I11. `"FEMININITY"` resolved four times by string lookup — cache SkillId in GameState
**Crate:** undone-ui (`lib.rs`, `left_panel.rs`, `saves_panel.rs`)

### I12. Tab buttons active during char creation phases with no effect — confusing affordance
**Crate:** undone-ui (`title_bar.rs:31`)

### I13. No tests for `process_events`, paragraph cap, or `AppPhase` transitions
**Crate:** undone-ui

### I14. `SceneId` newtype defined in domain but unused everywhere else — false type safety
**Crate:** undone-domain (`ids.rs:33`)

### I15. `has_before_life()` is a redundant alias for `was_transformed()`
**Crate:** undone-domain (`enums.rs:156`)

### I16. Unused `anyhow` dependency in undone-scene
**Crate:** undone-scene (`Cargo.toml`)

### I17. `default_slot` in pack.toml is stale after arc-system refactor
**Pack data:** `pack.toml`

### I18. Arrival scenes redundantly set route flags already set by preset
**Pack data:** `robin_arrival.toml`, `camila_arrival.toml`

### I19. `stats.toml` defines three stats never referenced in any scene
**Pack data:** `stats.toml`

---

## Minor Findings (deferred)

| # | Location | Issue |
|---|----------|-------|
| M1 | `npc.rs:34` | `NpcCore.knowledge` is dead, undocumented |
| M2 | `player.rs:37` | `SkillValue.modifier` needs doc comment |
| M3 | `enums.rs:213` | `Personality` missing doc explaining no Serialize/Display |
| M4 | `enums.rs:109` | `LateTwenties` vs `MidLateTwenties` distinction undocumented |
| M5 | `game_data.rs:7` | Unbounded flag/collection growth (principle 5) |
| M6 | `game_data.rs:86` | `red_check` separator relies on naming convention |
| M7 | `eval.rs:150` | `eval_to_value` type inference order undocumented |
| M8 | `eval.rs:293` | `CategoryType::Personality` always returns false without comment |
| M9 | `eval.rs:59` | Skill rolls use unseeded `thread_rng` |
| M10 | `registry.rs:25` | `trait_defs`, `npc_trait_defs`, `skill_defs` unnecessarily `pub` |
| M11 | `engine.rs:334` | `select_intro_prose` passes hardcoded `"variant"` as scene_id |
| M12 | `template_ctx.rs:265` | Fresh `minijinja::Environment` created per render call |
| M13 | `engine.rs` | Test world construction duplicated across 5 modules |
| M14 | `Cargo.toml:16` (ui) | `slotmap` dependency unused |
| M15 | `lib.rs:304` | Duplicate `.background()` between `body` and `main_column` |
| M16 | `char_creation.rs:495` | `rand::random` (global RNG) in name randomizer |
| M17 | `title_bar.rs` | Tab buttons missing `keyboard_navigable()` |
| M18 | `settings_panel.rs` | Theme/mode buttons missing `keyboard_navigable()` |

---

## Strengths (cross-crate)

- **Dependency DAG is clean.** No cycles, no upward deps. Every crate respects the declared hierarchy.
- **No hardcoded content IDs in core engine.** Scene engine, scheduler, effects — all content-agnostic.
- **Load-time validation is systematic.** Conditions, trait IDs, skill IDs, arc IDs, goto targets — all checked before runtime.
- **Bounded transition guard works.** `MAX_TRANSITIONS_PER_COMMAND = 32` prevents goto cycles.
- **Save migration chain is complete.** v1→v2→v3→v4 tested end-to-end.
- **Fail-fast principle is generally followed.** Pack loading, scene loading, and game init all surface errors visibly.
- **String interning is correct.** Typed ID wrappers prevent cross-type confusion at compile time.
- **204 tests cover core lifecycle.** Load, validate, render, execute effects, schedule, migrate.
- **Scene cross-references are fully valid.** All schedule→scene, goto→action, arc→state, flag→setter references resolve.
