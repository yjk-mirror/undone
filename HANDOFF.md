# Undone — Handoff

## Current State

**Branch:** `master`
**Tests:** 224 passing, 0 failures.
**Scenes:** 33 total (19 pre-sprint + 14 new).
**Content focus:** CisMale→Woman only. AlwaysFemale, TransWoman, CisFemale all deprioritized.
**Sprint 1 complete + reviewed:** "The Engine Works" — 208→219 tests. All engine bugs fixed, all arc scenes reachable.
**Sprint 2 complete:** "FEMININITY Moves" — FEMININITY increments in all 7 workplace scenes (+20 total). plan_your_day rewritten. 219→220 tests.
**Sprint 3 complete:** "Robin's Playable Loop" — 14 new scenes, full writing audit pass. 220→223 tests.
**TRANS_WOMAN branches removed** from all scene files. CisMale-only pattern: `{% if not w.alwaysFemale() %}` with no `{% else %}`.
**All prose second-person present tense.** Zero third-person PC narration.
**Writing toolchain:** writing-guide updated with positive "scene must earn its place" rule. scene-writer + writing-reviewer custom agents current.

---

## ⚡ Next Action

**MCP Playtest — COMPLETE ✅**

Full 9-step playtest passed with Robin preset (ANALYTICAL trait, Night theme). Two bugs found and fixed during playtest:

1. **NPC wiring bug** — `StartScene` without `SetActiveMale`/`SetActiveFemale` caused "effect requires active male NPC but none is set" errors. Fix: `start_scene()` helper in lib.rs wraps StartScene + NPC wiring. 5 callsites replaced.
2. **`gd.week()` stuck at 0** — No scene fired `advance_time`, so week counter never incremented. All free_time scenes gate on `gd.week() >= 1` → permanently unreachable. Fix: added `advance_time slots=28` to `workplace_evening.toml` (final arc scene). TimeSlot has 4 variants (Morning/Afternoon/Evening/Night), so 7 days × 4 = 28 slots = 1 week.

**Playtest results:**
1. ✅ Char creation → transformation_intro → fem creation → game start
2. ✅ Workplace arc (7 scenes, linear triggers, FEMININITY 10→35)
3. ✅ `settled` state reached, work slot scenes firing (plan_your_day, work_corridor, work_late)
4. ✅ Free_time scenes rotating (grocery_store, morning_routine, evening_home, bookstore, rain_shelter, park_walk, coffee_shop — all seen)
5. ✅ Jake thread: coffee_shop → MET_JAKE set → "Jake. With a dog." appears in park_walk
6. ✅ Marcus thread: "Marcus is still here" action in work_late scene
7. ✅ FEMININITY-gated intro_variants (ANALYTICAL branches firing at FEMININITY 20–49 range)
8. ✅ Trait branches (ANALYTICAL branch content visible in workplace_evening, evening_home, plan_your_day, work_corridor)

**After playtest:** User testing and feedback. Then plan Sprint 4.

### Remaining open items (post-Sprint 3)
- **Post-arc content void** — Sprint 3 expanded free_time from 3→8 scenes and added 7 work slot scenes (settled state). Remaining gap: campus arc has no post-arc slot equivalent. → Sprint 4+.
- **Prose polish pass** — Workplace arc prose is mostly clean after Sprint 2–3 audit passes. Campus arc has ~20 open Critical/Important writing findings from the 2026-02-25 audit. → Sprint 4+.
- **Free_time expansion** — more universal scenes needed. → Sprint 4.
- **NPC character docs missing** — 13 named NPCs have no character docs (Marcus, David, Jake, Frank, etc.). → As needed.
- **Presets as pack data** — `PRESET_WORKPLACE` / `PRESET_CAMPUS` are static Rust structs. → Sprint 5.
- **Test fixture DRY** — 8+ identical `make_world()` helpers across crates. → Sprint 5.
- **Hardcoded content IDs in engine code** — `FEMININITY`, `TRANS_WOMAN`, `ALWAYS_FEMALE`, `NOT_TRANSFORMED`, `BLOCK_ROUGH`, `LIKES_ROUGH` appear as string literals in Rust code. Engineering Principle 2 violation.
- **Parser recursion depth limit** — `undone-expr` recursive descent parser has no depth guard. Engineering Principle 5 violation (unbounded growth).
- **Saves panel scroll** — `saves_panel.rs` is missing `shrink_to_fit()` on its scroll container; scrolling may not activate with many saves.
- **Tab buttons active during char creation** — clicking Game/Saves/Settings tabs during char creation has no visible effect but the buttons appear clickable (no disabled state).

### User playtest feedback (2026-02-27, second session)

**UX / Navigation:**
1. **Settings inaccessible from "Your Story Begins"** — Can't click Settings tab during char creation. User wants to adjust text size before starting. Related: I12 in engineering audit (tab buttons active but no effect).
2. **No landing page / load game screen** — No way to load a saved game before starting a new one. Game launches straight into char creation. Needs a title screen or launcher with New Game / Continue / Load / Settings.
3. **Deferred Settings teleport** — User clicked Settings during char creation (no effect), then later got randomly teleported to the Settings tab mid-game. Tab click was queued or state leaked across phases.
4. **Default text size too small** — Feels small on first launch. User wants to be able to change it before playing.

