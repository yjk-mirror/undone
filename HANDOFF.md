# Undone — Handoff

## Current State

**Branch:** `playtest-fixes` (open, 4 commits ahead of master — ready to merge after Batch 4)
**Tests:** 200 passing, 0 failures.
**App:** 3-phase character creation working end-to-end. Preset character selection (Robin / Raul / Custom) on BeforeCreation screen. Three-way mode selector with dyn_container. Preset detail view shows blurb + read-only attributes. All traits exposed in custom mode including new attitude traits. Dropdown popup theming fixed (Night mode). Age enum simplified (MidLateTwenties). Origin subtitles. v4 save migration.
**Content:** transformation_intro.toml fully rewritten — second person, 4 beats, 3-way origin branch, trait branches for SHY/AMBITIOUS/OUTGOING/OVERACTIVE_IMAGINATION.
**Traits:** 5 new traits added to traits.toml: ANALYTICAL, CONFIDENT, SEXIST, HOMOPHOBIC, OBJECTIFYING. All non-hidden, group personality/attitude. All exposed in character creation custom mode under "Former attitudes."

---

## ⚡ Next Action

**Batch 4 — Week-2 scenes.** Batches 1–3 complete and committed. Dispatch 4 parallel scene-writer agents for the four week-2 scenes, audit with writing-reviewer, apply fixes, add schedule.toml entries, then merge the playtest-fixes branch to master.

Scene targets:
- `robin_work_meeting.toml` (arc state: `"working"`, Robin week 2)
- `robin_evening.toml` (arc state: `"working"`, Robin week 2)
- `camila_study_session.toml` (arc state: `"first_week"`, Camila week 2)
- `camila_dining_hall.toml` (arc state: `"first_week"`, Camila week 2)

Scenes should use the new attitude traits (SEXIST, HOMOPHOBIC, OBJECTIFYING) where they enrich the experience. Read character docs and arc docs before writing.

---

## Playtest Feedback (2026-02-25)

Issues found during first playtest. All need to be fixed next session.

### Batch 1 — Quick Fixes (UI/data)

**1. Dark mode dropdown text invisible.**
Dropdowns (Age, Sexuality) have unreadable text in the Night theme. Fix the color tokens so dropdown option text is legible across all three themes.

**2. "Evan" default name is wrong.**
The Name Before field should be empty on load. Show suggestion/placeholder text (a random name from the existing names list). Add a "Randomize" button. Do not pre-fill with "Evan".

**3. Age enum too granular — simplify.**
Current variants have too many entries. Desired simplification:
- Remove plain `Twenties`
- Add `EarlyTwenties` ("Early 20s") and `MidLate Twenties` ("Mid to Late 20s")
- Audit the full enum and remove anything overly fine-grained
- Update all match arms, display, save migration as needed

**4. Clarify origin radio labels.**
"Something happened to me — I was a woman" vs "I was always a woman" is confusing. Needs a short descriptive line under each option explaining the difference. Suggested:
- `CisMaleTransformed`: "Something happened to me — I was a man" → *subtext: "Transformed from male. The core experience."*
- `TransWomanTransformed`: "Something happened to me — I was a trans woman" → *subtext: "Already knew yourself. The transformation was recognition."*
- `CisFemaleTransformed`: "Something happened to me — I was a woman" → *subtext: "You were female. Something still changed."*
- `AlwaysFemale`: "I was always a woman" → *subtext: "No transformation. Play as yourself."*

### Batch 2 — Transformation Intro Rewrite

**5. transformation_intro.toml is in third person. This is a critical violation.**
All prose must be second-person present tense. The current scene uses "she/her". Rewrite entirely.

**6. Transformation intro doesn't reflect male character's traits/attributes.**
The scene should branch on the traits the player selected in BeforeCreation (SHY, AMBITIOUS, etc.) and reference physical attributes where relevant. Use scene-writer agent; use writing-reviewer to audit.

**7. Transformation intro is too short and has AI-isms.**
Too brief for a moment of this weight. Should be longer — the character doesn't immediately realise what happened; it takes a few moments to register. Fix all staccato/dramatising patterns per writing guide.

### Batch 3 — Origin Character Presets (New Feature)

**8. Add Robin and Camila as selectable preset origins.**
On the BeforeCreation screen, give the player three options:
- **Start as [Robin's male name]** — shows a brief description of who they were before (no female spoilers), locks all fields, auto-fills everything
- **Start as [Camila's male name]** — same treatment
- **Create your own** — existing form, fully editable

When a preset is selected: show the character description, display all their traits/attributes as read-only, and proceed straight to TransformationIntro. The user cannot change anything.

Read `docs/characters/robin.md` and `docs/characters/camila.md` for their pre-transformation identities. Write the short "who were you before" blurb (second person, no female spoilers) as part of this task.

### Batch 4 — Week-2 Scenes (Content)

**9. Robin week 2:** `robin_work_meeting.toml`, `robin_evening.toml` (arc state: `"working"`)
**10. Camila week 2:** `camila_study_session.toml`, `camila_dining_hall.toml` (arc state: `"first_week"`)

Use parallel scene-writer agents + writing-reviewer audit.

---

**Deferred content phases** (lower priority, from `docs/plans/2026-02-24-prolific-session.md`):
- Phase 7: `plan_your_day.toml` prose depth
- Phase 8: Free-time recurrence variety

---

## game-input-mcp — Updated

**Input tools:** `press_key(title, key)`, `click(title, x, y)`, `scroll(title, x, y, delta)`, `hover(title, x, y)`.
Keys: `"1"`–`"9"`, `"enter"`, `"tab"`, `"escape"`, `"space"`.
Scroll: sends WM_MOUSEMOVE before WM_MOUSEWHEEL (floem routes wheel events via cached cursor_position). Positive delta = up, negative = down (one tick = one wheel notch).
**Lifecycle tools:** `start_game(working_dir)`, `stop_game(exe_name)`, `is_game_running(exe_name)`.
Process management uses Toolhelp32 snapshot API.

---

## screenshot-mcp — Persistent Sessions

Rewrote from one-shot WGC capture to persistent capture sessions (10fps). First request creates session + waits up to 1s for initial frame. Subsequent requests read cached frame (~20ms). Sessions are keyed by window title, evicted when window closes. Fast PNG encoding via `png` crate with `Compression::Fast`.

---

## UI — Current State

**Layout:**
- Title bar always visible: UNDONE branding, Game/Saves/Settings tabs, window controls
- Stats sidebar on the **left** (280px fixed): player name, stats, NPC panel, mode toggle
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
- `crates/undone-ui/src/right_panel.rs` — stats sidebar, NPC panel, mode toggle
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
