# Undone — Handoff

## Current State

**Latest session (2026-06-02, MERGE CONFIRM + PUSH — registry is on master, remote synced):**
The `script-api-registry` work was already merged to master in a prior session (`e046ffe merge:
script-api-registry`) — the branch is gone and the populated `REGISTRY`
(`crates/undone-scene/src/script/api/table.rs`, ~95 reads + ~37 writes) is live on the main tree.
This session confirmed that and **pushed master to `origin/master`** (was 134 commits ahead;
`f35e4b2..0a20a15`). The registry consolidation plus all content/tooling work since are now on the
remote. The "NOT yet merged" / "Merge → master" notes in the entry below are **superseded**.

**Strategic posture (user-confirmed):** the engine has converged on *scripting as the content layer*.
New scenes that recombine existing verbs are pure Rhai + minijinja + TOML (no Rust); new
traits/skills/stats/NPCs are TOML data; only a genuinely new *mechanic/system* (e.g. the
DESIRE/COMPOSURE need-state) or a genuinely new *verb* needs Rust — and the registry made adding a
verb a single row + accessor that propagates to Rhai/gate/prose/prose-gate. Content-first from here.

**Two latent tools-workspace defects found + FIXED this session** (`9782763`; root-cause fixes, not
workarounds — supersedes the "proposed allowlist" infra note in the registry entry below):
- **MCP build was broken since the registry merge** (this was the real "rebuild blocker" — NOT a
  locked binary). Making the MCP servers depend on `undone-scene` meant `rhai-mcp-server`'s
  `rhai = ["sync"]` feature unified `rhai/sync` onto `undone-scene`, whose eval closures capture a
  thread-local raw-ptr borrow bridge and are deliberately non-`Send`/`Sync` → 13 compile errors in
  `rhai_bind.rs`. Fix: dropped `sync` from `rhai-mcp-server`; it now builds its validation `Engine`
  per-call instead of storing it on the (Send+Sync) handler. `cd tools && cargo check --release`
  clean; `rhai-mcp-server` 10/10 tests pass.
- **rust-analyzer orphaning root-caused + fixed:** `rust-mcp` spawned rust-analyzer with no
  `kill_on_drop` and `processId: null`, so the child outlived the server. Added `.kill_on_drop(true)`
  + `processId = std::process::id()` (LSP-spec self-exit if rust-mcp is hard-killed). This is the
  correct fix — subagents that start rust-mcp no longer leak rust-analyzer, so no agent `mcpServers`
  allowlist is needed. Cleaned ~1.2 GB of existing orphans this session (only the live session's RA
  subtree remains).

**Open from here:**
- **Marcus terms-fork implementation** — design + phased plan committed (`dcea56a`, `0a20a15`:
  `docs/plans/2026-06-02-marcus-terms-fork-*`); fan-out execution pending.
- **Produce + swap the MCP binaries** — code is fixed and verified-compiling, but the live `.exe`s are
  locked by the running MCP servers, so the swap must happen with them down: **restart the session,
  then `pwsh tools/rebuild-mcp.ps1`** (or run it from a session with those servers disabled). Only
  after that do the new `kill_on_drop`/`processId` behavior and the `minijinja-mcp-server`
  `jinja_validate_prose` tool go live.

---

**Prior session (2026-06-02, SCRIPT API REGISTRY — single source of truth, COMPLETE, MERGED to master `e046ffe`):**
Executed `docs/plans/2026-06-02-script-api-registry.md` end-to-end (Phases A–K). Replaced the three
independently-maintained copies of the content-facing method surface (Rhai `read_api/`+`write_api/`,
the `validate.rs` spec tables, the Minijinja `template_ctx.rs` snapshot) with ONE declarative
`static REGISTRY` in `crates/undone-scene/src/script/api/table.rs` that drives all four consumers,
and added a load-time prose gate.

**What shipped (worktree `.worktrees/script-api-registry`, ~13 commits):**
- `script/api/` module: neutral `ApiValue`/`ApiArg`/`ApiError` types; `MethodDescriptor`/`Receiver`/
  `ArgShape`/`Contexts`; `REGISTRY` (~95 reads + ~37 writes); pure accessor fns in `api/read/*` +
  `api/write/*` (mechanical lifts of the old closures, code-reviewer PASS on fidelity).
- **Rhai** (`api/rhai_bind.rs`) registers cond/effect engines by iterating `REGISTRY`; `read_api/` +
  `write_api/` DELETED. Full `rhai_parity.rs` golden corpus green.
- **Static gate** (`validate.rs`) `read_spec`/`write_spec` are now `REGISTRY` lookups + `method_spec_from_argshape`; the 165-line hand-written spec match is gone. Negative-surface tests assert per-receiver sets (`f.hasTrait`/`m.isPregnant` stay unknown).
- **Minijinja snapshot ELIMINATED** (`api/minijinja_bind.rs`): six ZST `Object` views read live `World`
  through `with_read_borrows`; `template_ctx.rs` shrunk to npc-presence + guard + render (plain
  `render` only). The §0 read/prose equivalence harness passes with one shared accessor.
- **Prose load gate** (`api/prose_validate.rs`, single-quote-aware tokenizer): wired into `loader.rs`
  (all prose fields) + `validate-pack` (preset beats); `char_creation` discovery-beat render de-masked.
- **minijinja-mcp-server** gains `jinja_validate_prose` (authoring-time == load-time).
- **Decisions baked in (design §8 step 0):** `m`/`f`/`role` `getName` → `effective_name()` (display
  name, not spawn name); `getName` ADDED to `m`/`f`; `checkSkill`/`checkSkillRed` condition-only
  (barred from prose); `getSkill` unknown id → error.

**CONTENT DECISION (user-approved):** `BEAUTIFUL`/`PLAIN` were referenced in 17 prose sites but never
registered as traits — the old lenient snapshot silently rendered them `false` (dead else-branches);
the strict unified accessor surfaced this. **User chose to rewrite** (not register inert traits): all
17 sites now use graded `w.getAppearance()` checks (BEAUTIFUL → Beautiful/Stunning/Devastating; PLAIN →
Plain). Prose text unchanged, only the condition expression. NOTE: these branches were DEAD before and
now FIRE for Beautiful+/Plain PCs — confirm the threshold (exact-vs-graded) matches intent if revisited.

**Verification (all green):** full `cargo test --workspace`; `validate-pack` clean (74 scenes, prose
gate active); `cargo clippy -p undone-scene` clean except 4 PRE-EXISTING `Arc`/`sort_by_key` lints
(compiled.rs/simulator.rs/loader.rs:117 — untouched); `cargo fmt --all` clean. 8 independent
acceptance tests (ops:test-author). **Code-reviewer PASS** on accessor-lift fidelity. **Playtester
PASS** — 18 scenes, prose renders correctly, NPC names resolve to display name ("Jake" not spawn),
Stunning appearance branch fires, zero prose errors / zero raw-template leaks.

