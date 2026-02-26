# Sprint 2 Implementation Plan — "FEMININITY Moves"

**Created:** 2026-02-26
**Branch:** `sprint2-femininity-moves`
**Goal:** FEMININITY reaches 25+ naturally by workplace arc end. `plan_your_day` is a
real scene. Coffee_shop over-naming removed.

---

## Design: FEMININITY Progression Curve

Starting value for `CisMaleTransformed`: **10**
Target at arc completion: **25+**

Each workplace scene represents a discrete adaptation moment. The amount granted
reflects the weight of the experience — how much genuine learning about being female
in the world the scene contains.

| Scene | Stage → | FEMININITY +gained | Running total (from 10) |
|---|---|---|---|
| `workplace_arrival` | → arrived | +2 | 12 |
| `workplace_landlord` | arrived | +2 | 14 |
| `workplace_first_night` | arrived | +2 | 16 |
| `workplace_first_clothes` | → clothes_done | +5 | 21 |
| `workplace_first_day` | → working | +3 | 24 |
| `workplace_work_meeting` | working | +3 | 27 ✅ |
| `workplace_evening` | → settled | +3 | 30 |

**Rationale per scene:**
- **+2 (arrival)**: First hours. The subway gaze, the checkpoint. Disorienting but not transformative.
- **+2 (landlord)**: Managing the ID confrontation. Successfully navigating a public test.
- **+2 (first night)**: Discovering the bra problem. The logistics of being female become concrete.
- **+5 (first clothes)**: Bra fitting, dressing room mirror. The body becomes real and navigable. Highest growth moment.
- **+3 (first day)**: A full working day as a woman — being read as junior, managing the gap.
- **+3 (work meeting)**: Presenting design under gendered scrutiny. Kevin explaining her own work.
- **+3 (evening)**: The light switch. The quiet. Adaptation becoming habit.

FEMININITY-gated branches activate at 15, 20, 25. After arc: value at 30 means week-2 content
(< 25 branches) no longer fires. Correct — the player has adapted.

---

## Tasks

### Batch 1: TDD + FEMININITY increments (Tasks 2.1, 2.2, 2.5)

**Step 1.1** — Write failing test `femininity_reaches_25_by_arc_end`
- File: `crates/undone-scene/src/lib.rs`
- Reuse `workplace_arc_full_playthrough` test structure
- After arc reaches `settled`, assert FEMININITY skill value >= 25
- Must **fail** before Step 1.3 (FEMININITY stays at 10 currently)

**Step 1.2** — Verify test fails: `cargo test femininity_reaches_25 -p undone-scene`

**Step 1.3** — Add `skill_increase FEMININITY` effects to workplace scenes.
Effect TOML:
```toml
[[actions.effects]]
type   = "skill_increase"
skill  = "FEMININITY"
amount = N
```
Scene-by-scene assignments (add to ALL terminating action paths):
- `workplace_arrival`: `take_subway` +2, `call_a_cab` +2
- `workplace_landlord`: `wait_him_out` +2, `explain_briefly` +2, `frank_radiator` NPC action +2
- `workplace_first_night`: `order_food_sleep` +2, `research_bra_situation` +2, `call_someone` +2
- `workplace_first_clothes`: `get_basics` +5, `dwell_on_mirror` +5
- `workplace_first_day`: `meet_the_team` +3, `prove_it_now` +3, `dan_explains` NPC action +3
- `workplace_work_meeting`: `present` +3
- `workplace_evening`: `moment` +3

**Step 1.4** — Verify test passes: `cargo test femininity_reaches_25 -p undone-scene`

**Step 1.5** — `cargo test --workspace && cargo clippy && validate-pack`

---

### Batch 2: Content fixes (Tasks 2.3, 2.4)

**Step 2.1** — Coffee_shop prose fix
- File: `packs/base/scenes/coffee_shop.toml`
- Remove over-naming: "There's a geometry to being a woman in a line"
- Rewrite: show the concrete physical awareness without naming it as a category
- Target: the spatial self-awareness (bag against hip, shoulders in, the three inches of
  air) is shown directly. "He didn't notice any of that" stays but loses the editorial
  framing that follows.

**Step 2.2** — `plan_your_day` full rewrite via `scene-writer` agent
- Current stub: 2 bare actions, minimal prose
- Target: real hub scene with 4 choices, FEMININITY-appropriate branches at < 20 / 20-40 / 40+
- Design spec for the agent:
  - Time-slot appropriate intro (morning / afternoon / evening)
  - Low FEMININITY (< 20): The day still feels like a problem set. The list is long.
    Everything is still slightly new — the coffee order, the commute, the body in the mirror.
  - Mid FEMININITY (20-40): Habits are forming. Not automatic, but repeatable.
    There are small moments that feel like *hers* now.
  - Choices:
    1. "Go out" → `free_time` slot (explore the city)
    2. "Run errands" → `free_time` slot, small money effect
    3. "Work from home" → finish, stress effect if work started
    4. "Rest" → finish, stress -2, anxiety -1
  - `{% if not w.alwaysFemale() %}` inner voice block reflecting on the day's domestic
    texture — something small that's become normal

**Step 2.3** — `writing-reviewer` agent: audit plan_your_day output

**Step 2.4** — Apply Critical fixes to plan_your_day

**Step 2.5** — Validate: `mcp__minijinja__jinja_validate_template` on plan_your_day.toml

**Step 2.6** — Final pass: `cargo test --workspace && cargo clippy && validate-pack`

---

## Done Criteria

- [ ] `femininity_reaches_25_by_arc_end` test passes
- [ ] FEMININITY starts at 10, reaches 30 naturally after all 7 workplace arc scenes
- [ ] `plan_your_day` passes writing-reviewer with zero Criticals
- [ ] Coffee_shop "geometry" over-naming removed
- [ ] All tests pass, clippy clean, validate-pack clean
