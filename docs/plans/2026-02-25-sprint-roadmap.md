# Sprint Roadmap — Playable Loop First

**Created:** 2026-02-25
**Approach:** Fix the engine, then fill it with content. No quality shortcuts.

## Quality Protocol — Every Sprint

**Engineering:**
- TDD: failing test first, then implementation
- `cargo test --workspace` + `cargo clippy` clean after every batch
- `validate-pack` must catch what we're fixing
- No workarounds — proper fixes only (Principle #8)

**Content:**
- Writing-reviewer audit on every scene touched
- All Criticals fixed before merge. Importants addressed or documented with rationale.
- Template validation on every prose block
- Second-person present tense verified (zero PC "she/her" in prose)

**Every sprint ends with:**
- Full test + clippy pass
- HANDOFF.md updated
- Session log entry
- Working tree clean on master

---

## Sprint 1: "The Engine Works"

**Goal:** A player can start a game, play through an opening arc, and not hit silent
failures or unreachable content.

| Task | Category | What |
|---|---|---|
| 1.1 | Engine Bug | Fix `add_npc_liking` silent fail — require active NPC or surface error. Test reproduces the bug first. |
| 1.2 | Reachability | Fix `workplace_first_clothes` unreachable — restructure week_one trigger ordering. Scheduler test proves both scenes reachable. |
| 1.3 | Reachability | Fix `workplace_landlord` — add arc state check to trigger condition. |
| 1.4 | Engine Bug | Fix scheduler load failure — promote to visible error, not silent empty. |
| 1.5 | Engine Bug | Wire `ArcDef.initial_state` in `new_game()` OR remove it. No dead fields. |
| 1.6 | Engine Bug | Fix FEMININITY min/max — correct skills.toml, add clamp to skill_increase. |
| 1.7 | Validation | `validate_effects` checks stat/skill names at load time. Wire into validate-pack. |
| 1.8 | Validation | Category IDs validated at load time. Wire into validate-pack. |
| 1.9 | Cleanup | Remove dead `default_slot` from pack.toml + manifest struct. |
| 1.10 | Cleanup | Remove dead `From<&NpcCore>`, unused anyhow dep, redundant `has_before_life()`. |
| 1.11 | Integration | Add integration test: simulate full workplace arc playthrough — start → pick_next loop → all scenes reachable → arc completes. |

**Done when:** `cargo test` passes (including new integration test), `validate-pack` catches
all previously-silent errors, `workplace_first_clothes` is reachable, NPC liking effects land.

---

## Sprint 2: "FEMININITY Moves"

**Goal:** FEMININITY actually increments during gameplay. FEMININITY-gated content is
reachable. The hub scene is real.

| Task | Category | What |
|---|---|---|
| 2.1 | Design | Document FEMININITY progression curve — which scenes grant how much, expected value at each arc stage. Write to docs/plans/. |
| 2.2 | Content | Add `skill_increase FEMININITY` effects to appropriate workplace scene actions. |
| 2.3 | Content | `plan_your_day` full rewrite — hub scene with real choices, FEMININITY-appropriate branches. Full scene-writer → writing-reviewer → fix cycle. |
| 2.4 | Content | Coffee_shop prose fix — remove "geometry to being a woman" over-naming. |
| 2.5 | Verification | Test that simulates playthrough and asserts FEMININITY reaches 25+ by arc end. |

**Done when:** Playthrough naturally reaches FEMININITY 25+. `plan_your_day` is a real
scene that passes writing-reviewer with zero Criticals. All FEMININITY-branched prose
has a reachable path.

---

## Sprint 3: "Campus Catches Up"

**Goal:** Campus arc scenes match workplace quality: second-person, no Critical prose
issues, playable end-to-end.

| Task | Category | What |
|---|---|---|
| 3.0 | Pre-work | Read all 7 campus scenes, understand arc flow, write NPC docs for campus NPCs. Writers need character docs before touching prose. |
| 3.1 | Content | Rewrite all 7 campus scenes to second-person (parallel scene-writer agents). Feed writing-audit Criticals as input to agents. |
| 3.2 | Content | Add FEMININITY increments to campus arc scenes. |
| 3.3 | Validation | Template validation on all campus files. |
| 3.4 | Review | Writing-reviewer pass on all campus scenes. Zero Criticals. Feed new patterns back into writing-guide/scene-writer/writing-reviewer. |

**Done when:** Campus arc plays end-to-end in second person. Zero Critical findings.
FEMININITY increments. NPC docs exist for all campus NPCs.

---

## Sprint 4: "After the Arc"

**Goal:** The game doesn't dead-end after the opening arc. There's a real free_time
experience with variety.

| Task | Category | What |
|---|---|---|
| 4.1 | Design | Brainstorm + design post-arc life: what scenes fire, weekly loop shape, recurring NPCs, progression hooks. Document in docs/plans/. |
| 4.2 | Content | Write 3–5 new free_time scenes (universal). Full writing pipeline per scene. |
| 4.3 | Content | Add weight decay or variety mechanism to scheduler so morning_routine doesn't dominate. |
| 4.4 | Content | Write 1–2 settled scenes (workplace post-completion). |
| 4.5 | Content | Write 1–2 first_week scenes (campus post-completion). |
| 4.6 | Pre-work | NPC character docs for every NPC who appears in new scenes. |
| 4.7 | Verification | Simulate 4 weeks post-arc play. Count unique scene encounters. Verify variety quantitatively. |

**Done when:** 6+ unique scene encounters before repeats after arc completion. Weekly
loop feels varied. morning_routine ≤ 30% of encounters.

---

## Sprint 5: "Design Debt"

**Goal:** Engine stops hardcoding content. Presets become data. Test friction reduced.

| Task | Category | What |
|---|---|---|
| 5.1 | Design Debt | Presets as pack data — load from TOML instead of static Rust structs. |
| 5.2 | Design Debt | Remove hardcoded trait/skill IDs from char_creation.rs. Grep verification: zero hits in non-test code. |
| 5.3 | Design Debt | Spawner uses pack-loaded races instead of hardcoded list. |
| 5.4 | Design Debt | Test fixture DRY — shared `test-fixtures` crate. Migrate ALL existing test modules. |
| 5.5 | Design Debt | SceneId newtype: use everywhere or remove. |
| 5.6 | Validation | `validate_trait_conflicts` wired into validate-pack. |

**Done when:** Zero hardcoded content IDs in engine Rust code (verified by grep). Presets
load from TOML. New Player/BeforeIdentity field changes require updating one helper.

---

## Parking Lot

Items not yet sprinted. Promote when relevant.

- **Prose polish pass** — Important-level writing fixes across workplace scenes (staccato, adjective swaps, emotion announcements). Do after Sprint 3.
- **NPC relationship infrastructure** — persistent NPC records, follow-up scenes. Needs design.
- **Player choice consequences** — branching flag effects per-scene. Needs design.
- **UI polish** — para cap bypass, thought scroll, saves scroll, tab states, hover artifact.
- **Unused traits/skills/stats** — wire into scenes or remove from data files.
- **Custom character starting scenario** — freeform arc-agnostic content. Deferred.
- **Parser recursion depth limit** — theoretical concern, no active exploit path.
