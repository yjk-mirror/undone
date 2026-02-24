# Prolific Session Plan — 2026-02-24

Multi-hour autonomous work plan. Work through tasks in order. Each phase has a clear
completion criterion. Use a single worktree for the whole session.

Read `HANDOFF.md`, `docs/writing-guide.md`, and all arc docs before touching anything.

---

## Worktree Setup

```bash
git worktree add .worktrees/prolific-session -b prolific-session
```

Work in `.worktrees/prolific-session` for everything. Merge to master when complete.

---

## Phase 1 — Engine: `gd.arcState()` in prose template context

**Why first:** Every arc-aware scene written from here on will want to branch prose on arc
state. Adding it now unblocks all future scene writing. Small, bounded, high leverage.

**File:** `crates/undone-scene/src/template_ctx.rs`

The `GameDataCtx` struct exposes `week()`, `day()`, `timeSlot()`, `hasGameFlag()`, etc.
`gd.arcState("arc_id")` is available in the *expression evaluator* (`undone-expr`) but NOT
in the minijinja template context. Add it.

**Steps:**
1. In `template_ctx.rs`, find `GameDataCtx` and add a method `arc_state(&self, arc_id: &str) -> String`
   that returns `world.game_data.arc_states.get(arc_id).map(|s| s.as_str()).unwrap_or("")`
2. Register it with minijinja — match how `hasGameFlag` is registered
3. Update `docs/writing-guide.md` — remove the stale note "gd.arcState() not available in prose
   templates" if there is one; add it to the template objects reference table
4. Add a test in the scene engine that a prose template using `{% if gd.arcState("base::robin_opening") == "working" %}` renders the correct branch
5. `cargo test -p undone-scene` must pass

Also while in `template_ctx.rs`: confirm `w.getSkill("FEMININITY")` is already live (the
writing guide has a stale "not yet available" note). If it's live, remove that note from
`docs/writing-guide.md`.

---

## Phase 2 — Prose Revision: `rain_shelter` and `transformation_intro`

**Why:** Both scenes have AI-ism issues — staccato declaratives for effect, em-dash reveals,
over-named experiences. See writing guide anti-patterns 9 and 10.

**Reference:** BG3 narrator — dry, plain, second-person, trusts the scene. Nothing
performative. Just what happened.

### `rain_shelter.toml`

Specific problems to fix:
- "the universal stranger-in-shared-misery nod" → just show the nod
- "There's a specific quality to being looked at by a strange man in a small space. You know
  it now. Not danger, exactly — more like being *placed*." → too much naming; find the image
  instead of the category
- "He's already decided three things about you, and none of them are the things you'd choose."
  → staccato reveal; replace with something more grounded
- "The city goes on." → trailing staccato closer; cut or replace

The transformation interiority block and the five personality trait branches are structurally
good — keep those. Revise the framing prose between them.

Also: `docs/writing-samples.md` Sample 3 quotes richer prose than what's in the file. Either
update the scene to match the sample (if the sample is the target) or update the sample to
match the scene. Pick whichever is better prose.

### `transformation_intro.toml`

Specific problems to fix in the CisMale (default `{% else %}`) branch:
- "It happens fast. It happens the way a mirror breaks — not slowly, all at once." →
  anaphoric repetition + em-dash reveal
- "He grabs the counter." → isolated for dramatic effect
- These patterns read as signals rather than as prose

The TransWoman branch ("Oh. There you are.") is better — it earns its short sentences
because that IS the voice (quiet recognition). The CisMale branch should have its own
texture: not shock-and-awe, more like being very still while something enormous is
happening.

Also: **CisFemaleTransformed has no prose.** The `{% if not w.alwaysFemale() %}` block
means CisFemaleTransformed falls through to the `{% endif %}` with nothing. This origin
needs a third voice: different from TransWoman (no relief — something more like
disorientation from a different angle, or possibly something quieter). Add:
```
{% if not w.alwaysFemale() %}
  {% if w.hasTrait("TRANS_WOMAN") %}
    [TransWoman voice — recognition]
  {% else %}
    [CisMale voice — shock/reorientation]
  {% endif %}
{% elif not w.hasTrait("NOT_TRANSFORMED") %}
  [CisFemale voice — her own flavour]
{% endif %}
```
For AlwaysFemale (`{% else %}` or the residual `{% endif %}`), the current behavior is correct
(no transformation prose, scene continues to "Continue" action which is valid).

After revision: validate both templates with `mcp__minijinja__jinja_validate_template`.

---

## Phase 3 — Char Creation UI: Missing Traits + Input Validation

**File:** `crates/undone-ui/src/char_creation.rs`