**NOT DONE — next session:** _(superseded — see the top entry; merge + push are now DONE)_
- ~~**Merge `script-api-registry` → master**~~ — DONE (`e046ffe`), pushed to origin 2026-06-02.
- **Rebuild MCP binaries after merge + session restart:** `pwsh tools/rebuild-mcp.ps1` (the worktree
  built its own `tools/target`; the live `.mcp.json` points at the MAIN repo's `tools/target/release/`).
  The new `minijinja-mcp-server` `jinja_validate_prose` tool only appears after that + restart. _(still
  open unless already run this machine)_

**INFRA ISSUE (flagged by user):** dispatching subagents (ops:code-reviewer, ops:test-author,
playtester) each spawned a FULL MCP server set including rust-analyzer; on subagent exit those
rust-analyzers were ORPHANED. 8 instances accumulated (~8.6 GB) → PC unresponsive. Killed 5; 3 are
stuck/defunct (parents dead, resist kill — need reboot). **Proposed fix:** restrict subagents' MCP
servers via agent `mcpServers` frontmatter (as scene-writer/writing-reviewer already do — they only
connect `minijinja`). The ops:* agents (global plugin) and ad-hoc subagents inherit the project
`.mcp.json` (all 5 servers). Options: (a) add `mcpServers: []` or a minimal allowlist to the ops agent
defs; (b) a project setting so spawned subagents don't auto-start MCP servers. Decide deliberately —
touches global ops config. NOT yet implemented.

**Latest session (2026-06-02, story-map authoring tool — SHIPPED):**
Built the `story-map` CLI per `docs/plans/2026-06-02-story-map-tool.md` (branch `story-map`).
It derives the scene-connectivity graph from base-pack data, reconciles it against an
authored `packs/base/roadmap.toml`, and emits `docs/story-map.{md,json}` — a writer-facing
map answering "what exists, what connects, what to write next."

- **New:** `src/story_map.rs` (logic), `src/bin/story_map.rs` (CLI), `packs/base/roadmap.toml`
  (7 threads, all 74 shipping scenes claimed — 0 orphans), `tests/story_map_acceptance.rs`,
  `.gitattributes` (forces LF on the generated docs so `--check` is stable under
  `core.autocrlf=true`).
- **Engine touch-ups:** `reachability::{required_game_flags, arc_state_eqs}` pub scan
  wrappers; `Scheduler::bindings()` + `SceneBinding` projection for authoring tools.
- **Model:** flags and `ARC=STATE` are uniform *signals*. A scene *produces* (effects) and
  *requires* (gates) signals. Dangling = produced-but-never-required (open door, write a
  follow-up); broken = required-but-never-produced (unreachable gate). The `write_next`
  digest ranks dangling > broken > planned.
- **Story-map tool** (`cargo run --bin story-map`) — regenerate after any content change.
  `cargo run --bin story-map -- --check` guards staleness (exit 1 if the committed map is
  stale). First run: 7 threads, 84 write-next items, 0 orphans.
- **Verification:** 11 story_map unit + 8 acceptance + 10 independent-acceptance tests green
  (29 total); `--check` clean; validate-pack still "All checks passed (74 scenes)". Independent
  blind code-review found 3 real defects (action-condition gates unscanned → false dangling;
  preset starting flags ignored → false broken gates; explicit thread lists stolen by
  flag_prefix) — all fixed + locked with tests, re-review PASS. Independent QA (CLI) PASS.
  scene-writer agent wired to read `docs/story-map.json` and pick `write_next` items.
- **Known minors (non-blocking, optional follow-ups):** (1) a flag set and read only via a
  negated within-scene self-guard (e.g. `LUNCH_WITH_MARCUS`) shows as dangling — technically
  correct under the flags+arcs signal model, mildly noisy as a "write a follow-up" hint.
  (2) Orphan scenes' internal dangling/broken findings aren't computed until the scene is
  assigned to a thread (the orphan itself is loudly reported with a fix hint first).

**Previous session (2026-06-02, looping-adult REVIEW + improvements — fan-out review, then full improvement pass):**
Reviewed the just-merged looping-adult layer with a 4-agent fan-out (3 writing-reviewers on the
6 un-reviewed scenes + 1 engineering code-review of the desire/composure engine), then executed a
full improvement pass on branch `looping-adult-polish` (merged to master). Playtester-verified.

**Engineering:**
- Code-review verdict **PASS** — no correctness defects. Fixed the one real latent bug it found:
  COMPOSURE (a v7 structural skill) was absent from pre-v7 saves → loaded as composure **0** (max
  loss-of-control), silently unlocking the reckless content gates. `start_loaded_game_checked` now
  backfills missing COMPOSURE to `STARTING_COMPOSURE` (the const made `pub` for single-source);
  new test `loaded_save_missing_composure_skill_is_backfilled_to_starting_value`.
- **Player-facing Desire + Composure sidebar meters** (was dev-IPC-only — the layer's whole point
  is a *felt* need-state). `PlayerSnapshot` gained `desire`/`composure`; composure via a new cached
  `GameState.composure_id` (mirrors `femininity_id`), desire from `game_data`; Composure row under
  Femininity, Desire under Arousal in `right_panel`. Test covers the snapshot refresh.
- `marcus_repeat_office` BACK_ARCHER ordering bug (trait climax-response fired *after* the finish
  line) reordered to match `give_in_he_moves`. Doc notes (rule 10): `advanceTime` accrues desire;
  `desire_scaled` weight-0 footgun.

**Content:**
- **6 scene revisions** (scene-writer fan-out + line-by-line diff review): cut narrated-interiority/
  body-analysis, staccato closers, over-naming; added FEMININITY-calibration branches across
  jake_repeat_night, jake_morning_quick, marcus_repeat_office, marcus_leverage, desire_solo_night,
  desire_ambush. Playtester confirmed low-FEM vs high-FEM read genuinely different in-game.
- **Orgasm-verb spelling sweep (come → cum)** across all explicit scenes — 70 replacements in 14
  files (climax sense only; `came` and the orgasm-as-subject "the orgasm comes" preserved, as do
  all motion/arrival uses). New permanent house-style rule.

**Tooling (after-writing-audit feedback):** writing-guide + scene-writer + writing-reviewer get the
`cum` rule; `prose-lint` gains an `orgasm_spelling` check + the recurring interiority tells
(internal-faculty naming, "the body makes a case", "noted and filed"). prose-lint self-test 18/18.

**Verification:** full workspace `cargo test` green; undone-ui 112 tests; validate-pack clean (74
scenes); intro render-regression green; cum-sweep diff reviewed line-by-line; **playtester PASS** —
meters present + updating live (Desire 0→100, Composure 60→20), all 6 revised scenes render with
zero errors at both FEM levels, cum spelling correct in-game. Dropped one false review finding (a
"duplicated Marcus simile" that didn't actually exist in marcus_leverage).

**Follow-ups (not blockers):**
- ~~Suspected em-dash → "??" rendering artifact~~ **INVESTIGATED — false finding, no bug.** Source is
  U+2014 (`E2 80 94`); the data path is UTF-8 end-to-end with no transcode (`markdown_to_text_layout`
  passes text through unchanged); the prose font chain ("Literata, Palatino, Georgia, serif") resolves
  to Georgia, which is installed and has the glyph; and a screenshot viewed directly shows clean
  em-dashes. The "??" was the original playtester's vision model misreading its own screenshot, and the
  `?` in dev-IPC/console output is a cp949 (Korean Windows) console-codepage artifact — neither reflects
  what the game renders. Nothing to fix.
- **Dev `set_stat` can't set `desire`/`composure`** (only money/stress/anxiety/femininity) — a
  playtest-tooling gap; adding them to `dev_ipc.rs set_stat` (desire → game_data, composure → skill)
  would make future playtesting of this layer far easier.
- The prior session's deferred **sidebar meters** and **writing-review of the remaining scenes** are
  now both DONE. Still open from that list: DUBCON/BLOCK_ROUGH tagging judgment call on
  `gym_regular_first` "stay" + `marcus_pushes` "let_him"; `jake_seeks_more` gates on built skills.

**Previous session (2026-06-01, LOOPING-ADULT LAYER — engine + 12 scenes, director-driven fan-out):**
Shipped the looping-adult content layer on branch `looping-adult-layer` (design spec:
`docs/plans/2026-06-01-looping-adult-layer-design.md`). The adult side was all terminating
one-shots; this makes desire recur and escalate.

**Engine (TDD, all green):**
- **DESIRE** need-state: `GameData.desire: BoundedStat` (0–100), accrues +8/consumed-slot in
  `advance_time_slot`, discharged by release scenes. Rhai: `gd.desire()` read, `gd.addDesire`/
  `gd.setDesire` writes (+ minijinja template ctx + load-validation gate).
- **COMPOSURE** promoted to a structural skill (registry const `composure_skill()` + load
  validation, like FEMININITY), seeded 60 at new-game; `w.composure()` / `w.changeComposure(n)`
  (giving in lowers it). Spiral is content-gated (low composure unlocks reckless variants).
- **Scheduler desire bias**: per-event `desire_scaled = true` scales weight 1×..4× by desire
  (`effective_weight`/`desire_multiplier`). Data-driven — engine never decides what's "adult".
- Save **v6 → v7** (no-op migration; `desire` serde-defaults). Desire+composure surfaced in
  both dev-IPC snapshots (`GameStateSnapshot`, `RuntimeSnapshot`).

**Content (12 scenes, parallel scene-writer fan-out, all pass load gate):**
- **Cal / gym-regular** (NEW NPC `ROLE_GYM`, `docs/characters/cal.md`): `gym_regular_intro`
  (hook) → `_recurs` (repeatable) → `_first` (done-to-her) → `_deepens` (repeatable, GYM_ACT_*
  ladder). Power-inversion + submission.
- **Jake** repeatables: `jake_repeat_night`, `jake_morning_quick`, `jake_seeks_more` (she-seeks,
  JAKE_ACT_* unlocks).
- **Marcus** affair: `marcus_repeat_office`, `marcus_pushes` (MARCUS_ACT_* escalation),
  `marcus_leverage` (cost — MARCUS_TERMS_HERS/HIS/AFFAIR_COOLING outcomes).
- **Desire**: `desire_solo_night` (release valve, discharges), `desire_ambush` (no discharge).
- All wired into `schedule.toml` free_time/work slots with desire_scaled + npc_role bindings.

**Verification (all gates passed):** full workspace `cargo test` green; `validate-pack` clean
(74 scenes). **Render regression test** added (`all_scene_intros_render_without_missing_methods`)
— the playtester caught 8/12 scenes crashing at render because `gd.desire()`/`w.composure()`
were in the Rhai API but NOT the minijinja template context (validate-pack doesn't render prose);
fixed + guarded. **Independent test-author** (ops:test-author, did NOT see implementation
reasoning) wrote 19 acceptance tests across `desire_composure_acceptance.rs` /
`desire_save_compat.rs` / `composure_required_id.rs` covering all 10 criteria (desire accrual/
clamp, composure seed/clamp, scheduler desire-bias over seeded iterations against the REAL
schedule, Rhai+minijinja accessor parity, v6→v7 save back-compat, COMPOSURE required-id) — all
PASS. **Writing-reviewer** ran on Cal thread + marcus_pushes/jake_seeks_more — CRITICAL
narrated-power-inversion ("you've done this / used to feel like a cliff") and narrated-interiority
fixes applied (show the body, never explain). **Focused playtester re-run CONFIRMED in-game:**
all 7 previously-broken scenes render clean, repeatable scenes genuinely vary by desire (12 vs
100 = different psychological states, not word-swaps), give-in lowers composure and release
discharges desire (verified before/after via dev-IPC), explicit prose "delivers." Docs updated
per rule 10 (content-schema, engine-design). Merged to master.

**Deferred (not blockers):**
- **Player-facing sidebar meter** for desire/composure — only in dev-IPC snapshots so far; add
  Desire/Composure rows to `right_panel.rs` `stats_panel` + `PlayerSnapshot` (needs a cached
  `composure_id` on GameState like `femininity_id`, and desire from `world.game_data`).
- **Writing-review** the remaining 8 scenes (only Cal + 2 reviewed); same writers/brief, but
  watch for the narrated-interiority pattern that recurred.
- **DUBCON content-level / BLOCK_ROUGH** judgment call on `gym_regular_first` "stay" +
  `marcus_pushes` "let_him" (reviewer flagged; needs user direction on tagging).
- **jake_seeks_more** escalation actions gate on built sexual skills the player may not have —
  consider lowering gates or adding skill-building paths.

**Previous session (2026-05-31, queued engineering cleanup continued):**
Read the live handoff first; `AGENTS.md` was not present at the repo root, so the
instructions supplied in the session prompt were followed. Ran the requested queue
commands: `desloppify status`, `desloppify next`, and
`desloppify show review --status open --no-budget`. Completed two concrete review
fixes and resolved both exact plan items:

1. `scheduler_duplicate_pick_logic` — refactored `crates/undone-scene/src/scheduler.rs`
   so `pick()`, `check_triggers()`, and `pick_next()` share `ScheduleCandidate`
   helpers for once-only filtering, condition/trigger evaluation with slot-aware
   warning context, weighted selection, and `PickResult` construction. This keeps
   single-slot and global scheduling from drifting.
2. `runtime_slot_start_panics` — made requested-slot starts fallible in
   `crates/undone-ui/src/runtime_controller.rs`, removed the `expect()`, routed
   scheduled starts through one helper shared by `pick_next`, and added a regression
   test proving a scheduler/content mismatch returns an `Unknown scene` command error
   from `choose_action` instead of silently succeeding through engine error prose.

**Verification:** `cargo fmt --check --all`, `cargo test -p undone-scene`, and
`cargo test -p undone-ui` passed (undone-ui now 111 tests). `desloppify next` now
points at subjective re-review/score refresh rather than a concrete fix. Remaining
open concrete subjective review items are `registry_mutation_api_too_broad` and
`dev_ipc_mixes_transport_and_commands`; both are broader refactors and were left
queued rather than forced as low-risk cleanup. The Rhai MCP release rebuild was not
attempted; prior lock warning still stands unless a session restart has released
`tools/target/release/rhai-mcp-server.exe`.

**Latest session (2026-05-31, subjective review pass + first fixes):**
Ran the queued subjective review workflow. `desloppify review --prepare` succeeded, but
`desloppify review --run-batches --runner codex --parallel` is currently blocked by the
local Codex runner emitting an invalid quoted config value for `model_reasoning_effort`
(`\"low\``). Imported a manual, evidence-backed subjective review from the blind packet
instead; scores are explicitly **manual/provisional** and will reset on the next scan unless
replaced by a trusted review run. Current strict score after the pass/fixes is **82.4/100**
(objective **79.1/100**), with four open subjective review items remaining:
`scheduler_duplicate_pick_logic`, `runtime_slot_start_panics`,
`registry_mutation_api_too_broad`, and `dev_ipc_mixes_transport_and_commands`.

Fixed three subjective review items:
1. `runtime_snapshot_duplicate_npc_shapes` — `BoundActiveNpcSnapshot` now flattens the
   shared `ActiveNpcSnapshot`, preserving the JSON shape while removing duplicated NPC
   display fields and routing bound/unbound NPC conversion through one helper.
2. `dev_ipc_silent_result_failures` — dev IPC now writes/replaces the result before
   command cleanup, logs pathful persistence failures, and keeps command input available
   when result persistence fails; added two poll-path tests.
3. `loaded_game_missing_femininity_panics` — added `start_loaded_game_checked()`, routed
   save loading through it, and shared the required `FEMININITY` lookup with new-game paths.

**Verification:** `cargo fmt --check --all` passed. `cargo test -p undone-ui` passed
(110 tests, 0 failed). Earlier in this same session, `cargo test -p undone-scene`,
`node --test tools/deepseek-helper.test.mjs`, `node tools/deepseek-helper.mjs --help`,
and `cargo run --bin validate-pack` also passed. The `rhai-mcp-server` release rebuild is
still blocked by the live process locking `tools/target/release/rhai-mcp-server.exe`.

**Latest session (2026-05-31, straightforward queued cleanup):**
Tried the queued `rhai-mcp-server` release rebuild first, but the binary swap is still
blocked by the live MCP process holding `tools/target/release/rhai-mcp-server.exe`
(`os error 5`). No forceful process cleanup was attempted. Completed the next clean
engineering follow-up instead: extracted the large inline `engine.rs` test module into
`crates/undone-scene/src/engine/tests.rs`, leaving production `SceneEngine` code in
`crates/undone-scene/src/engine.rs` (now **768 LOC**) with only `#[cfg(test)] mod tests;`
at the bottom. Behavior-preserving module split; test code remains under
`engine::tests` with `use super::*`.

Added retry/backoff to `tools/deepseek-helper.mjs` (3 attempts, exponential
backoff for transient HTTP/network failures) with a Node built-in test covering
429→500→success behavior; the CLI remains side-effect-free when imported and
still supports `--help`. Added `docs/writing-delegation.md`, a repo-neutral
writer/reviewer dispatch contract, closing the last concrete writing-agent
tooling audit item; only optional prompt-wrapper/local-dedupe polish remains.

**Verification:** `cargo fmt --all`, `cargo fmt --check --all`, and
`cargo test -p undone-scene` passed (150 unit tests plus the related
integration/doc-test groups, 0 failed). The two Rhai acceptance test files
received rustfmt-only formatting changes. `node --test tools/deepseek-helper.test.mjs`
and `node tools/deepseek-helper.mjs --help` passed. `cargo run --bin validate-pack`
passed with the known prose warnings still present. **Still blocked:** rebuild
`rhai-mcp-server` after a session restart with
`cd tools && cargo build --release -p rhai-mcp-server`.

**Latest session (2026-05-30, god-file refactor — char_creation.rs split, MERGED):**
Split the 2636-LOC `crates/undone-ui/src/char_creation.rs` god-file into 6 focused
submodules under `crates/undone-ui/src/char_creation/` (parent now **925 LOC**, holding
only the public-entry-point views + the test module). Submodules: `contracts.rs`
(validate_registry/runtime/startup_contract), `config.rs` (preset/config building,
resolve_starting_traits, fem_form_defaults, robin_quick_config), `signals.rs`
(BeforeFormSignals/FemFormSignals + init-error IO), `sections.rs` (BeforeCreation form
sections), `buttons.rs` (build_next_button/build_begin_button), `widgets.rs` (shared
themed widgets). **Behavior-preserving** — extraction was verbatim (verified byte-identical
bodies + prose modulo rustfmt signature reflow). Parent kept as `char_creation.rs` (Rust
2018 allows `foo.rs` + `foo/` dir), so the cross-crate public API path
(`undone_ui::char_creation::{validate_*_contract, robin_quick_config, resolve_starting_traits}`,
consumed by `src/validate_pack.rs` + `tests/preset_integration.rs`) is unchanged via
`pub use` re-exports; internal items are `pub(crate)`. Test-only imports (Appearance,
BeforeSexuality) live inside `mod tests` to keep the lib-only clippy gate clean.

**Verification (all gates passed):** full workspace `cargo test` green (107 undone-ui +
all crates, 0 failed), `cargo clippy --workspace` clean for the changed files (6 pre-existing
`arc_with_non_send_sync`/`sort_by_key` warnings elsewhere are untouched), `cargo fmt --check`
clean for the changed files, validate-pack clean ("All checks passed. 62 scenes."). **Code
review PASS** (independent multiset body-comparison confirmed 54 functions both sides, no
logic/prose drift, visibility correctly scoped). **Playtester PASS** — full char-creation
flow (Robin/Raul presets + Custom form with all dropdowns/checkboxes/chips, all 5 FemCreation
discovery beats, Begin→InGame transition) renders and functions with no regressions.
Merged fast-forward to master (6 commits, `5266ba8`); worktree + branch cleaned up.

**Still blocked (session-restart):** rhai-mcp-server binary swap — source compiles clean,
but `tools/target/release/rhai-mcp-server.exe` is locked by the live MCP process (`os error 5`);
run `cd tools && cargo build --release -p rhai-mcp-server` after a session restart.
Harmless leftover: an empty `.claude/worktrees/` dir (gitignored; transient handle lock).

**Latest session (2026-05-29, maintenance + gap-closing pass):**
Janitorial + one engineering gap closed. Working tree cleaned: 17 throwaway
playtester screenshots moved to `~/.claude/trash/undone-screenshots-2026-05-29`;
`.gitignore` hardened so session artifacts never recur (`/​*.png` at root, `.codex/`,
`.claude/scheduled_tasks.lock`, the synced `.claude/skills/desloppify/` — the last
three placed *after* the `!.claude/**` negation so they actually take effect).
**Reachability false-positive fixed:** `check_reachability` now takes the set of
preset `starting_flags`, so a `hasGameFlag` gate on a flag a preset seeds (e.g.
`ROUTE_CAMPUS` from the Camila preset) no longer reads as unreachable. This cleared
all 11 `ROUTE_CAMPUS` warnings — **validate-pack reachability is now completely clean**
(only the 27 prose-heuristic warnings remain: 20 player_action_in_intro + 6
player_speech_in_intro + 1 filler_action — all content-side, creative-gated). New
unit test `flag_satisfied_by_preset_starting_flag_passes`. Verification: undone-scene
165 tests pass, validate_pack_simulation + all integration suites pass, full
`cargo test` exit 0, validate-pack exits 0 "All checks passed. 62 scenes." Two
commits: `c900cca` (gitignore) + `b11dd1a` (reachability fix; docs updated per
principle 10). **Known follow-up unchanged from last session:** rebuilding
`rhai-mcp-server` to pick up the new condition/effect validation tools is still
blocked by the live MCP process holding a lock on `tools/target/release/rhai-mcp-server.exe`
— the source compiles clean (`cargo check` green); the binary swap will succeed on
the next session restart (run `cd tools && cargo build --release -p rhai-mcp-server`).
The local `.claude/settings.local.json` MCP-toggle diff (rust server off) is an
intentional machine-local change, left uncommitted.

*Tech-debt sweep (same session):* Closed the two "small engineering" items.
**Audit I11 (cache FEMININITY SkillId)** was already done — `GameState.femininity_id`
is the cache; no hot string lookup exists (stale HANDOFF entry, now marked).
**Audit I13 (process_events test coverage)** shipped as commit `b2dacca`: 9 unit
tests covering every `EngineEvent` arm, the `scene_finished` return, the NPC
known-context merge guard (both directions — proven non-vacuous by temporarily
mutating the production guard and watching the test fail, then restoring), and the
player-snapshot refresh. undone-ui suite 97→107, all green. Production code untouched
(190-line pure test addition). Remaining small item: the lone `work_marcus_drinks:55`
filler_action prose nit (creative, rule-11-gated).

**Latest session (2026-05-29, Phase 1 Rhai foundation — COMPLETE, all 11 tasks shipped):**
Replaced the custom `undone-expr` condition parser AND the closed `EffectDef` enum with
embedded **Rhai**, invisibly to players. Executed `docs/plans/2026-05-29-phase1-rhai-foundation.md`
end to end (~14 commits on branch `phase1-rhai`, merged to master).

New `crates/undone-scene/src/script/` layer: two engines (read-only `cond` + read/write
`effect`) cached per-thread; full read API (6 receivers `w/gd/m/f/role/scene`, ~80 methods);
full write API (`w.*` player, `gd.*` game-data, `scene.setFlag/removeFlag`,
`npc("m"|"f"|role).*`) with **continue-on-error** semantics; and a load-time fail-fast
**source-scan gate** (`script/validate.rs`) that reconstructs the legacy
`validate_call_signature` + `validate_condition_ids` + `validate_effects` across ALL
branches (catches a typo'd id even in a short-circuited branch — stronger than a runtime
dry-run). Conditions → `Option<CompiledScript>`; effects → `Option<CompiledScript>` (single
`effect = '…'` Rhai call-list per action). `reachability`, `references_game_flag`, and
`has_persistent_world_mutation` migrated to source scans (full parity). `undone-expr`
**deleted**; `SceneCtx`/`SceneNpcRef` moved into `undone-scene::scene_ctx`. rhai-mcp-server
realigned: new `rhai_validate_condition`/`rhai_validate_effect` tools run the real gate.

**Resolved decisions:** borrow-bridge = thread-local raw-ptr eval context (bench-justified,
within ~2% of the alternative); static analysis = **source-string scan** (not Rhai
`internals`); Rhai **single-quotes are CHAR literals** so all condition strings migrated
single→double quote (the design's "near-verbatim" claim was wrong — scene+schedule TOML
migrated, effects via `tools/migrate-effects.py`).

**Verification (all green):** full workspace `cargo test` = 420 passing, 0 failed;
`validate-pack` loads 62 scenes clean (35 reachability warnings = parity, no errors) and
rejects a typo'd content id at LOAD (acceptance test `crates/undone-scene/tests/rhai_parity.rs`);
rhai-mcp-server 10 tests; **playtester confirmed complete behavioral parity** — 13-scene
Robin→Jake playthrough (opening arc → settled → coffee_shop → jake_apartment explicit chain),
every effect landed (femininity 10→46, money/arousal/flags/arc-states/NPC-liking/relationship,
Jake naming in sidebar), explicit content rendered, zero error text. The game plays identically.

**Follow-ups (not blockers):** the running rhai-mcp-server uses the pre-built binary at
`tools/target/release/`; rebuild it (`cd tools && cargo build --release -p rhai-mcp-server`)
to pick up the new condition/effect validation tools. Phase 2 (vertical-slice explicit scene
with a fluid-composure resist check) is BLOCKED on a creative scene spec from the user.

---

**Latest session (2026-05-29, Rhai+fragment architecture design + Phase 0 pacing fix shipped):**
Full brainstorming pass on the Rhai-scripting + Disco-Elysium fragment-engine pivot. Two design
artifacts committed: `docs/plans/2026-05-29-rhai-fragment-architecture-design.md` (approved spec,
5 phases) and `docs/plans/2026-05-29-phase0-pacing-fix.md` (Phase 0 plan). Key reframe from the
design-research pass: the DE check mechanic ALREADY EXISTS (`checkSkill`/`checkSkillRed` + roll
math + `red_check_failures` + `FailRedCheck` in eval.rs/effects.rs), and the fragment model is a
unification of the existing intro_variants/thoughts/actions/npc_actions primitives — so the pivot
is far smaller than it looked. Decisions locked: Rhai replaces BOTH undone-expr AND the EffectDef
enum; effects = constrained call-list; checks use a fluid COMPOSURE stat (giving in lowers it →
harder to resist next time); 62 scenes NOT migrated (compatibility path); save-scum prevention
deferred.

**Phase 0 (adult-content pacing fix) SHIPPED to master.** Resolved open question O5 by reading
`scheduler.rs` `pick_next`: triggers fire BEFORE the weighted pool, scanning slots ALPHABETICALLY
(`free_time` < `workplace_opening`). So `coffee_shop`'s old `week>=2` floor was load-bearing — it
kept the romance chain from hijacking the opening arc. Fix: re-anchored `coffee_shop` and
`bar_closing_time` on `arcState('base::workplace_opening') == 'settled'` (ordering-safe, calendar-
free) and stripped `gd.week()>=N` floors from the Jake liking-builders + `jake_first_date`/
`jake_second_date`/`jake_apartment` triggers (the story-flag chain already enforces order). Added
2 scheduler regression tests (opening-arc precedence + calendar-free reachability); undone-scene
suite 166→168, all green; validate-pack clean (62 scenes). **Playtester acceptance: first explicit
content now reachable Day 1 (was week 4+, ~27-day reduction), opening arc order intact, no bugs.**
Commits: `8b65cb2` (schedule), `85196d0` (tests), plus the two doc commits.

**Content follow-up flagged (not a blocker):** `jake_apartment` intro says "The wanting has been
there for weeks" — reads oddly now that the scene is reachable in ~1 day. Re-anchor that line for
the faster pacing in a future content pass.

**Next:** Phase 1 (Rhai foundation) implementation plan being authored. Phase 2 (vertical-slice
explicit scene) is blocked on a creative scene spec from the user (rules 1 & 11).

**Previous session (2026-05-17, prose surgery — scale-anchor repetition):** Three scenes (`gym_changing_room`, `marcus_apartment`, `jake_stays_over`) all used the "scale of him vs you" body-observation as their FEMININITY-gated anchor. `gym_changing_room` owns it structurally (women's-only locker room — scale-against-the-space *is* the scene), so the other two were re-anchored on the registers each is actually built around:
- `marcus_apartment`: visibility / deliberateness / lit-room (doorman saw her, elevator knew his floor, lamp on vs. the dark closet, he keeps his eyes on her). Three FEMININITY<25 branches edited.
- `jake_stays_over`: territorial integration (his jacket on her chair, his phone on her nightstand, quiet drawer-sounds; in the kitchen he moves the careful way you move in someone else's). Two FEMININITY<30 branches edited.

writing-reviewer + playtester both ran post-edit. Playtester confirmed live at FEMININITY 10 that all four edited branches render correctly and the new anchors land. Pre-edit issue list and the one rhythm-echo I introduced were both addressed in the same change set. **Two pre-existing prose issues surfaced during audit and are out of scope for this surgery:** narrator body-analysis "the body has already filed all of this before anything rational has caught up" (marcus_apartment cut_to_it), and emotion-announcement "warm and low has settled in your stomach" (marcus_apartment intro). Logged for a future content pass — neither was introduced by this change.

Also: committed a stray dev-dependency pin on `undone-scene/Cargo.toml` left uncommitted from the prior test-author session (set_npc_name_tests needs `undone-save` + `serde_json` as dev-deps; without them the suite fails to compile). 498 tests passing, validate-pack clean.

**Previous session (2026-05-17, cleanup + feedback triage + display-name feature):** Audited every open feedback item against actual code. Most engineering-audit Criticals/Importants were already resolved silently; remaining open items mostly content-side or low-leverage. 7 commits:
1. Finalized in-progress cleanup pass: pruned 9 dead rust-mcp stub tools (~1500 LOC), extracted `UI_FONT_FAMILY` constant, deduplicated `make_test_male_npc` test helper, dropped unused `anyhow`/`slotmap` deps.
2. Fixed ROLE_THEO NPC routing through the scheduler (`npc_role` was missing on the three Theo schedule entries); also dropped the redundant `set_game_flag ROUTE_*` writes in arrival scenes (audit I18).
3. Fixed `ThoughtAdded`/`ErrorOccurred` scrolling to bottom on the first event of a new scene — should respect the new-scene-top invariant the way `ProseAdded` already did (audit I8).
4. `validate-pack` now skips underscore-prefixed scene subdirs (matches the runtime loader's effective scope; drops two dead lint warnings from `_archive/`).
5. **Feature: `SetNpcName` effect + `NpcCore.display_name`** — root-cause fix for "sidebar shows random spawn name (Brian) instead of the story name (Jake)". `NpcCore` gained an `Option<String>` display name with `effective_name()` accessor; new `set_npc_name` effect writes it; `NpcActivatedData::from_npc` and the prose `getName` accessors all read through `effective_name()`. Decoupled from `set_npc_role` so a scene can rename without rebinding and vice versa. Save format bumped v5 → v6 (no-op JSON shape, `#[serde(default)]` on the field). Wired into `coffee_shop` (Jake), `workplace_work_meeting` (Marcus), `campus_library` (Theo).
6. Independent acceptance suite for `SetNpcName`: 27 tests via `ops:test-author`, all passing, each with a `// BREAKS IF` comment naming the user-visible behavior it guards.

**Playtester confirmed live:** scheduler-driven `coffee_shop` at week 2 fires `set_npc_role` + `set_npc_name`; spawn-name "Derek" becomes display-name "Jake"; the People Here sidebar reads "Jake" in the next scene (`plan_your_day`).

**Workspace state:** 498 tests passing (was 470), `validate-pack` clean (one pre-existing content nit: "check your phone" in `work_marcus_drinks` line 55), save format at v6.

**Audit triage outcome:** of the ~45 engineering-audit findings, 8/8 Criticals were resolved before this session, ~14/19 Importants resolved, the rest are either intentional (unused future-hook stats), low-leverage (FEMININITY string lookup), or content work (post-arc void, specific scene rewrites). The HANDOFF's prior "People Here doesn't populate for Theo" complaint was a class-level engine gap that also affected Jake and Marcus; root cause was the missing display-name mechanism, now fixed.

**Previous session (2026-03-22, fourth pass — consequences sprint, 7 new scenes):** Two batches. Batch 1 (4 scenes): `marcus_apartment` (explicit — Marcus's second encounter, deliberate, full sexual response traits, MARCUS_REJECTED flag for "leave"), `campus_theo_morning` (Camila wakes in Theo's bed — HOMOPHOBIC morning-after), `gym_changing_room` (Robin in women's locker room — OBJECTIFYING branching strongest), `jake_stays_over` (domesticity — Jake's morning routine in Robin's apartment). Enhanced `morning_routine` and `evening_home` with post-sexual texture (mirror after sex, underwear decisions, Jake text, bath scene, Marcus office awareness). Batch 2 (3 scenes): `marcus_monday_rejected` (consequence — Marcus's professional distance after Robin left his apartment), `bad_date` (Ryan from the app — three exit strategies, OBJECTIFYING reads the script he's running), `campus_dining_after_theo` (Camila sees Theo in public — HOMOPHOBIC visibility of desire). Writing-reviewed batch 1, fixed Important findings. **62 scenes total. All tests passing. validate-pack clean.**

**Previous session (2026-03-22, third pass — adult content sprint + Camila playtest):** Deepened all 4 existing explicit scenes (jake_apartment, bar_stranger_night, work_marcus_closet, party_stranger_after) with Robin's 18 sexual response traits — HAIR_TRIGGER, NIPPLE_GETTER, SENSITIVE_NECK, EASILY_WET, SUBMISSIVE, PRAISE_KINK, MULTI_ORGASMIC, HEAVY_SQUIRTER, and 10 physical detail traits. Each trait produces structural physical changes (different pacing, different events), not adjective swaps. Deepened jake_morning_after "wake him up" action into explicit morning sex (sleepier, bodies that know each other, full trait integration). Created campus_theo_night — Camila's first sexual encounter with Theo, using HOMOPHOBIC (desire-before-shame) and SEXIST (male-gaze inversion) to produce a scene Robin couldn't have. Added MET_THEO flag to campus_library and schedule entry for campus_theo_night. Playtested Camila flow — playtester confirmed discovery beats strong (Beat 1 Scale + Beat 5 HOMOPHOBIC strongest), campus arc well-constructed, dining hall backpack moment "most emotionally potent thing in the arc", Theo night "genuinely hot." Fixed 2 bugs: (1) campus_orientation NPC action had `goto = "engage"` (action ID, not scene ID) → `finish = true`; (2) CRITICAL: "Begin Your Story" crashed the game — `phase.set(AppPhase::InGame)` inside `on_click_stop` caused floem reactive panic. Fixed by deferring with `exec_after(Duration::ZERO)` in 3 locations (char_creation begin, error surface, landing page load). **446 tests passing, 55 scenes, validate-pack clean.**

**Previous session (2026-03-22, second pass — tools, playtest, campus):** Restored tools/ from git, rebuilt all 5 MCP servers. Fixed start_game focus-steal: now launches pre-built binary directly with DETACHED_PROCESS+CREATE_NEW_PROCESS_GROUP flags instead of cargo run. Built game release binary. Playtested Robin opening flow (workplace_arrival through week 1) — playtester confirmed: discovery beats are "the best writing in the game", explicit scenes land, DM voice is consistent, ID checkpoint is best choice design. Fixed `style = "inner_voice"` → `"observation"` across all 29 non-archive scenes. Wrote Camila's 5 discovery beats (Scale→Body→Face→Name→Sexual, trait-branched: CONFIDENT/AMBITIOUS/OUTGOING/SEXIST/HOMOPHOBIC). Rewrote all 7 campus scenes in DM narrator voice (parallel scene-writer agents). **446 tests passing. validate-pack clean.**

**Previous session (2026-03-22, prolific writing sprint):** Complete voice rewrite of all 47 Robin scenes + 5 Robin discovery beats. Voice rules hardened: NO inner voice at all. 18 parallel scene-writer agents across 6 phases. Explicit scenes written direct. Opening arc has full memory integration (7 flag stacks).

**Previous session (2026-03-19, full session):** Infrastructure audit → Track B (presets as TOML) → Track C (simulator cadence verified) → FemCreation discovery scaffolding → skill/doc updates. **446 tests passing.** Presets now TOML data files loaded by undone-packs. Discovery beats render minijinja prose through throwaway World. scene-writer/writing-reviewer agents updated for pipeline v2.

**Previous (2026-03-19, infrastructure audit + cleanup):** Verified full infrastructure health. Built MCP servers. Cleaned worktrees. Committed uncommitted pipeline files. 3 modified tracked files (scene-writer.md, writing-reviewer.md, pack-prompt.mjs) accidentally reverted during cherry-pick cleanup — minor loss (pipeline v2 agent wiring).

**Latest session (2026-03-16, dead-space smoke + padding dedupe):** Closed the remaining UI-correctness engineering gap from the prior sweep. `undone-ui` now derives action-bar side padding from the same shared layout constant used by the responsive width math, so the live chrome and layout budget cannot silently drift. `game-input-mcp` now has a reusable dev-IPC client + runtime audit helpers plus a new Windows smoke binary, `ui-dead-space-smoke`, that builds/releases the app, launches `undone --dev --quick`, sends native Win32 pointer clicks into the real window, and asserts through the existing runtime contract that title-bar dead space and bottom continue-bar dead space do nothing while the visible `Continue` control still advances the scene. Verified with fresh `cargo fmt --all` (root + `tools`), `cargo test -p undone-ui -- --nocapture` (98 passing), `cargo test --manifest-path tools/Cargo.toml -p game-input-mcp -- --nocapture` (10 passing), and `cargo run --manifest-path tools/Cargo.toml -p game-input-mcp --bin ui-dead-space-smoke` (passed twice on a fresh release launch). **Main remaining risk:** the pointer-level audit is still a targeted Windows smoke, not an independent QA/code-review pass across all affected surfaces. **Next follow-up should focus on:** independent QA/code review, with optional smoke expansion only if new title-bar/tab or empty-region regressions are observed.

**Latest session (2026-03-16, UI correctness sweep merged):** Merged first-pass UI correctness hardening to `master`. Improved: bottom-bar `Continue` hitbox ownership, responsive action-bar width/row calculations, runtime acceptance coverage for continue-flow and resize-sensitive progression, scene-reset coverage keyed to `scene_epoch`, and deterministic markdown rendering coverage. Verified with fresh `cargo fmt --check`, `cargo test -p undone-ui -- --nocapture` (97 passing), full `cargo test`, and a live release-window audit via dev IPC. **Main remaining risk:** native-window dead-space behavior is still not covered by true pointer-level automation, especially for title-bar/tab chrome and empty click regions. **Next follow-up should focus on:** real dead-space click verification, deduplicating the shared action-bar padding constant between layout math and view styling, and an independent QA/code-review pass rather than more broad UI changes.

**Latest session (2026-03-15, prose pipeline v2):** Built deterministic prose pipeline that separates voice from rules. Claude orchestrates, DeepSeek writes, user prose calibrates — nobody crosses lanes. New tools: `prose-lint.mjs` (regex quality gate — banned phrases, POV, AI-isms, player-acts-in-intro), `prose-to-toml.mjs` (labeled prose + spec JSON → TOML), refactored `pack-prompt.mjs` (few-shot voice samples + tech rules + spec), `scene-pipeline.mjs` (orchestrator: spec → draft → lint → revise → convert). New doc: `docs/writer-tech.md` (mechanical rules only, replaces writer-core.md in prompts). Updated agents: `scene-writer.md` (uses pipeline), `writing-reviewer.md` (uses lint). Voice samples slot ready at `docs/voice-samples/` — empty until user writes samples. Pipeline tested end-to-end with live DeepSeek API call. **Next: user writes voice samples in `docs/voice-samples/`, then pipeline produces calibrated prose at scale.**

**Latest session (2026-03-13, prose reset):** User rejected all existing scene prose. All 54 scene files archived to `packs/base/scenes/_archive/`. One test scene run through DeepSeek — user says it's slightly better but still sounds too much like Claude because writer-core.md (the DeepSeek system prompt) was written by Claude.

**Latest session (2026-03-12, post-playtest fixes):** POV leak sweep: eliminated all third-person "she/her/Camila" references to the player across 9 scene files (campus_arrival, campus_call_home, campus_dorm, campus_library, campus_orientation, campus_study_session, jake_apartment, jake_morning_after, work_marcus_aftermath). Inner-voice fragments reframed from `*X*, she thinks` to `*X.*`. Detail text on action buttons also fixed. NPC binding bug fixed: added `npc_role` field to schedule events so Jake/Marcus scenes bind the correct NPC as active_male instead of picking first slotmap entry. All 12 Jake and Marcus schedule entries now carry their role. All tests pass.

**Latest session (2026-03-12, player-agency-sweep complete):** Player-agency sweep fully merged to master. Phase 1 built automated detection (TOML-aware intro extraction, 50-verb heuristic, 16 prose audit tests). Phase 2 rewrote all 23 affected scenes (79→0 findings): 4 Tier 1 Critical + 19 Tier 2/3. Fix patterns: player speech → indirect/reported, "You [verb]" → passive world-state/noun phrases, extended autopilot → compressed scene-setting, borderline involuntary → reframed without "You" subject. `cargo test player_agency_audit_report` = 0 findings. Worktree removed. Plan at `docs/plans/2026-03-12-player-agency-sweep.md`, final results at `docs/plans/player-agency-audit-results.md`.

**Latest session (2026-03-12, workplace month-one merge):** Workplace month-one foundation work is merged to `master`. The engine now supports authored multi-NPC role bindings through scene evaluation, runtime snapshots, and scene effects. The opening flow was rewritten through `transformation_intro` and the workplace week-one spine, then expanded with persistent opening-memory flags, callback scenes, and stronger week-two carry-forward. Month-one gameplay now has real adult-route state: Jake, Marcus, bar-stranger, and party-stranger explicit paths persist sexual history/virginity state, Jake can pay off in week 4, and the party-outside setup now has a real explicit follow-up. The final audit found and fixed one last mismatch: Jake's `kiss_and_see` apartment payoff now matches the persistent full-intimacy state it sets.

**Latest session (2026-03-12):** Engineering + creative cleanup complete. UI runtime loading now goes through a shared bootstrap path, live character-creation/runtime contract failures surface as recoverable errors instead of panics, and `validate-pack` now exposes a reusable library API with prose-audit coverage shared by tests and the CLI. The targeted scene cleanup landed for the campus cluster plus the known filler/fine-test scenes (`weekend_morning`, `coffee_shop`, `bookstore`, `work_friday`). A post-implementation audit found one additional live-flow bug and fixed it: pack-load/init failures now force the app into the visible in-game error panel instead of leaving the user stranded in the landing flow.

**Latest session (2026-03-11):** Window tooling follow-up complete. `game-input-mcp` now exposes a first-class `set_window_size(width, height)` tool over the existing dev-command path, `RuntimeSnapshot` now serializes live `window_width` / `window_height`, and the Dev tab now supports custom width/height inputs plus a `Default` reset wired to the shared layout defaults. Live acceptance passed on a fresh `undone --dev --quick` launch: `set_tab("dev")` → `set_window_size(1800, 1000)` → `get_runtime_state()` reported `1800x1000`, then `jump_to_scene("base::plan_your_day")` → `advance_time(1)` → `choose_action("go_out")` progressed to a different scene with a new action set while preserving the wide layout in screenshots. Responsive audit of `saves_panel`, `settings_panel`, `title_bar`, `landing_page`, and `char_creation` found no additional reproduced wide-window regressions. One live-only bug found during acceptance was fixed: the Dev tab resize inputs now resync from app-level window signals after external/tooling-triggered resizes.

**Latest session (2026-03-10):** Player-correctness runtime implementation complete. `undone-ui` now has a shared `RuntimeController` and `RuntimeSnapshot`, acceptance-style runtime harness coverage, intro-time NPC fallback binding before scene render, richer dev IPC/runtime MCP wrappers, and a hardened save-loader contract that records the pack-loaded ID prefix and only replays runtime-only interned IDs when the saved/current prefixes still match. Live release smoke passed for `get_runtime_state`, `set_tab`, `choose_action`, and `continue_scene` against `undone -- --dev --quick`. Two live-only regressions found during smoke were fixed: disposed view-local signal reads during tab swaps, and a dev-panel `RefCell` borrow cycle caused by bumping `dev_tick` before releasing the mutable `GameState` borrow.

**Latest session (2026-03-09, second pass):** Conductor autonomous batch — 8 technical debt tasks. Char creation: preset names flow through (no more hardcoded Eva/Ev), before-phase no longer leaks post-transformation traits, traits displayed as categorized chips. SetAllNpcLiking: unit test + MCP wrapper added. Refactoring: 7 duplicate make_world() test helpers → shared test_helpers module, ID newtypes sealed (inner Spur private), registry fields encapsulated, eval helpers narrowed to pub(crate), hardcoded content ID audit (one runtime fix, rest documented).

**Branch:** `master`
**Verification (2026-03-12):** `cargo test -p undone-scene -p undone-ui -- --nocapture`, `cargo test --test prose_audit -- --nocapture`, `cargo test --test validate_pack_simulation -- --nocapture`, and `cargo run --bin validate-pack` all passed on `master`. Fresh runtime launch against `target\debug\undone.exe --dev --quick` also passed and rendered the live workplace opening. `validate-pack` still reports only non-blocking pre-existing warnings in `campus_library`, `neighborhood_bar`, `shopping_mall`, and `work_marcus_drinks`, plus reachability warnings for exact-liking Marcus/Jake gates.
**Scenes:** 62 total (47 Robin + 7 campus + 1 campus_theo_night + 7 new scenes [marcus_apartment, gym_changing_room, jake_stays_over, campus_theo_morning, marcus_monday_rejected, bad_date, campus_dining_after_theo], all in DM voice, 2026-03-22).
**Content focus:** CisMale→Woman only. AlwaysFemale, TransWoman, CisFemale all deprioritized.
**Sprint 1 complete + reviewed:** "The Engine Works" — 208→219 tests. All engine bugs fixed, all arc scenes reachable.
**Sprint 2 complete:** "FEMININITY Moves" — FEMININITY increments in all 7 workplace scenes (+20 total). plan_your_day rewritten. 219→220 tests.
**Sprint 3 complete:** "Robin's Playable Loop" — 14 new scenes, full writing audit pass. 220→223 tests.
**TRANS_WOMAN branches removed** from all scene files. CisMale-only pattern: `{% if not w.alwaysFemale() %}` with no `{% else %}`.
**All prose second-person present tense.** Zero third-person PC narration. Verified by comprehensive grep + automated test.
**NPC role binding via schedule.** Jake/Marcus scenes bind correct NPC via `npc_role` field in schedule events.
**Writing toolchain:** writing-guide updated with positive "scene must earn its place" rule. scene-writer + writing-reviewer custom agents current.
**Writing-agent research documented:** repo-local audit plus external lorebook/caching research captured for a future tooling pass; no behavior changes landed yet.
**DeepSeek writing infrastructure built:** `docs/writer-core.md` (compact prompt prefix, ~12KB), `docs/review-core.md` (review criteria, ~4KB), `tools/pack-prompt.mjs` (prompt assembly from scene specs). Scene-writer and writing-reviewer agents thinned to orchestrator shape. Full pipeline tested: 81% cache hit rate on second call. Writing guide m/f availability statement corrected.
**Authoring validation policy tightened:** `validate-pack` now warns on scenes with no persistent world mutation, not just missing game flags / NPC liking / arc advance. Scene-local flags and pure navigation no longer trigger false positives.
**Hardcoded content-ID audit reduced further:** character-creation presets now carry explicit starting flags instead of a misleading `arc_flag`, startup/`validate-pack` verify those flags are referenced by the scheduler, and structural smooth-legs / rough-content trait lookups are centralized through `PackRegistry`.

---

## ⚡ Next Action

**62 scenes. 498 tests passing. Sidebar shows story names. Save format v6.**

Pick one of these — each is independently scoped:

### A. Content expansion (writer-led)
- **Post-Theo campus life** — study sessions with new subtext, phone call
  home after sleeping with a man (HOMOPHOBIC register).
- **Marcus continued arc** — after `MARCUS_APARTMENT`, what's the relationship?
  Regular thing? Office complication? After `MARCUS_TALKED`, does respect evolve?
- **Jake relationship deepening** — a fight, a disagreement, the first time
  something goes wrong.
- **`plan_your_day` consequences** — choices still have weak consequences.
- **Stranger re-encounters** — the bar stranger again, the party stranger again.
- ~~**"Scale" cross-scene repetition**~~ — Resolved 2026-05-17. `gym_changing_room` keeps scale; `marcus_apartment` re-anchored on visibility/lit-room; `jake_stays_over` re-anchored on territorial integration. Commit `e8eef70`.

Use `scene-writer` agent for drafts, `writing-reviewer` for the pass.

**Two pre-existing prose issues in `marcus_apartment` for the next content pass to clean up** (out of scope for the scale surgery — neither was introduced by it):
- Narrator body-analysis "the body has already filed all of this before anything rational has caught up" (`cut_to_it`, FEMININITY<25).
- Emotion-announcement "warm and low has settled in your stomach" (intro, FEMININITY<25).

### B. Tech debt — small engineering
- ~~**Cache `FEMININITY` SkillId in `GameState`** (audit I11)~~ — Already done.
  `GameState.femininity_id: SkillId` is resolved once at construction; every hot
  path (`from_player`, `process_events`, snapshot refresh) reads the cached field.
  No per-snapshot string lookup exists. (Entry was stale.)
- ~~**UI test coverage for `process_events`** (audit I13)~~ — Done 2026-05-29
  (commit `b2dacca`). 9 tests cover every `EngineEvent` arm, the `scene_finished`
  return, the NPC known-context merge guard (both directions, mutation-verified),
  and the end-of-burst player-snapshot refresh. Paragraph cap was already covered
  (`append_story_paragraph_*`); `AppPhase`/scene-epoch invariants are covered by the
  `reset_scene_ui_state_*` tests. The deferred scroll-to-bottom bump (`exec_after`)
  is inherently runtime-bound, not unit-observable — left to runtime/playtester.
- **`work_marcus_drinks` line 55 "check your phone"** — last remaining
  validate-pack `filler_action` warning. One-line rewrite.

### C. Tech debt — bigger refactors (not session-end work)
- ~~**`char_creation.rs` is 2636 LOC.**~~ — Done 2026-05-30 (commit `5266ba8`).
  Split into 6 submodules (contracts/config/signals/sections/buttons/widgets);
  parent now 925 LOC (views + tests). Behavior-preserving, all gates passed.
- ~~**`engine.rs` is ~1860 LOC.**~~ Test module extracted 2026-05-31;
  parent is now 768 LOC. Any further production split should be a deliberate
  design pass, not a session-end cleanup.
- **20 desloppify subjective dimensions** are still unscored.

### D. Open verifications
- **35% width transient layout regression** — flagged in a prior session
  with no repro details. Needs a playtester run to reproduce before any code
  touches it.
- **Marcus content live-test** — `marcus_apartment`, `marcus_monday_rejected`
  flow has never been playtested end-to-end.

### Resolved this session
- ✅ ROLE_THEO routing through scheduler (commit `cffd7e0`)
- ✅ Sidebar shows story names — `SetNpcName` effect (commit `71bfc49` + tests `cf2b626`)
- ✅ ThoughtAdded/ErrorOccurred new-scene-top invariant (commit `f97375b`)
- ✅ validate-pack lint noise from `_archive/` (commit `3555908`)
- ✅ Rust-mcp dead stubs + UI font constant + test-helper dedup (commit `db0921a`)
- ✅ Redundant `set_game_flag ROUTE_*` in arrival scenes (audit I18, in `cffd7e0`)

### Completed this session (consequences sprint):
- 7 new scenes written in 2 batches:
  - Batch 1: marcus_apartment, campus_theo_morning, gym_changing_room, jake_stays_over
  - Batch 2: marcus_monday_rejected, bad_date, campus_dining_after_theo
- morning_routine enhanced with post-sexual intro texture
- evening_home enhanced with 2 new intro_variants + 2 new actions (text_jake, take_a_bath)
- 7 schedule entries added (fixed duplicate campus_theo_morning entry)
- Writing-reviewed batch 1, fixed Important findings
- 62 scenes total, validate-pack clean

### Resolved this session (conductor batch):
- ~~Add test for SetAllNpcLiking~~ — Done (dedicated test in dev_ipc.rs)
- ~~Add MCP wrapper for set_all_npc_liking~~ — Done (game-input-mcp)
- ~~FemCreation hardcodes "Eva"/"Ev"~~ — Fixed (preset names flow through)
- ~~Before-phase shows post-transformation attributes~~ — Fixed (filtered to personality only)
- ~~Attribute formatting raw comma list~~ — Fixed (chip/tag layout with categories)
- ~~Test fixture DRY~~ — 7 duplicates → shared make_test_world()
- ~~Encapsulation leaks~~ — ID newtypes sealed, registry fields private, eval helpers pub(crate)
- ~~Hardcoded content IDs~~ — Audited, one runtime fix, rest documented

### Previous: Prolific writing session + writing-review audit (2026-03-08)

### Completed this session (writing session continuation):
- **16 new scenes written** across 4 tracks (Jake romance, Marcus tension, stranger encounters, content deepening)
- **3 explicit adult scenes** — jake_apartment (first time, tenderness), bar_stranger_night (stranger, loss of control), work_marcus_closet (workplace, transgression)
- **Jake arc complete** (7 scenes): coffee_shop → coffee_shop_return → jake_outside → jake_first_date → jake_second_date → jake_apartment → jake_morning_after + jake_text_messages (recurring)
- **Marcus arc complete** (8 scenes): workplace_work_meeting → work_marcus_coffee → work_marcus_favor → work_marcus_late → work_marcus_drinks → work_marcus_closet → work_marcus_aftermath
- **Stranger encounters** (3 scenes): bar_closing_time → bar_stranger_night, party_invitation
- **Universal/deepening** (6 scenes): weekend_morning, shopping_mall, landlord_repair, laundromat_night + 2 existing expanded (workplace_work_meeting +2 actions, workplace_evening +2 actions)
- **Character docs** created: `docs/characters/jake.md`, `docs/characters/marcus.md`
- **Full schedule integration** with flag progression chains validated

### Confidence ratings (user-requested):
| Scene | Track | Confidence | Notes |
|---|---|---|---|
| jake_first_date | Jake | HIGH | Natural progression from existing scenes |
| jake_second_date | Jake | HIGH | Market/peach scene, first kiss — vivid |
| jake_apartment | Jake | MEDIUM | Explicit, tone-critical — needs user review |
| jake_morning_after | Jake | HIGH | Aftermath is good writing territory |
| jake_text_messages | Jake | MEDIUM | Unusual format (text messages in prose) |
| work_marcus_late | Marcus | HIGH | Established workplace register |
| work_marcus_drinks | Marcus | HIGH | Clear escalation, good trait branching |
| work_marcus_closet | Marcus | MEDIUM | Workplace explicit — needs user review |
| work_marcus_aftermath | Marcus | HIGH | Monday-morning consequences |
| bar_closing_time | Stranger | HIGH | Setup scene, well-defined |
| bar_stranger_night | Stranger | MEDIUM | Must not be exploitative — needs user review |
| party_invitation | Stranger | HIGH | Good trait branching, 4 actions |
| weekend_morning | Deepening | HIGH | Private body register, distinct |
| shopping_mall | Deepening | HIGH | Fitting room mirrors, choosing vs survival |
| landlord_repair | Deepening | HIGH | Domestic power dynamic, not romantic |
| laundromat_night | Deepening | MEDIUM | Mundane tension — may need stronger hook |

**Scenes requiring user creative review:** jake_apartment, bar_stranger_night, work_marcus_closet (all explicit, all MEDIUM confidence)

### Writing-reviewer audit (2026-03-08, continuation session):
- **18 scenes audited** by 4 parallel writing-reviewer agents (stranger+universal, marcus, jake, expanded)
- **~65 Critical/Important findings** across all tracks — all fixed
- **Recurring patterns killed:** narrator body-analysis, omniscient narrator, "specific" overuse, transformation-as-yardstick, full articulated thoughts, vague abstractions
- **One copy-paste duplicate paragraph** found and fixed in workplace_evening.toml
- **Estimated post-fix grades:** Stranger+Universal A-/A, Marcus B+/A-, Jake A-/A, Expanded B+
- **work_marcus_closet** was lowest-rated (C+ pre-fix → B+ post-fix) — most narrator body-analysis violations

### Remaining priorities:
1. ~~**Zero adult content**~~ — RESOLVED. 3 explicit scenes written (jake_apartment, bar_stranger_night, work_marcus_closet). Game can now prove its premise.
2. ~~**Button overflow bug**~~ — FIXED. Reactive `max_height` workaround.
3. **FemCreation "Who Are You Now" too brief** — needs 4-5 interactive discovery beats (creative direction required)
4. **7 campus arc scenes uncalibrated** — deprioritized (Camila route, not default Robin route)
5. ~~**Traits display overflow**~~ — FIXED
6. ~~**Single-action scenes**~~ — FIXED. workplace_work_meeting now has 3 actions, workplace_evening has 3 actions.
7. ~~**Content volume**~~ — SIGNIFICANTLY IMPROVED. 16 new scenes. Free_time has 20 events (was 12). Work has 12 events (was 8). Post-arc rotation is now rich.

### Remaining open items (post-Sprint 3)
- **Writing-agent/tooling cleanup** — ✅ RESOLVED for concrete queued items. Contract mismatch fixed, agents thinned, prompt packer built, cache instrumentation and retry/backoff added, and repo-neutral dispatch documented in `docs/writing-delegation.md`. Optional future polish only: canonical prompt wrappers / local dedupe.
- **Post-arc content void** — Sprint 3 expanded free_time from 3→8 scenes and added 7 work slot scenes (settled state). Remaining gap: campus arc has no post-arc slot equivalent. → Sprint 4+.
- **Prose polish pass** — Workplace arc prose is mostly clean after Sprint 2–3 audit passes. Campus arc has ~20 open Critical/Important writing findings from the 2026-02-25 audit. → Sprint 4+.
- **Free_time expansion** — more universal scenes needed. → Sprint 4.
- **NPC character docs missing** — 13 named NPCs have no character docs (Marcus, David, Jake, Frank, etc.). → As needed.
- **Presets as pack data** — built-in presets still live as static Rust structs in `crates/undone-ui/src/char_creation.rs`, even though their starting-flag contract is now explicit and validated. → Sprint 5.
- ~~**Test fixture DRY**~~ — RESOLVED. Shared `make_test_world()` in undone-world::test_helpers.
- ~~**Hardcoded content IDs in engine code**~~ — RESOLVED. Audited all engine crates. One runtime fix (eval.rs FEMININITY error path). Remaining are test fixtures and closed enum mappings, documented with audit comments. Presets in char_creation.rs still embed base-pack strings but are UI/content, not engine.

### User playtest feedback (2026-02-27, second session)

**UX / Navigation:**
1. ✅ **Settings inaccessible from "Your Story Begins"** — Fixed: Settings tab now works from any phase (char creation, transformation intro, fem creation, in-game).
2. ✅ **No landing page / load game screen** — Fixed: startup now opens landing page with New Game / Continue / Load / Settings and supports loading saves before character creation.
3. ✅ **Deferred Settings teleport** — Fixed: tab state resets to Game on InGame transition. Settings accessible from all phases eliminates the deferred-click problem.
4. ✅ **Default text size too small** — Fixed: default bumped 17→19. (User's existing prefs file still shows 17 — only affects fresh installs.)

**Char creation:**
5. **"Who Are You Now" screen extremely lacking** — Known issue (attribute dropdowns not implemented). But user is hitting it now as a real blocker to the experience feeling complete.
6. **FemCreation ignores preset names** — When using Robin preset, "Who Are You Now?" shows "Eva"/"Ev" (custom form defaults) instead of "Robin"/"Robin". `FemFormSignals::new()` hardcodes Eva/Ev; it should carry forward from the preset's `name_fem`/`name_androg` fields. Fix: pass preset names into `fem_creation_view` or store them on `PartialCharState`.
7. *(Previous)* Trait list runoff, post-transformation attributes in before-phase, attribute formatting — still open.

**Opening scene:**
8. ✅ **Wrong opening scene** — Fixed: `transformation_intro` is now the plane boarding + in-flight sleep scene. Transformation occurs in the gap before FemCreation.

**Story panel layout:**
9. ✅ **Flavor text box too small** — Fixed: detail strip enlarged (min_height 28→40, padding 6→10, font 13→14).
10. ✅ **Action buttons misaligned with prose** — Fixed: buttons now have horizontal padding matching prose column.

**Sidebar:**
11. ✅ **NPC display — wrong character showing** — Fixed with Sidebar Phase 1 guardrails: known-NPC-only visibility and stable active-NPC handling.
12. ✅ **NPC formatting broken** — Fixed with new `People Here` card layout and qualitative bands.
13. ⚠️ **Multiple NPC display unclear** — Phase 2 pending (multi-NPC chips/selection).

**Writing quality:**
14. ✅ **Rain scene writing bad** — Fixed: register calibrated (2026-03-08). DM narrator, no thought-insertion.
15. ✅ **Repetitive transformation narration** — Fixed: "you used to" pattern banned. Transformation shown through physical facts only.
16. ✅ **DeepSeek API for writing agents** — Infrastructure built (2026-03-07). Full pipeline tested.
17. ✅ **Remaining scenes register rewrite** — All 26 non-campus scenes calibrated. !w.alwaysFemale() → FEMININITY thresholds, "you used to" eradicated, thoughts → fragments. Campus arc (7) deprioritized.

### Char creation bugs (previous session)
- ~~**Trait list runoff**~~ — FIXED: flex_basis(0) + flex_grow(1) + max_width(400) on read_only_row value label. Visually confirmed.
- **Post-transformation attributes in before-phase** — preset detail shows post-transformation physical traits (Straight hair, Sweet voice, Almond eyes, Wide hips, Narrow waist, Small hands) in the "Who Were You?" phase. Before-phase should only show before-life attributes (name, age, race).
- **Attribute formatting** — trait/attribute display is a raw comma list with no structure. Needs proper layout (grouped, wrapped, styled).
- **Needs test coverage** — these visual/layout bugs should be caught by integration tests or a structured playtest checklist, not manual user testing.

### Design principles to document
- **Conditional actions (BG3-style)** — actions can show/hide or be grayed out based on player traits, skills, stats, personality. Already supported via `condition` fields in TOML. Needs design documentation with examples: stat checks, personality-locked dialogue, skill-gated options. BG3 is the reference.
- **Player agency** — actions are the player's choices. Intro/NPC prose describes the world acting on her; actions are her responses. Already added to writing-guide.md + agent docs (2026-02-27 session).
- **Testing philosophy** — anything that can be considered broken should be covered by tests. Not just unit tests — integration tests, layout validation, playtest checklists. Current gap: UI layout issues only caught by manual playtesting.

### Code audit findings (2026-02-27)
- ✅ **9 `eprintln!` → `log` crate** — all replaced with `log::warn!`/`log::error!`. Engineering Principle 9 added.
- ✅ **Dead `SceneId` struct** — removed from `ids.rs`
- **~25 hardcoded content IDs** across 6 engine files — already in open items, audit confirmed scope
- **1 `expect` risk** in `new_game()` FEMININITY resolve — panics if pack broken
- **3 `pub fn` eval helpers should be private**
- **`pub` field leaks** — registry internals (trait_defs, npc_trait_defs, skill_defs), lasso Spur on ID newtypes, TOML deser types
- **Scene agency violations** — 5 scenes have PC speaking/deciding in intro/NPC prose (campus_call_home, campus_arrival, campus_orientation, rain_shelter small_talk, workplace_first_day dan_explains). 27/33 scenes clean.

### Writing sessions — no compilation needed
For pure writing (authoring `.toml` scene files), no Rust compilation is needed. Scenes load at runtime. The workflow is:
1. Write `.toml` files in `packs/base/scenes/`
2. Run `cargo run` (or use the game-input MCP) — if no Rust code changed, only linking (~5s)
3. Validate templates with `mcp__minijinja__jinja_validate_template` before running

---

## game-input-mcp

**Input tools:** `press_key(title, key)`, `click(title, x, y)`, `scroll(title, x, y, delta)`, `hover(title, x, y)`.
Keys: `"1"`–`"9"`, `"enter"`, `"tab"`, `"escape"`, `"space"`.
Scroll: sends WM_MOUSEMOVE before WM_MOUSEWHEEL (floem routes wheel events via cached cursor_position). Positive delta = up, negative = down (one tick = one wheel notch).
**Lifecycle tools:** `start_game(working_dir, dev_mode)`, `stop_game(exe_name)`, `is_game_running(exe_name)`.
**Dev IPC tools:** `dev_command(command_json, timeout_ms)`, `get_game_state()`, `get_runtime_state()`, `jump_to_scene(scene_id)`, `choose_action(action_id)`, `continue_scene()`, `set_tab(tab)`, `set_window_size(width, height)`, `set_game_stat(stat, value)`, `set_game_flag(flag)`, `remove_game_flag(flag)`.
Successful runtime commands now return the same structured runtime snapshot used by the dev panel inspector and acceptance tests, including live `window_width` and `window_height`.
Process management uses Toolhelp32 snapshot API.

---

## screenshot-mcp — Persistent Sessions

Rewrote from one-shot WGC capture to persistent capture sessions (10fps). First request creates session + waits up to 1s for initial frame. Subsequent requests read cached frame (~20ms). Sessions are keyed by window title, evicted when window closes. Fast PNG encoding via `png` crate with `Compression::Fast`.

---

## UI — Current State

**Layout:**
- Title bar always visible: UNDONE branding, Game/Saves/Settings tabs, window controls
- Stats sidebar on the **left** (280px fixed): player name, stats, `People Here` (known-NPC only), mode toggle
- Story + choices on the **right** (flex-grow): scrollable prose + choices bar
- Window opens at 1200×800, titled "Undone"

**Scroll (floem):**
- All scroll containers use `.scroll_style(|s| s.shrink_to_fit())` — required for floem scroll in flex layouts
- Story panel scroll: `flex_grow(1.0).flex_basis(0.0)` (flex sibling of detail strip + choices bar)
- Char creation scroll: `size_full()` (sole child of dyn_container)
- Outer dyn_container: `flex_grow(1.0).flex_basis(0.0).min_height(0.0)` — required so taffy constrains children
- **Scroll-to-bottom timing:** floem's `scroll_to` uses `update_state_deferred` which processes after layout in the same frame. When content change and scroll signal fire simultaneously, `child_size` may be stale. Fix: defer `scroll_gen` bump via `exec_after(Duration::ZERO)` to next frame. Only scroll on append (not on first prose of a new scene).

**Theme system:**
- Three modes: Warm Paper (default), Sepia, Night
- Mode toggle at bottom of stats sidebar
- All colors driven by `ThemeColors::from_mode()` reactively

**Keyboard navigation:**
- Arrow Up/Down highlight choices, Enter confirms highlighted choice, Escape clears highlight
- Number keys 1–9: configurable via NumberKeyMode (Instant = fire immediately, Confirm = highlight then Enter)
- Detail strip shows highlighted choice detail (falls back to hovered)

**Key source files:**
- `crates/undone-ui/src/lib.rs` — AppSignals, AppTab, AppPhase, app_view
- `crates/undone-ui/src/char_creation.rs` — character creation form (pre-game phase)
- `crates/undone-ui/src/saves_panel.rs` — save/load/delete UI
- `crates/undone-ui/src/title_bar.rs` — custom title bar, tab nav, window controls
- `crates/undone-ui/src/story_panel.rs` — story panel, centered prose, detail strip, choices bar
- `crates/undone-ui/src/right_panel.rs` — stats sidebar, People Here module, mode toggle
- `crates/undone-ui/src/settings_panel.rs` — settings tab (theme, font size, line height, number key mode)
- `crates/undone-ui/src/theme.rs` — ThemeColors, ThemeMode, NumberKeyMode, UserPrefs, save/load prefs
- `crates/undone-ui/src/game_state.rs` — PreGameState, GameState, init_game(), start_game()
- `.interface-design/system.md` — full design system spec

---

## Planned Future Sessions

1. ~~Scene engine~~ ✅
2. ~~Scheduler~~ ✅
3. ~~Save / load~~ ✅
4. ~~Design research~~ ✅
5. ~~UI quality pass~~ ✅
6. ~~NPC spawning + character creation~~ ✅
7. ~~UI polish~~ ✅
8. ~~Writing guide~~ ✅
9. ~~Engineering hardening~~ ✅
10. ~~Character creation UI~~ ✅
11. ~~Writing import~~ ✅ (3 scenes with original prose)
12. ~~Names update~~ ✅
13. ~~Saves tab~~ ✅
14. ~~**Character creation redesign**~~ ✅ (PcOrigin system: two-step flow, 4 origin types, trans woman PC type)
15. ~~**Keyboard controls redesign**~~ ✅ (arrow nav, Confirm mode, Escape, highlight style)
16. ~~**Settings tab UI**~~ ✅ (theme, font size, line height, number key mode controls)
17. ~~**Engine foundation**~~ ✅ (identity, time, activity loop, effects, evaluator — 9 tasks, 29 new tests)
18. ~~**Validate and audit engine foundation**~~ ✅ (15 findings categorized, 10 fixed, 5 deferred/resolved, 16 new tests)
19. **More scenes** — expand base pack content

---

## Open Items — Future Sessions

### UI Polish (Small-Medium)
- **Detail strip hover highlight**: Brief unwanted background highlight in Warm theme on first hover (floem default style leak — partially fixed with explicit hover/focus overrides)
- **Choice button positioning**: Consider better visual balance between prose area and choices
- **Save metadata display**: Show player name / week in save list without full deserialization

### Tooling
- **game-input limitation**: PostMessage-based input may not establish focus like real user input — keyboard shortcuts may not fire after PostMessage click

---

## Session Log

| Date | Summary |
|---|---|
| 2026-06-02 | MCP tools fixes (`9782763`). Rebuilding the MCP binaries surfaced two latent defects (both root-cause fixed, no workarounds). (1) The tools build had been BROKEN since the registry merge — making the servers depend on `undone-scene` meant `rhai-mcp-server`'s `rhai = ["sync"]` unified `rhai/sync` onto `undone-scene`, whose thread-local raw-ptr eval closures are deliberately non-Send/Sync → 13 compile errors in `rhai_bind.rs`. Fixed by dropping `sync` and building the validation Engine per-call (handler stays Send+Sync; `check_syntax` already did this). (2) rust-analyzer orphaning (the multi-GB pileup): `rust-mcp` spawned RA with no `kill_on_drop` and `processId: null`; added `.kill_on_drop(true)` + own-PID `processId` (LSP-spec self-exit on hard kill) — so subagents that start rust-mcp no longer leak, making the proposed agent-`mcpServers`-allowlist unnecessary. Verified: `cd tools && cargo check --release` clean; `rhai-mcp-server` 10/10 tests pass. Cleaned ~1.2 GB of live orphans (only the active session's RA subtree remains). NOT done: producing + swapping the actual `.exe`s — the running servers lock them, so restart the session then run `pwsh tools/rebuild-mcp.ps1`. |
| 2026-06-02 | Merge-confirm + push + handoff refresh. Confirmed `script-api-registry` was already merged to master (`e046ffe`) — branch gone, `REGISTRY` live in `crates/undone-scene/src/script/api/table.rs` (~95 reads + ~37 writes). Pushed master to `origin/master` (was 134 commits ahead; `f35e4b2..0a20a15`) — registry + all content/tooling/Marcus-terms-fork-plan work now on remote. Refreshed HANDOFF: prepended current-state entry, marked the stale "NOT yet merged" / "Merge → master" notes superseded, recorded the content-first strategic posture (scripting is the content layer; only new mechanics/verbs need Rust). No code changes. Still open: Marcus terms-fork implementation (`dcea56a`/`0a20a15` plan docs), MCP-binary rebuild on this machine, subagent MCP-orphaning infra decision. |
| 2026-06-02 | Story-map authoring tool shipped (plan `docs/plans/2026-06-02-story-map-tool.md`, branch `story-map`). New `src/story_map.rs` + `src/bin/story_map.rs` derive the scene-connectivity graph from base-pack data, reconcile against authored `packs/base/roadmap.toml` (7 threads, all 74 scenes claimed, 0 orphans), and emit `docs/story-map.{md,json}` with a ranked `write_next` digest (dangling > broken > planned). Flags and `ARC=STATE` treated as uniform signals: produced (effects) vs required (gates); dangling = produced-never-required, broken = required-never-produced. Engine touch-ups: `reachability::{required_game_flags, arc_state_eqs}` pub wrappers + `Scheduler::bindings()`/`SceneBinding` projection. Added `.gitattributes` forcing LF on the generated docs so `--check` is stable under `core.autocrlf=true`. Two plan-test bugs fixed during TDD (the plan's tests used `gd.changeStress`→`w.changeStress` and an unregistered `base::arc` that `compile_effect` rejects — test helper now registers it). Verification: 8 story_map unit tests + 5 acceptance tests green; `cargo run --bin story-map -- --check` clean; `scene-writer` agent + `docs/content-schema.md` wired to the tool. |
| 2026-05-17 | Prose surgery — scale-anchor repetition. `gym_changing_room`, `marcus_apartment`, `jake_stays_over` all shared a "scale of him vs you" body-observation anchor in their FEMININITY-gated branches. `gym_changing_room` owns it structurally (women's-only locker room); `marcus_apartment` re-anchored on visibility/deliberateness/lit-room (3 FEMININITY<25 branches: intro, stay_for_the_wine bedroom, cut_to_it bedroom); `jake_stays_over` re-anchored on territorial integration (2 FEMININITY<30 branches: intro, make_coffee closer). writing-reviewer audit caught one rhythm-echo I introduced + four minor closing-sentence issues, three fixed in-pass and the echo fixed post-playtester. Playtester launched the release binary in --dev --quick with Robin preset (FEMININITY 10), used jump_to_scene + choose_action to play each affected branch, took screenshots of rendered prose, and reported all four passages render correctly with no template breakage or UI jank. Two pre-existing prose issues in marcus_apartment (narrator body-analysis on "the body has already filed all of this", emotion-announcement on "warm and low has settled in your stomach") surfaced during audit but were not introduced by this surgery — logged in Next Action for a future content pass. Also committed a stray `crates/undone-scene/Cargo.toml` dev-deps pin left uncommitted by the prior test-author session — `set_npc_name_tests` needs `undone-save` + `serde_json` as dev-deps to compile. cargo test --workspace = 498 passing; validate-pack clean. Commits `49de362` (dev-deps pin) and `e8eef70` (prose surgery). |
| 2026-05-17 | Cleanup + feedback triage + display-name feature. Audited every open audit/HANDOFF feedback item against actual code — most engineering Criticals/Importants resolved silently in prior sessions. 7 commits: (1) finalized the in-progress cleanup pass — pruned 9 dead rust-mcp stub tools (~1500 LOC), `UI_FONT_FAMILY` constant, deduped `make_test_male_npc`, dropped unused `anyhow`/`slotmap`. (2) Fixed `ROLE_THEO` scheduler routing (npc_role missing on Theo schedule entries) + dropped redundant `set_game_flag ROUTE_*` in arrival scenes (audit I18). (3) Fixed `ThoughtAdded`/`ErrorOccurred` scroll-to-bottom violating the new-scene-top invariant (audit I8). (4) `validate-pack` now skips underscore-prefixed scene subdirs. (5) **Feature**: `NpcCore.display_name` + `SetNpcName` effect — root-cause fix for sidebar showing random spawn names instead of story names. Decoupled from `set_npc_role`; save v5→v6 (no-op JSON, `#[serde(default)]`); wired into `coffee_shop`/`workplace_work_meeting`/`campus_library`. (6) Independent acceptance suite via `ops:test-author`: 27 tests, each with `// BREAKS IF` comment. (7) Playtester verified live on scheduler-driven `coffee_shop` at week 2 — sidebar reads "Jake". 470 → 498 tests passing; validate-pack clean. |
| 2026-03-12 | Engineering + creative cleanup. Implemented `docs/plans/2026-03-12-engineering-creative-cleanup.md`: shared UI runtime bootstrap, recoverable startup/content-contract errors, reusable `validate-pack` library API, prose-audit coverage, targeted campus-scene POV/register cleanup, and filler/fine-test cleanup for core scenes. Post-implementation audit found one live-flow bug (init failures stayed in landing flow); fixed by forcing visible error-phase startup when `pre.init_error` exists. Verification: `cargo fmt`, `cargo test -p undone-ui --lib`, `cargo test --test validate_pack_simulation -- --nocapture`, `cargo test --test prose_audit -- --nocapture`, `cargo run --bin validate-pack`, plus fresh `cargo run --bin undone -- --dev --quick` runtime launch. Non-blocking repo-wide prose warnings remain outside the touched slice. |
| 2026-03-09 | Playable-game fixes + audit. Implemented plan `docs/plans/2026-03-09-playable-game-fixes.md` (6 tasks). (1) Action prose invisible on scene transitions — added `awaiting_continue` signal + Continue button; player now sees action prose before next scene loads. (2) Scheduler burying NPC intros — converted `coffee_shop` and `neighborhood_bar` from random-weighted to deterministic triggers at weeks 2/3; `morning_routine` weight 15→10. (3) AROUSAL never moving — wired `add_arousal` effects into 8 charged/explicit scenes (jake_apartment, work_marcus_closet, bar_stranger_night, jake_second_date, work_marcus_drinks, bar_closing_time, jake_first_date, weekend_morning). (4) FemCreation zero prose — added brief framing paragraph. (5) Plane scene thin — elaborated with career identity beats, gate normality, and "last version of you" close. (6) Runtime playtested all changes. Code-reviewed, acceptance-tested. Post-merge audit: fixed Continue-into-dead-end (scheduler exhaustion shows message instead of stale UI) and key consumption during awaiting state. 289 tests, 0 failures. |
| 2026-03-08 cont.6 | Opus audit of Sonnet's dev tooling work. Deep audit via 5 parallel agents (IPC, reachability/simulator, BoundedStat, MCP server, main/UI integration). All code production-quality. Four fixes: (1) simulator picks per time slot not per week — `weekend_morning` now correctly appears, avg/run 28× more accurate; (2) dev panel `DevContext` struct eliminates 7-param pollution across 10+ call sites (net -70 lines); (3) "All NPC → Close" button routed through new `SetAllNpcLiking` IPC command instead of direct mutation; (4) engine design doc lists all 8 IPC commands (was 5). 287 tests, 0 failures, 0 warnings. |
| 2026-03-08 cont.5 | Dev tooling cleanup. Merged `codex/dev-tooling-plan` to master after cleanup pass: removed `dyn_view` from title bar, fixed IPC polling 50ms→100ms, atomic tmp+rename for command+result files, added AdvanceTime + SetNpcLiking IPC commands + matching MCP tools, quick-action buttons in dev panel (Advance 1 Week, All NPC→Close). Audit found + fixed: negated hasGameFlag reachability false-positives (suppress warning when inside Not), tmp file leak on rename failure. 287 tests, 0 failures. |
| 2026-03-08 cont.4 | Writing-reviewer audit + fix pass. 4 parallel writing-reviewer agents audited all 18 new/expanded scenes. ~65 Critical/Important findings fixed across all tracks: narrator body-analysis, omniscient narrator, "specific" overuse, transformation-as-yardstick, full articulated thoughts, vague abstractions. Fixed copy-paste duplicate paragraph in workplace_evening. Fixed 3rd-person POV slips. Post-fix grades: Stranger+Universal A-/A, Marcus B+/A-, Jake A-/A, Expanded B+. 49 scenes pass validate-pack. |
| 2026-03-08 cont.3 | Prolific writing session. 16 new scenes across 4 tracks: Jake romance arc (jake_first_date, jake_second_date, jake_apartment [explicit], jake_morning_after, jake_text_messages), Marcus tension arc (work_marcus_late, work_marcus_drinks, work_marcus_closet [explicit], work_marcus_aftermath), stranger encounters (bar_closing_time, bar_stranger_night [explicit], party_invitation), content deepening (weekend_morning, shopping_mall, landlord_repair, laundromat_night). Expanded 2 existing single-action scenes (workplace_work_meeting +2 actions, workplace_evening +2 actions). Created character docs (docs/characters/jake.md, marcus.md). Full schedule.toml integration with flag progression chains: MET_JAKE→JAKE_FIRST_DATE→JAKE_SECOND_DATE→JAKE_INTIMATE→JAKE_MORNING_AFTER, FIRST_MEETING_DONE→MARCUS_LATE_NIGHT→MARCUS_DRINKS→MARCUS_INTIMATE→MARCUS_AFTERMATH, BAR_STRANGER_INVITED→BAR_STRANGER_SLEPT. 3 scene-writer agents dispatched per batch (parallel). 49 scenes total. 262 tests, 0 failures. validate-pack clean. |
| 2026-03-08 cont.2 | UI bug fixes + playtester agent rewrite. Fixed traits text overflow in char creation (flex_basis(0) + flex_grow(1) + max_width(400) on read_only_row label). Rewrote `.claude/agents/playtester.md` per user direction — explicit horny player perspective, not polite QA. Attempted button overflow bug (3rd choice clips below window when buttons wrap to 2 rows) — 5 approaches failed: min_height(0) on scroll/parents, flex_grow+flex_basis on ancestors, height_full on story v_stack, moving buttons inside scroll (rejected: user says buttons shouldn't require scrolling), wrapping scroll in extra container. Root cause: floem scroll widget doesn't properly shrink in flex column. All experimental changes reverted. User directed: research floem docs/source next session, fix the bug, write findings to floem-layout skill. |
| 2026-03-08 cont. | Scene calibration completion pass. Calibrated remaining 16 non-campus scenes (morning_routine, coffee_shop, bookstore, park_walk, grocery_store, evening_home, work_standup, work_lunch, work_corridor, work_late, work_friday, work_marcus_coffee, work_marcus_favor, jake_outside, coffee_shop_return, plan_your_day). All `!w.alwaysFemale()` guards replaced with `w.getSkill('FEMININITY') < N` thresholds. All ~25 "you used to" / "from the other side" banned patterns replaced with involuntary physical reactions and body-first observations. All `{% else %}` AlwaysFemale branches removed. All inner voice thoughts converted to fragments. 26/33 scenes now calibrated (7 campus arc deprioritized). 262 tests, 0 failures. validate-pack passes. |
| 2026-03-08 | Writing register calibration + Robin opening scenes. Calibrated the prose register through 7 iterative attempts with user feedback — landed on DM narrator style (casual, specific, on the player's shoulder). Updated all 7 writing docs (writing-guide, creative-direction, writer-core, review-core, writing-samples, scene-writer agent, writing-reviewer agent) to enforce the register. Wrote calibration design doc + session prompt. Rewrote 4 scenes to calibrated register: `transformation_intro.toml` (plane, before-body accessors), `workplace_arrival.toml` (2-round: ID + transport), `workplace_first_day.toml` (2-round: Dan + lunch, 5 actions), `neighborhood_bar.toml` (3-round: order → nurse/NPC → accept/decline, matches Sample 0). All 33 scenes pass validate-pack. All Jinja templates valid. Writing-reviewer audit run on key scenes. |
| 2026-03-07 | Hardcoded content-ID audit session. Reconciled current code against docs first, then tightened the character-creation runtime contract instead of leaving preset route flags and rough-content traits as ambient UI strings. `PartialCharState.arc_flag` became explicit `starting_flags`; startup and `validate-pack` now validate that built-in preset starting flags are actually referenced by the scheduler; rough-content preference traits (`BLOCK_ROUGH` / `LIKES_ROUGH`) and structural smooth-legs trait lookups are centralized through `PackRegistry`; runtime/template code now reuses those helpers instead of scattering structural trait strings. Added targeted tests in `undone-scene`, `undone-packs`, and `undone-ui`. Final verification passed: `cargo fmt --all`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo run --bin validate-pack`, `node tools/deepseek-helper.mjs --help`. |
| 2026-03-07 | Authoring validation policy refinement session. Reconciled current code/docs first, then tightened `validate-pack`'s "no lasting effects" heuristic to match actual persistent world mutations instead of only `set_game_flag` / `add_npc_liking` / `advance_arc`. Added reusable `EffectDef::mutates_persistent_world()` and `SceneDefinition::has_persistent_world_mutation()` helpers with unit coverage, including NPC-action coverage. `validate-pack` now warns only when a scene truly lacks persistent world mutation; scene-local flags and pure navigation no longer create false positives. Synced `docs/engine-contract.md` and `docs/audits/2026-03-07-engine-readiness-matrix.md`. Final verification passed: `cargo fmt --all`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo run --bin validate-pack`, `node tools/deepseek-helper.mjs --help`. Current warning inventory: `base::campus_library` still warns because it only mutates scene-local state. |
| 2026-03-07 | DeepSeek writing infrastructure session. Built the full prompt-packing and reference doc pipeline: `docs/writer-core.md` (compact 12KB DeepSeek prompt prefix with voice, anti-patterns, trait tables, TOML format, adult content directive), `docs/review-core.md` (compact 4KB review criteria), `tools/pack-prompt.mjs` (assembles prompts from JSON scene specs with route-specific context). Fixed `docs/writing-guide.md` m/f availability statement (action/NPC-action only, not intro). Thinned `scene-writer.md` from 301→40 lines and `writing-reviewer.md` from 288→40 lines to orchestrator shape referencing the core docs. Tested full pipeline: rain_shelter reproduction (v1 had over-naming, short output; v2 after iteration had proper trait branches in actions, correct NPC branching within single action, 81% cache hit rate). New scene test (workplace_lunch_break) confirmed cache efficiency and structural quality. Annotated resolved findings in `docs/audits/2026-03-07-writing-agent-tooling-audit.md`. Updated `docs/deepseek-writing-tool.md` with prompt packer workflow. |
| 2026-03-07 | Documentation + research session. Audited the current writing-agent path (`scene-writer`, `writing-reviewer`, `CLAUDE.md`, `writing-guide`, `deepseek-helper`) and documented that it is usable but not yet minimal-friction or fully contract-clean. Wrote `docs/audits/2026-03-07-writing-agent-tooling-audit.md` covering the current state and the highest-signal local gap: `.claude/agents/scene-writer.md` teaches `m`/`f` prose-template access while `docs/writing-guide.md` and current engine contract do not guarantee that. Wrote `docs/research/2026-03-07-writing-context-lorebooks-and-caching.md` with public research on Anthropic subagents/memory/prompt caching, DeepSeek context caching and pricing, SillyTavern lorebooks/personas/Data Bank, and public Janitor-ecosystem script patterns. Added `docs/prompts/2026-03-07-engineering-fresh-session-prompt.md` for the next fresh engineering session. No code changes; no verification run needed. |
| 2026-03-07 | Ironclad engine hardening session. Save/resume: added authoritative UI-side resume helpers in `game_state.rs`, centralized in-place save reload through runtime reset, and added end-to-end coverage that saves a real workplace-route world, reloads it through UI helpers, proves no opening-scene replay, and proves scheduler resume follows persisted arc state (`workplace_landlord`) with no stale runtime leakage. Runtime diagnostics: scene condition failures now emit visible `ErrorOccurred` diagnostics while still gating false; template render failures now surface through the same diagnostic path; UI tests confirm `ErrorOccurred` reaches story output. Authoring validation: scene load now rejects duplicate scene IDs plus duplicate `actions[].id` / `npc_actions[].id`, and `validate-pack` now rejects cross-pack duplicate scene IDs instead of silently overwriting. NPC contract: added UI coverage proving fallback male binding is valid for post-start action effects, and documented the important limitation that fallback `m`/`f` binding is not available during intro rendering. Synced `docs/engine-contract.md` and `docs/audits/2026-03-07-engine-readiness-matrix.md`. Final verification passed: `cargo fmt --all`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo run --bin validate-pack`, `node tools/deepseek-helper.mjs --help`. |
| 2026-03-03 | Opening+sidebar UX session. Replaced `transformation_intro` with the creative-direction plane scene (board flight as before-self, route-aware setup, fall asleep in air), and wired throwaway intro world to carry preset route flags so workplace/campus intro branches render correctly. Implemented Sidebar Phase 1 in `right_panel`: `People Here` module, known-NPC-only visibility policy (no info leakage), qualitative liking/attraction bands, and active-NPC event handling guard to avoid unknown placeholders overwriting known context. Added tests for known-NPC gating and band mapping. Added UX spec for Phase 2+ at `docs/plans/2026-03-03-npc-sidebar-redesign-ux.md`. Verified playability with `cargo check --workspace`, `cargo test --workspace`, and `validate-pack`. 240 tests, 0 failures. |
| 2026-03-03 | Landing page + resume flow session. Added new startup Landing phase with New Game / Continue / Load / Settings, wired Continue/Load to validated save loading before character creation, and updated phase/tab behavior so Game/Saves are available in Landing and InGame while Settings remains global. Added `start_loaded_game()` / `load_game_state_from_save()` in UI game_state wiring. Also completed hardening/simplification batches: parser depth guard + EOF consume, condition method signature validation, structural ID enforcement, save-load runtime reset, race registry usage in spawner, saves scroll fix, FEMININITY caching, and docs/tooling alignment (`docs/creative-direction.md`, writing agents, guide updates). 236 tests, 0 failures. |
| 2026-02-27 | UI quick wins session. Fixed 7 of 16 user-reported issues: Settings accessible from any phase (was broken during char creation + caused teleport bug), NPC sidebar hidden (was showing unmet NPCs with raw data), detail strip enlarged, button alignment padded to match prose, default font size 17→19, NPC coworker name collision (Robin→Alex in work_standup). Found new bug: FemCreation form ignores preset names (shows Eva/Ev instead of Robin/Robin). Documented remaining open items. 224 tests, 0 failures. |
| 2026-02-27 | Systematic cleanup session. Code: replaced all 9 `eprintln!` calls in library crates with `log` crate (`log::warn!`/`log::error!`), added `log = "0.4"` as workspace dependency. Removed dead `SceneId` newtype from `ids.rs`. Docs: major `engine-design.md` refresh (Player struct, GameData, NpcCore, EffectDef 35 variants, pack manifest, UI layout, workspace structure with tools/). `content-schema.md` fully updated (35 effect types grouped, gd./m./f. expression methods completed). `writing-guide.md` gd. methods updated. `scene-writer.md` effect types list synced. All three audit files annotated with ✅ RESOLVED / ⚠️ PARTIAL markers (engineering: 13 annotated, arc flow: 5 annotated, writing: systemic + per-scene annotations with file renames). `CLAUDE.md`: scene count 19→33, added Engineering Principles 9 (log crate in library code) and 10 (docs track implementation). 224 tests, 0 failures. |
| 2026-02-27 | MCP playtest session. Full 9-step playtest via screenshot + game-input MCPs. Found and fixed 2 bugs: (1) NPC wiring — StartScene without SetActiveMale/SetActiveFemale caused silent NPC effect failures; added `start_scene()` helper in lib.rs, replaced 5 callsites. (2) `gd.week()` stuck at 0 — no scene fired advance_time, free_time scenes permanently unreachable; added `advance_time slots=28` to workplace_evening.toml. TimeSlot has 4 variants (not 3), so 7×4=28. New test `pick_next_free_time_appears_after_arc_settles_with_advance_time`. All 9 playtest steps verified: char creation, 7-scene arc, settled state, free_time rotation (7 scenes seen), Jake thread (coffee_shop→MET_JAKE→park_walk), Marcus thread, FEMININITY-gated variants, ANALYTICAL trait branches. 223→224 tests, 0 failures. |
| 2026-02-27 | Project audit + tooling session. Full status review: 223 tests confirmed, 33 scenes, validate-pack clean. Cross-referenced all 3 audit files against Sprint 1–3 fixes — built definitive resolved/unresolved table (28 fixed, 8 by policy, 7 partial, 41 open). Key finding: C1 (once_only) and C2 (choose_action) were ALREADY FIXED — audit files and content-schema.md were stale. Updated content-schema.md (removed "NOT YET IMPLEMENTED" once_only note), HANDOFF.md open items, MEMORY.md sprint status. Writing toolchain audit: scene-writer.md missing gd.npcLiking + m./f. receivers + "scene must earn its place" checklist items; writing-reviewer.md had 8 invented trait IDs (HIGH_VOICE, SENSITIVE_NIPPLES, etc.) + missing Critical patterns (desire/shame ordering, trait-gated best content); writing-guide.md condition tables incomplete. All three files fixed. Screenshot MCP zombie-session bug: WGC PersistentCapture stops delivering frames after window surface recreation (floem phase transition) but is_finished() returns false. Fixed capture.rs to detect and auto-recreate zombie sessions. Binary rebuilt. Pushed 11 commits to origin. MCP playtest started — game boots, char creation works, transformation_intro renders. Playtest paused for session restart (screenshot MCP needs respawn). |
| 2026-02-25 | Writing pipeline Batches 4–8 (worktree: writing-pipeline). Batch 4: PresetData expanded to ~40 fields, Robin fully configured (38 traits, all physical/sexual attributes), Appearance dropdown replaces BEAUTIFUL/PLAIN checkboxes. Batch 5: Content rename — scene files, arcs.toml, schedule.toml from character-specific to archetype-based (robin→workplace, camila→campus). Batch 6: Rust test fixtures and comments updated. Batch 7: 7 parallel scene-writer agents rewrote all workplace scenes to second-person, stripped AlwaysFemale else branches, morning_routine.toml fixed. 3 writing-reviewer audits (0 Criticals). Batch 8: docs/characters/ → docs/presets/, arc docs renamed, writing tools updated with new accessors+traits. 11 commits, 53 files changed (+2554/−1750). 208 tests, 0 failures. Merged to master, worktree removed. |
| 2026-02-25 | Writing pipeline Batches 0–3 (worktree: writing-pipeline). Batch 0: Appearance/NaturalPubicHair/BeforeVoice enums, Complexion::Glowing, Player.appearance + Player.natural_pubic_hair + BeforeIdentity.voice fields, all 12 construction sites updated, v4→v5 migration extended. Batch 1: 4 new traits (NATURALLY_SMOOTH/INTOXICATING_SCENT/HEAVY_SQUIRTER/REGULAR_PERIODS), PLAIN/BEAUTIFUL removed (Appearance enum replaces). Batch 2: 6 new accessors (getAppearance/getNaturalPubicHair/getName/beforeName/beforeVoice/hasSmoothLegs) in template_ctx.rs + eval.rs. Batch 3: 3 engine bugs fixed — once_only flag setting at both pick_next call sites, stale action condition re-check in choose_action, NPC action next branches (NpcActionDef/NpcAction.next + loader resolution + engine evaluation). Identified test fixture DRY issue (8+ identical make_world helpers). 204 tests, 0 failures. 4 commits. |
| 2026-02-25 | Char creation UI attributes plan written (`docs/plans/2026-02-25-char-creation-ui-attributes.md`). 6 tasks: PartialCharState/CharCreationConfig expansion, before-panel dropdowns (6 fields), fem-panel dropdowns (14 fields in 3 sections), test helpers, physical trait pickers (6 groups), sexual trait pickers (BLOCK_ROUGH gated). |
| 2026-02-25 | Character attribute schema (worktree: char-attributes). 15 new enums: Height, HairLength, SkinTone, Complexion, EyeColour, HairColour, NippleSensitivity, ClitSensitivity, PubicHairStyle, InnerLabiaSize, WetnessBaseline, ButtSize, WaistSize, LipShape, PenisSize. PlayerFigure expanded 3→7 (Petite/Slim/Athletic/Hourglass/Curvy/Thick/Plus), BreastSize 4→7 (Flat/Perky/Handful/Average/Full/Big/Huge). Player struct: 12 new fields + String→enum for eye_colour/hair_colour. BeforeIdentity: 5 new fields (height, hair_colour, eye_colour, skin_tone, penis_size). 126 traits across 13 groups (hair, voice, eyes, body_detail, skin, scent, sexual 25, sexual_preference 20, dark_content 11, lactation, fertility, menstruation, arousal_response). 48 skills (9+39 new). 48 stats (3+45 new). ~27 new PlayerCtx template methods + eval_call_string condition accessors. Save migration v4→v5 (field defaults, String→enum conversion, variant remaps). NPC spawner updated for new enum variants. Docs: writing-guide 10-tier FEMININITY, content-schema accessor table, scene-writer + writing-reviewer agent updates. 20 files changed, +2581/-102. 204 tests, 0 failures. |
| 2026-02-25 | Content focus narrowed + arc flow audit. User directed: CisMale→Woman only (AlwaysFemale/TransWoman/CisFemale all deprioritized). Removed TRANS_WOMAN branches from all 7 scene files. Rewrote `docs/writing-samples.md` (removed 3rd-person robin_arrival sample — root cause of POV violations). Updated writing-guide, scene-writer, writing-reviewer to CisMale-only focus. Created `docs/content-schema.md` (complete schema reference). Arc flow audit found 3 engine bugs: NPC `next` field missing (6 scenes affected), `once_only` inert, `add_npc_liking` fails silently. `robin_first_clothes` confirmed unreachable. 13 named NPCs have no character docs. Full report: `docs/audits/2026-02-25-arc-flow-audit.md`. |
| 2026-02-25 | Writing & game design audit: 4 parallel agents (Robin prose, Camila prose, universal prose, game design). Found 16 Critical, 34 Important, 21 Minor. Systemic: both arcs in third-person (guide says second-person), TRANS_WOMAN register missing from 13/19 scenes, post-arc content void (3-scene loop after arcs exhaust), 8 skills with zero scene usage. Key writing criticals: over-naming ("specific quality", "geometry of being a woman"), staccato closers ("the city goes on"), adjective-swap branches, alwaysFemale gating gaps, `plan_your_day` is a placeholder stub. Full report: `docs/audits/2026-02-25-writing-design-audit.md`. |
| 2026-02-25 | Engineering audit: 6 parallel review agents across all 7 crates + pack data. 204 tests passing, 0 clippy warnings. Found 8 Critical, 19 Important, 18 Minor. Key criticals: once_only flag never set, choose_action skips condition check, scheduler load failure silent, ArcDef.initial_state dead, spawner count bug, hardcoded content IDs in char_creation. Full report: `docs/audits/2026-02-25-engineering-audit.md`. |
| 2026-02-25 | Linux/Mac dev readiness: removed hardcoded Windows `target-dir` from `.cargo/config.toml` (use `CARGO_TARGET_DIR` env var instead), added `enabledMcpjsonServers` for cross-platform servers to committed `settings.json`, added Linux setup section to CLAUDE.md. Also cleaned stale test binary from deleted arc-system worktree (had embedded old CARGO_MANIFEST_DIR). 204 tests, 0 failures. |
| 2026-02-25 | Arc system implementation (worktree: arc-system). Scheduler: added pick_next() — evaluates triggers across ALL slots first (alphabetical), then weighted pick from all eligible events. Arc scenes now reachable via ROUTE_* flags. GameState: removed default_slot (vestigial). UI: PartialCharState.arc_flag flows from char creation → starting_flags. PresetData gets arc_flag field (each preset declares its own flag). Custom players start freeform (no arc picker). Build: .cargo/config.toml with shared target dir (no more cold worktree builds) + codegen-units=16. 204 tests, 0 failures. |
| 2026-02-25 | Reorientation + cleanup: merged prolific-session branch (6 commits), code review (15 findings, fixed C1+C2+I3+M5), code simplifier pass (4 cleanups), created scene-writer + writing-reviewer custom agents in .claude/agents/. 200 tests, 0 failures. Pushed to origin. |
| 2026-02-21 | Design session: decompiled Newlife, designed Undone engine, wrote scaffold plan |
| 2026-02-21 | Tooling session: built rhai-mcp-server + minijinja-mcp-server, wired MCP + hooks |
| 2026-02-22 | Scaffold session: Tasks 1–3 complete. MCP confirmed working. Added agentic workflow rules. |
| 2026-02-22 | Scaffold session: Tasks 4–13 complete. Parallel agents for Tasks 7–11. 30 tests pass. Scaffold done. |
| 2026-02-22 | Scene engine: brainstorm + design. Flat pool model, event queue API, full backend scope. |
| 2026-02-22 | Scene engine: 10-task implementation. Pack loader, eval wiring, effect system, minijinja templates, SceneEngine, rain shelter scene. 58 tests, 0 warnings. |
| 2026-02-22 | Scene engine: code audit + cleanup. Merged to master, worktree removed. |
| 2026-02-22 | Autonomous session: Scheduler + Save/Load. 70 tests, 0 warnings. Merged to master. |
| 2026-02-22 | Design research session: character creation, NPC spawning, UI patterns, personality arch. All open questions resolved. |
| 2026-02-22 | NPC spawning + char creation: 7-task plan via agent team. Sexuality/Personality enums, Player three-name system, NPC spawner with diversity guarantees, new_game() factory. 85 tests, 0 warnings. Merged to master. |
| 2026-02-22 | Code audit: reviewer + simplifier. Fixed diversity guarantee bug (.take() truncated required personalities for male_count < 3). Simplified pick_traits to use choose_multiple. |
| 2026-02-22 | Planning session: UI plan written. Setting pivot to fictional NE US city. Design system init (Evening Reader / three modes). system.md written. |
| 2026-02-22 | UI implementation: floem migration + layout (Gemini-authored, unreviewed). 87 tests pass. Warm Paper theme. Scene boots. Module split. |
| 2026-02-23 | UI Quality Pass: Added Dark/Sepia theme modes, keyboard navigation, fixed clippy warnings. |
| 2026-02-23 | UI Review + Fixes: £→$, window size (1200×800), panels swapped (stats left), mode toggle added. Built screenshot-mcp (WGC, no focus steal, Content::image). Registered in .mcp.json. |
| 2026-02-23 | screenshot-mcp debug: fixed stop()→wait() race condition in capture_window(). New binary at .exe.new. .mcp.json updated. Restart Claude Code to activate. UI audit complete — 6 violations documented in HANDOFF ready to implement. |
| 2026-02-23 | UI polish: screenshot-mcp verified working. Applied 5/6 audit fixes (focus_visible, single seam, chrome font, hover signal, border-radius 4px). Fix 3 letter_spacing not available in floem 0.2. Window config + panel swap committed. Code reviewed — fixed missed NPC name font, double prefs.get(), renamed left_panel→story_panel / right_panel→sidebar_panel. Merged to master. |
| 2026-02-23 | Writing guide session: docs/writing-guide.md written. NE US locale, Minijinja syntax, FEMININITY dial, four transformation textures, content gating (BLOCK_ROUGH/LIKES_ROUGH), markdown in prose, scene design principles, full checklist. Adapted from newlife-plus writing-style.md + scene-design.md. Added to CLAUDE.md key documents. |
| 2026-02-23 | UI session: 3-agent team. Custom title bar (no OS chrome, Game/Saves/Settings nav, window controls). Prose centered in story panel. Choice detail strip (hover shows action.detail). Sepia theme darkened (warm amber-cream, not muddy). 87 tests pass. Documented game-input-mcp plan (PostMessage, no focus steal). |
| 2026-02-23 | Built game-input-mcp: press_key + click tools via PostMessage, no focus steal. Release binary built, .mcp.json updated. Restart to activate. |
| 2026-02-23 | Engineering hardening 1: 3-agent team. Window resize grips, prose centering, single-instance (fs4), Display impls for all domain enums, lexer overflow fix, engine expects, scheduler wired to SceneFinished, multi-pack scene loading, pack error visibility. 88 tests, 0 warnings. |
| 2026-02-23 | Engineering batch: 4 parallel agents in worktrees. packs_dir fix, female NPC effects, NpcActivated event, Literata font embed, markdown prose rendering. 95 tests, 0 warnings. |
| 2026-02-23 | Engineering hardening 2: FEMININITY unified (removed Player.femininity field, reads from skills map), w.hasStuff() wired to player inventory via StuffId registry, stats registration added to pack system (stats.toml), panics eliminated in error-recovery paths, spawner unwraps hardened. 100 tests, 0 warnings. |
| 2026-02-23 | Overnight autonomous session: 7 tasks via subagent-driven-development. Names → NE US, settings persistence (dirs + serde_json), character creation UI (AppPhase, PreGameState/GameState split, full form with floem widgets), saves tab (save/load/delete), rain_shelter rewrite (proper prose, 5 trait branches, transformation), morning_routine scene (domestic, mirror, wardrobe, Dunkin'), coffee_shop scene (NPC interaction, sit-with-him path, game flags). 104 tests, 0 warnings. |
| 2026-02-23 | Playtest + bugfix session: Fixed 3 bugs — char creation skipped (title bar now always visible), scroll broken (floem shrink_to_fit + flex_basis(0)), take().unwrap() crash (replaced with match). Added Runtime Testing Notes to CLAUDE.md. Built game-input-mcp scroll + hover tools. Documented char creation redesign ideas (male-first flow, keyboard controls). 104 tests, 0 failures. |
| 2026-02-23 | PC Origin System: Replace always_female:bool with PcOrigin enum (CisMaleTransformed/TransWomanTransformed/CisFemaleTransformed/AlwaysFemale). Two-step char creation flow. Trans woman PC type (FEMININITY=70). Auto-inject hidden traits in new_game(). w.pcOrigin() evaluator accessor. Save v2 with v1 migration. Trans woman branches in all 3 scenes. Writing guide updated with four-origin model + emotional register guidance. 111 tests, 0 failures. Deployed game-input scroll+hover binary. |
| 2026-02-23 | Engine correctness & safety pass: 7 tasks + 2 audit fixes. Scroll-to-bottom (scroll_gen signal), cross-reference validation (UnknownGotoTarget), transition counter guard (replaces stack depth), NPC personality rendering (String instead of PersonalityId), condition eval logging (eval_condition helper), unknown scene surfacing (ProseAdded error), data-driven opening scene/slot (manifest fields → registry → UI). Audit found 8 issues — fixed both HIGH (hardcoded scene ID in saves_panel, silent unwrap_or in scheduler). Engineering Principles added to CLAUDE.md. Remote configured (github-mirror SSH alias). 119 tests, 0 failures. |
| 2026-02-23 | Devtools imported into repo as tools/ (separate workspace). Fixed Windows-only tools (screenshot-mcp, game-input-mcp) to compile on Linux via #[cfg(target_os = "windows")] module gates and target-specific Cargo deps. All 4 tools build cleanly. Global permissions set to bypassPermissions for subagents. |
| 2026-02-23 | MCP cross-platform fix. All configs had hardcoded Windows paths. Added tools/mcp-launcher.mjs (OS-aware, self-locating via import.meta.url, appends .exe on Windows). .mcp.json now uses node + launcher for all 4 servers. post-edit-check.mjs and settings.json hook also de-hardcoded. rust MCP removed pending source migration into tools/ (source only exists on Windows machine). |
| 2026-02-23 | rust-mcp migration: Ported from rmcp 0.2 to 0.8 (ErrorData, wrapper::Parameters, params.0 access pattern). 22 tool methods updated. Release binary builds cleanly. Added to .mcp.json. All 5 MCP servers now in-repo. Cleanup pass: extracted dispatch() helper (-728 lines), removed dead code (ToolDefinition, get_tools, lsp.rs, service.rs), fixed error handling (McpError::internal_error instead of swallowing). Added CLAUDE.md skill override: always merge, never offer discard. |
| 2026-02-23 | Engineering tasks: 7-task plan executed in worktree. NumberKeyMode enum + UserPrefs (theme.rs), ErrorOccurred event + advance_with_action (engine.rs), silent stat effects fix (effects.rs), races from pack data (races.toml + registry + char creation), story cap (200 paras) + free_time fix + dispatch refactor (lib.rs), keyboard controls redesign (arrow nav, highlight, Confirm mode), settings panel (theme/font/line-height/number-key-mode). Code reviewed — fixed `drop` variable shadow. 124 tests, 0 failures. |
| 2026-02-23 | Tooling + scroll fix session. screenshot-mcp rewritten to persistent WGC sessions (10fps, ~20ms reads). game-input-mcp: WM_MOUSEMOVE before WM_MOUSEWHEEL (floem cursor_position routing fix), added start_game/stop_game/is_game_running lifecycle tools (Toolhelp32). Scroll-to-bottom fixed: root cause was floem timing — scroll_to deferred message fires with stale child_size when content change and scroll signal are in same reactive batch. Fix: exec_after(Duration::ZERO) defers scroll to next frame; skip scroll on first prose of new scene (start at top). CLAUDE.md: Engineering Principle #8 (no tech debt/workarounds/hacks), background task ≠ game exit guardrail. 124 tests, 0 failures. |
| 2026-02-23 | Engine foundation: 9 tasks from `docs/plans/2026-02-23-engine-foundation.md` (Sessions A/B/C, D deferred). Batch 1 (sequential): BeforeIdentity struct (domain), trait groups/conflicts (domain+packs), categories system (domain+packs+expr), TimeSlot enum (domain+world+scene). Batch 2 (4 parallel agents): 25+ evaluator methods (expr), 13 new effect types (scene/effects+types), slot routing + once_only/trigger scheduler (scene/engine+scheduler), save v3 with v2→v3 migration (save). Cross-agent integration: fixed PickResult type mismatch in UI (scheduler.pick() now returns PickResult), added SlotRequested event handler. Task 9: hub scene plan_your_day.toml, schedule.toml updates. 28 files changed, +1709/-118 lines. 153 tests (29 new), 0 failures. Merged to master (--no-ff). |
| 2026-02-24 | Engine foundation audit: 4 parallel audit agents reviewed all crates against plan. 15 findings (4 HIGH, 2 MED, 6 LOW, 3 NOTE). 3 parallel fix agents resolved 10 issues: CategoryType String→enum (data.rs+eval.rs), SetVirgin unknown type → error (effects.rs), LateTwenties added to AGE_YOUNG (categories.toml), MaleFigure Display impl (enums.rs), test rename v2→v3 (save), redundant check_triggers guard removed (scheduler.rs), 16 new tests (inCategory, beforeInCategory, check_triggers, 5 NPC effects, before=None paths). 5 findings resolved without code changes (deferred or not bugs). CLAUDE.md updated: background agents must not run cargo build/check/test (file lock contention). 169 tests, 0 failures. |
| 2026-02-24 | Engine routes foundation: 28-task plan (worktree: engine-routes-foundation). Engine: skill roll cache (RefCell<HashMap> in SceneCtx), checkSkill/checkSkillRed evaluator methods (percentile, clamped 5–95), arc_states + red_check_failures in GameData, arcState/arcStarted/arcAdvanced evaluator methods, full prose template context (getSkill/getMoney/getStress/timeSlot/wasTransformed etc.), thought system ([[thoughts]] → ThoughtAdded event, inner_voice/anxiety styles), narrator variants ([[intro_variants]] conditional intro replacement), arc effects (AdvanceArc/SetNpcRole/FailRedCheck), NPC roles (roles field on NpcCore, hasRole() evaluator), arc data format (arcs.toml, ArcDef, registry), route flags (starting_flags/starting_arc_states in CharCreationConfig), validate-pack binary. Docs: docs/world.md, docs/characters/robin.md + camila.md, docs/arcs/robin-opening.md + camila-opening.md, docs/writing-samples.md. Content: Robin arc (5 scenes: robin_arrival, robin_landlord, robin_first_night, robin_first_clothes, robin_first_day), Camila arc (5 scenes: camila_arrival, camila_dorm, camila_orientation, camila_library, camila_call_raul). 14 total scenes. 197 tests, 0 failures. |
| 2026-02-24 | Char creation redesign: 10-task plan (worktree: char-creation-redesign). AppPhase expanded to 4 variants (BeforeCreation/TransformationIntro/FemCreation/InGame). PartialCharState accumulates before-choices. PackRegistry+Scheduler derive Clone (throwaway world for intro scene). transformation_scene field in manifest/registry/loader. char_creation_view (BeforeCreation) + fem_creation_view (FemCreation). TransformationIntro phase runs transformation_intro scene against throwaway world. dispatch_action phase check transitions scene-finish → FemCreation. AlwaysFemale skips TransformationIntro. transformation_intro.toml scene with CisMale/TransWoman voice branches. Writing guide: AI-ism anti-patterns (staccato declaratives, over-naming), BG3 narrator reference. dev/CLAUDE.md skill overrides: finishing-a-development-branch auto-merges (no options prompt). 198 tests, 0 failures. |
| 2026-02-24 | Prolific session (partial — phases 1–3 of 8). Engine: gd.arcState() added to prose template context (2 new tests, 200 total). Writing guide: removed stale notes for getSkill/arcState, expanded template objects table with all current methods, updated FEMININITY section with live usage, added arcState branching example. Prose revision: rain_shelter AI-isms fixed (default nod named→shown, CisMale interiority shows-the-look-not-names-category, trailing staccato cut, NPC action prose upgraded), transformation_intro CisMale branch rewritten (removed anaphoric repetition, removed isolated staccato), CisFemaleTransformed branch added. Char creation: OUTGOING + OVERACTIVE_IMAGINATION added to trait grid (14 traits total), Next button guards empty before_name, FemCreation race defaults to before_race carry-forward. Phases 4–8 deferred. 200 tests, 0 failures. |
| 2026-02-24 | Playtest feedback pass (Batches 1–3). Batch 1: Dropdown Night-mode theming fixed (list_item_view + themed_item helper), "Evan" default removed + Randomize button added, Age::Twenties → MidLateTwenties + v4 save migration, origin radio subtitles. Batch 2: transformation_intro.toml full rewrite — second person, 4 beats, alwaysFemale/TRANS_WOMAN/default 3-way branches, SHY/AMBITIOUS/OUTGOING/OVERACTIVE_IMAGINATION trait branches, TRANSFORMATION_WITNESSED flag. Writing-reviewer audit: 4 Critical + Important fixes applied. Batch 3: Robin/Raul preset character selection (3-way mode: Robin/Raul/Custom), dyn_container for preset-vs-custom UI, section_preset_detail with blurb+read-only rows, build_next_button unified for preset+custom paths. New traits: ANALYTICAL, CONFIDENT, SEXIST, HOMOPHOBIC, OBJECTIFYING (traits.toml + custom mode UI + preset trait lists). Design doc updated with trait philosophy, race-change mechanics, "coming soon" greyout notes. 200 tests, 0 failures. Branch: playtest-fixes. |
| 2026-02-26 | Sprint 3: "Robin's Playable Loop" (worktree: sprint3-robins-playable-loop). 14 new scenes: 7 work slot (work_standup, work_lunch, work_late, work_corridor, work_friday, work_marcus_coffee, work_marcus_favor), 5 free_time (bookstore, park_walk, grocery_store, evening_home, neighborhood_bar), 2 Jake follow-up (coffee_shop_return, jake_outside). Engine: gd.npcLiking(role) evaluator added + set_npc_role effects in coffee_shop + workplace_work_meeting. Schedule.toml: work slot wired for settled state, 5 free_time additions, 2 Jake scenes gated on MET_JAKE. Writing audit pass: Critical/Important fixes across all 14 scenes (staccato, over-naming, AlwaysFemale blank fixed in work_marcus_favor, structural inconsistencies). Three weakest free_time scenes rewritten for heat: neighborhood_bar someone_buys_a_drink (both-sides-of-the-transaction angle), evening_home spend_time_on_yourself (intimate mirror scene), park_walk sit_for_a_while (glance moment). Writing guide: "scene must earn its place" section + two checklist items. 220→223 tests, 0 failures. Merged to master, worktree removed. |
| 2026-03-08 | Button overflow bug fix + floem skill update. Fixed 3rd-button clipping in story panel: root cause is floem 0.2.0 hardcoding `overflow: Visible` in `to_taffy_style()`, preventing flex-based shrinking of scroll. Fix: reactive `max_height` on scroll_area computed from action count (left_panel.rs). Updated floem-layout skill with "Scroll flex-shrink bug" section documenting root cause, workaround pattern, and 5 failed approaches. Updated MEMORY.md. 262 tests, 0 failures. |
| 2026-03-07 | Writing overhaul session (partial). Overhauled transformation writing direction: banned "you used to do this" pattern, replaced with involuntary physical reactions. Updated all 4 writing docs (writing-guide, creative-direction, writer-core, review-core). Removed androgynous name system. Designed scene roster (33→25, 8 scrapped). Drafted 3 example scenes (transformation_intro, workplace_arrival, neighborhood_bar) — stashed, not at quality bar. User feedback: drop `{% if not w.alwaysFemale() %}` guards (write transformation content directly), still too much telling (narrator puts thoughts in player's head). Fixed writing-samples.md (removed banned pattern from Sample 1). Prose calibration incomplete — resume with user before autonomous work. |
| 2026-02-26 | Sprint 2: "FEMININITY Moves" (worktree: sprint2-femininity-moves). 2 batches. Batch 1 (TDD): wrote failing test `femininity_reaches_25_by_workplace_arc_end`, added `skill_increase FEMININITY` effects to all 7 workplace scenes (+2/+2/+2/+5/+3/+3/+3 = 20 gain, 10→30 at arc end), test passed. Batch 2 (content): coffee_shop — removed "geometry to being a woman" over-naming, replaced with concrete spatial awareness shown directly; plan_your_day — full rewrite from stub to real hub scene with time-slot-aware intro, FEMININITY-gated intro_variants at <20 and 20–49, 4 choices (go_out/run_errands/work_on_something/stay_in), AMBITIOUS/ANALYTICAL/default stay_in branches, inner_voice thought gated at FEMININITY<35. writing-reviewer: 0 Criticals. 219→220 tests, clippy clean, validate-pack clean. Merged to master, worktree removed. |
| 2026-02-26 | Code review of Sprint 1: 0 Critical, 3 Important, 4 Minor findings. Fixes applied: AddTrait conflict path now returns EffectError::TraitConflict (previously silent eprintln); FemaleNpc hasTrait added to validate_condition_ids; get_skill_def guard changed to expect (was if let Some, could silently skip clamp); get_stat doc comment added warning about interning semantics. Campus integration test and AddTrait conflict test deferred to sprint backlog. 219 tests, 0 failures. |
| 2026-02-26 | Sprint 1: "The Engine Works" (worktree: sprint1-engine-works). 6 batches, 11 tasks, TDD throughout. Batch 0: removed dead default_slot field + has_before_life alias + unused anyhow dep + dead NpcSnapshot impl. Batch 1: scheduler load failure → visible init error; ArcDef.initial_state removed (dead field); SkillIncrease clamped to SkillDef min/max; FEMININITY min fixed 0→correct. Batch 2: validate stat/skill names in AddStat/SetStat/FailRedCheck at load time; validate_trait_conflicts wired into validate-pack binary; condition expression IDs (trait/skill/category) validated at load time. Batch 3: workplace_first_clothes made reachable by splitting week_one into sequential states (clothes_done); workplace_landlord trigger requires arcState=='arrived'. Batch 4: effect errors emit ErrorOccurred event instead of silent eprintln. Batch 5: full workplace arc playthrough integration test (7 scenes, scheduler to settled, no errors). 208→219 tests, 0 failures. validate-pack clean. Merged to master, worktree removed. |
| 2026-02-25 | Playtest feedback Batch 4 (week-2 scenes) + Batch 1 dropdown fix. Dark-mode dropdown trigger text fixed via themed_trigger helper applied to all 5 Dropdown instances in char_creation.rs. Four week-2 scenes written (parallel scene-writer agents), writing-reviewer audits run on all four, all Criticals and Importants addressed: staccato closers, em-dash reveals, over-naming, POV leaks (you/your in third-person), desire/shame ordering (HOMOPHOBIC branch), missing alwaysFemale() guards (Raul references), SEXIST hierarchy insight unlocked as default !alwaysFemale() path. schedule.toml updated with all 4 scene entries. All 200 tests pass. Merged playtest-fixes to master, worktree removed. |