**Char creation:**
5. **"Who Are You Now" screen extremely lacking** — Known issue (attribute dropdowns not implemented). But user is hitting it now as a real blocker to the experience feeling complete.
6. **Names wrong** — Character names don't match what was discussed/decided on. Need to update preset names to the agreed-upon names.
7. *(Previous)* Trait list runoff, post-transformation attributes in before-phase, attribute formatting — still open.

**Opening scene:**
8. **Wrong opening scene** — Current `transformation_intro` is not what was discussed. The agreed opening is: he gets off a plane, arrives in the city. The transformation scene should follow the arrival framing, not precede it as an abstract standalone.

**Story panel layout:**
9. **Flavor text box too small** — The detail strip (action hover/highlight description area) feels too cramped.
10. **Action buttons misaligned with prose** — Selections/choices should start at the same left edge as the prose text, not be offset from it.

**Sidebar:**
11. **NPC display — wrong character showing** — Left sidebar shows a random NPC the user doesn't recognize. No context for who they are or why they're shown.
12. **NPC formatting broken** — Whatever is displayed isn't properly formatted.
13. **Multiple NPC display unclear** — No visible plan for how the sidebar handles multiple NPCs.

**Writing quality:**
14. **Rain scene writing bad** — Too much telling-not-showing. Narrator puts thoughts directly into PC's brain ("you know what they think because you've been him"). Violates the show-don't-tell principle. The "you know what men think because you were one" angle is too explicit and repetitive — it should be shown through specific moments, not stated as narration.
15. **Repetitive transformation narration** — Multiple scenes hammer the same "you used to be a man so you understand" beat explicitly instead of letting it emerge from concrete observations. Needs subtlety — the insight should be demonstrated through what the PC notices, not announced by the narrator.
16. **User considering DeepSeek API for writing agents** — Wants to discuss using DeepSeek alongside Claude for scene writing before next writing sprint. Do not act on this yet.

### Char creation bugs (previous session)
- **Trait list runoff** — "Starting traits" row overflows off-screen. No wrapping, no colon/space between label and values. Runs off the right edge.
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
| 2026-02-26 | Sprint 2: "FEMININITY Moves" (worktree: sprint2-femininity-moves). 2 batches. Batch 1 (TDD): wrote failing test `femininity_reaches_25_by_workplace_arc_end`, added `skill_increase FEMININITY` effects to all 7 workplace scenes (+2/+2/+2/+5/+3/+3/+3 = 20 gain, 10→30 at arc end), test passed. Batch 2 (content): coffee_shop — removed "geometry to being a woman" over-naming, replaced with concrete spatial awareness shown directly; plan_your_day — full rewrite from stub to real hub scene with time-slot-aware intro, FEMININITY-gated intro_variants at <20 and 20–49, 4 choices (go_out/run_errands/work_on_something/stay_in), AMBITIOUS/ANALYTICAL/default stay_in branches, inner_voice thought gated at FEMININITY<35. writing-reviewer: 0 Criticals. 219→220 tests, clippy clean, validate-pack clean. Merged to master, worktree removed. |
| 2026-02-26 | Code review of Sprint 1: 0 Critical, 3 Important, 4 Minor findings. Fixes applied: AddTrait conflict path now returns EffectError::TraitConflict (previously silent eprintln); FemaleNpc hasTrait added to validate_condition_ids; get_skill_def guard changed to expect (was if let Some, could silently skip clamp); get_stat doc comment added warning about interning semantics. Campus integration test and AddTrait conflict test deferred to sprint backlog. 219 tests, 0 failures. |
| 2026-02-26 | Sprint 1: "The Engine Works" (worktree: sprint1-engine-works). 6 batches, 11 tasks, TDD throughout. Batch 0: removed dead default_slot field + has_before_life alias + unused anyhow dep + dead NpcSnapshot impl. Batch 1: scheduler load failure → visible init error; ArcDef.initial_state removed (dead field); SkillIncrease clamped to SkillDef min/max; FEMININITY min fixed 0→correct. Batch 2: validate stat/skill names in AddStat/SetStat/FailRedCheck at load time; validate_trait_conflicts wired into validate-pack binary; condition expression IDs (trait/skill/category) validated at load time. Batch 3: workplace_first_clothes made reachable by splitting week_one into sequential states (clothes_done); workplace_landlord trigger requires arcState=='arrived'. Batch 4: effect errors emit ErrorOccurred event instead of silent eprintln. Batch 5: full workplace arc playthrough integration test (7 scenes, scheduler to settled, no errors). 208→219 tests, 0 failures. validate-pack clean. Merged to master, worktree removed. |
| 2026-02-25 | Playtest feedback Batch 4 (week-2 scenes) + Batch 1 dropdown fix. Dark-mode dropdown trigger text fixed via themed_trigger helper applied to all 5 Dropdown instances in char_creation.rs. Four week-2 scenes written (parallel scene-writer agents), writing-reviewer audits run on all four, all Criticals and Importants addressed: staccato closers, em-dash reveals, over-naming, POV leaks (you/your in third-person), desire/shame ordering (HOMOPHOBIC branch), missing alwaysFemale() guards (Raul references), SEXIST hierarchy insight unlocked as default !alwaysFemale() path. schedule.toml updated with all 4 scene entries. All 200 tests pass. Merged playtest-fixes to master, worktree removed. |