### 3a. Add `OVERACTIVE_IMAGINATION` and `OUTGOING` to personality grid

Both traits are defined in `traits.toml` but missing from the checkbox grid in
`char_creation_view`. The current grid has 12 traits. Add both (14 total), which may require
adjusting the layout (currently a `grid(2)` — could go to `grid(3)` or keep `grid(2)` with
an extra row).

Note: OUTGOING conflicts with SHY. The Beautiful/Plain mutual-exclusion pattern already
exists in the codebase — use the same approach for SHY/OUTGOING/FLIRTY conflict group if
needed, OR just let the conflict system handle it at pack-load time (conflicts are registered,
the game engine already enforces them).

### 3b. Empty name guard on "Next →" button

If `before_name.get_untracked().trim().is_empty()`, do not transition — show an inline error
or simply don't call the phase transition. A simple guard (no visual error state needed, just
ignore the click) is acceptable for now.

### 3c. Race carry-forward BeforeCreation → FemCreation

The `FemFormSignals::new()` defaults race to the first registry race. If `partial.before_race`
is set, initialize `fem_form.race` from it instead. The player can still change it in the
FemCreation form — this just means the default matches what they picked.

After changes: `cargo check -p undone-ui && cargo fmt -p undone-ui`

---

## Phase 4 — World Canonization: `docs/world.md`

The world doc is a constraint document but thin on established facts. Update it with facts
established across all 15 scenes. Scan every `.toml` scene file for proper nouns, locations,
and world-facts, then add to world.md under appropriate sections.

Facts to canonize (from scene audit):
- **Robin's apartment:** Clement Ave, third floor, landlord Frank (older, wife named Janet
  visible at top of stairs)
- **Robin's company:** Financial district (no name given — leave unnamed unless a scene names
  it). Marcus is her grad school friend.
- **Robin's workplace colleague:** Dan (mid-level, explains things slowly, probably not
  malicious just oblivious)
- **Camila's university:** "The University" (not the Ivy). Diego is Raul's best friend and
  Camila's contact. Adam is a freshman at the Ivy encountered at orientation.
- **Clement Ave:** bus shelter, an ad for a personal injury lawyer peeling off the back panel,
  glass walls and metal bench
- **Currency confirmed:** dollars, specific amounts used in scenes
- **Transport:** buses named (wrong route going by), moped delivery drivers

Also: if `gd.arcState()` in templates is now live (Phase 1), update the Template Syntax Quick
Reference table in writing-guide.md to include it.

---

## Phase 5 — New Scene Content: Robin Week 2 (`working` state)

The Robin arc advances to `working` state after `robin_first_day`. There are currently no
scenes under `working` or `settled`. Write at least two.

**First: plan the arc before writing scenes.** Create/update `docs/arcs/robin-opening.md`
with week 2+ beat sheet. The `working` state should feel like a settled rhythm — she knows
the job, the apartment, the commute. The dramatic territory shifts from "navigating the
basics" to "navigating being a woman in a workplace where she's underestimated."

**Scene ideas (pick the strongest 2–3):**

**`robin_work_meeting.toml`** — A staff meeting. Dan presents something she knows is wrong.
Choices: correct him (AMBITIOUS path fires differently than default), stay quiet (SHY path),
use a softer framing. NPC action: someone agrees with Dan's error. Game flag: `DAN_CORRECTED`
or `DAN_NOT_CORRECTED`. Arc consequence: player stat changes based on choice.

**`robin_evening.toml`** — End of a workday. Coming home to the apartment. Frank in the
hallway or not. Small domestic beat — what she does in the evening when nothing is scheduled.
Could introduce a Marcus phone call (text, actually — "hey haven't heard from you" type).
No major choice needed; this can be a texture scene with one branching beat.

**`robin_marcus_call.toml`** (optional, only if time) — Marcus calls. He knew her before.
He asks how she's doing. Three paths: tell him things are fine, tell him something true,
avoid the call entirely. This is the scene where the past is present without being performed.

**Schedule wiring:** Add new scenes to `schedule.toml` under a new or expanded slot. The
`robin_opening` slot is well-structured — add new scenes under it with appropriate arc_state
conditions (`"arcState('base::robin_opening') == 'working'"` as trigger condition).

**After each scene:** validate template, check scene loads via validate-pack binary.

---

## Phase 6 — New Scene Content: Camila Week 2 (`first_week` state)

Same structure as Phase 5.

**First: update `docs/arcs/camila-opening.md`** with `first_week` beat sheet.
Camila's `first_week` state should feel like finding her footing — Raul's call was a
reckoning, now she's trying to figure out what she's building here vs. what she left.

**Scene ideas (pick 2):**

**`camila_study_session.toml`** — She ends up studying with someone from orientation (Adam
from the Ivy, or a new character — her choice of study partner shapes how it goes). SHY/CUTE
branches differentiate the dynamic. A NPC action: the person asks her something personal
(where are you from, what are you studying). Small consequence: game flag for having a
study partner or not.

**`camila_dining_hall.toml`** — A dining hall encounter. The mundane weight of a new
place — the food, the noise, the people eating in groups. She's alone or she's not. If alone:
OVERACTIVE_IMAGINATION fires a long-form internal branch. If sitting with someone: conversation
paths. This is texture + character, not plot.

**Schedule wiring:** Mirror the Robin pattern for the Camila slot.

---

## Phase 7 — Plan Your Day: Give It Prose Depth

`plan_your_day.toml` is the hub scene that fires once after week 2. It's currently three
sentences with no character. It should feel like a real beat in the week.

The existing structure (go_out → free_time slot, stay_home → finish) is fine. The prose
needs weight. Suggestions:
- Make the "staying home" path feel like a choice with texture, not a dead end
- The "going out" path should feel like stepping into something
- Add at least one branching beat by trait (SHY / AMBITIOUS diverge on whether going out
  feels like relief or reluctance)
- This scene fires once-only with a trigger, so it can reference specific world context
  (e.g., it's been a few weeks, the apartment is more familiar)

---

## Phase 8 — Schedule: Add Recurrence Variety to `free_time`

Currently `rain_shelter` and `coffee_shop` repeat indefinitely with no variation. Three options:

**8a. Game-flag variants within existing scenes** — use `gd.hasGameFlag("BEEN_TO_SHELTER")`
to vary the prose on repeat visits. Both existing scenes already have the bones for this
(they both have `set_game_flag` effects). Add return-visit prose variants.

**8b. New universal scene (1 scene)** — Write one additional short universal scene for
variety. Good candidates: an evening walk, a grocery store trip (ShopRite or Stop & Shop by
name), a park bench scene. Keep it short (2–3 paragraphs, 1–2 choices, one stat consequence).

Pick one option. 8a is faster and improves existing scenes; 8b adds variety. Both have value.

---

## Phase 9 — Audit Pass Before Merge

Before merging, run:
```bash
cargo test --workspace           # all 198+ tests pass
cargo clippy --workspace         # 0 warnings
./target/debug/validate_pack packs/  # all scenes load clean
```

Also run a manual sanity check in the game (start_game MCP tool) — click through to
character creation, verify the new origin branching in transformation_intro works, spot-check
one Robin scene.

Review all committed scene files against the writing checklist in `docs/writing-guide.md`.
Focus especially on:
- Second-person present tense throughout
- No emotion announcements
- No staccato/em-dash patterns (anti-patterns 9 and 10)
- All trait branches are structurally different
- AlwaysFemale paths are complete

---

## Phase 10 — HANDOFF.md Update and Commit

Update HANDOFF.md:
- Current State: reflect new test count, new scenes, arc advances
- Next Action: what naturally follows this session
- Session Log: one entry summarizing everything done

Commit structure (logical groupings, not one giant commit):
1. Engine: `gd.arcState()` in template context + tests
2. Writing guide updates (remove stale notes, add gd.arcState)
3. Prose revision: rain_shelter + transformation_intro
4. Char creation: trait additions + input guards + race carry
5. World.md canonization
6. Robin week 2 scenes + schedule wiring
7. Camila week 2 scenes + schedule wiring
8. plan_your_day revision
9. Schedule variety (recurrence variants or new scene)
10. HANDOFF.md

---

## Scope Notes

**Do in this session:**
- Everything in phases 1–5 (engine, prose, UI, docs) — these are high-confidence, bounded
- Phase 6 (Camila week 2) — do after Robin week 2 is done and feeling right
- Phase 7 and 8 — do if time remains; skip if scenes are taking longer than expected

**Skip if time-constrained:**
- Robin `settled` state (leave for next session — `working` is enough for now)
- Marcus phone call scene (Phase 5 optional — only if Robin week 2 goes quickly)
- Phase 8b new universal scene (Phase 8a is faster and good enough)

**Do not do:**
- Save format changes (risky without user present)
- Major architectural changes
- UI redesign or layout work
- Anything requiring decisions beyond what is documented here

---

## Definition of Done

- All tests pass
- 0 clippy warnings
- validate-pack is clean
- All scene templates validate with minijinja MCP tool
- World.md has been updated
- Writing guide stale notes removed
- HANDOFF.md reflects current state
- Working tree clean on master
