# Marcus Affair — The Terms Pay Off (Director's Brief / Design)

> **Date:** 2026-06-02
> **Thread:** Marcus affair (workplace, power/cost register)
> **Goal:** Make the `marcus_leverage` three-way fork actually change the back half of the
> affair. This is both *new content* (payoff scenes) and *checks in existing content*
> (re-gating + register variants in the recurring scenes).
> **This document is the brief each `scene-writer` subagent follows.** It carries the
> creative vision; subagents write TOML to spec, the lead (director) wires + validates.

## Context — the hinge that currently goes nowhere

`packs/base/scenes/marcus_leverage.toml` is the affair's pivot. The elevator, Priya from
data, Marcus's hand at the small of her back for two seconds in a four-foot box. It fires
**once** (gate: `MARCUS_INTIMATE && MARCUS_REPEAT_USED_CONF && !MARCUS_LEVERAGE`). Three doors:

| Action | Sets | Mechanical move | Meaning |
|---|---|---|---|
| **Say something** (`push_back`, gated CONFIDENT‖ANALYTICAL) | `MARCUS_TERMS_HERS` | composure +4, stress −3, anxiety −2, liking −1 | She named the terms. He recalibrated — "Fair." Power tilts to her. |
| **Let it go** (`say_nothing`) | `MARCUS_TERMS_HIS` | composure −3, stress +5, anxiety +4, liking +1 | She swallowed it. He keeps the leverage. The asymmetry presses. |
| **End it** (`end_it`) | `MARCUS_AFFAIR_COOLING` | composure +6, stress +4, anxiety −3, liking −3 | She ended it. The affair is over — or cooling. |

**The problem:** nothing reads any of these three flags. All paths lead back to the *same*
recurring office sex (`marcus_repeat_office`, `marcus_pushes`, gated only on `MARCUS_INTIMATE`).
Worse than flat: `MARCUS_AFFAIR_COOLING` means *she ended it*, yet the next weekday the
scheduler still offers office sex as if nothing changed. That is a **continuity break**, not
just a dangling thread. This work closes it.

## What success looks like

- The 3 fork flags (`MARCUS_TERMS_HERS` / `MARCUS_TERMS_HIS` / `MARCUS_AFFAIR_COOLING`) move
  from **dangling → consumed** in `docs/story-map.md`.
- Each door opens onto a materially different back-half: a once-scene that establishes the
  new register, and ongoing recurrence that *reflects* it (or stops, for cooling).
- "I ended it" is honored: recurring Marcus scenes suppress under `AFFAIR_COOLING` unless the
  player relapses.
- Voice and register stay true to `docs/characters/marcus.md` and `docs/writing-guide.md`.

## Register anchors (apply to every scene below)

From `docs/characters/marcus.md`:
- **Economical, professional even when personal.** Direct questions are real questions.
- **He concedes with data, never deference.** "Fair." "Yeah." "No. It's not." Never flirty.
- **Silence with Marcus is charged, not comfortable** (that's Jake).
- **High-consequence register:** career, reputation, the floor, Priya. The risk *is* the charge.
- **He doesn't know about the transformation.** He sees the woman at her desk.
- **Transformation lens (core to HERS):** "She's run this playbook before, in different
  shoes." The former male self knew workplace-power attraction from the *other* side; this
  body adds the layer of being on the receiving end — and, in HERS, of turning it back around.

House rules (from `docs/writing-guide.md`, non-negotiable): **second person present tense**;
explicit by design (never sanitize); **orgasm verb is "cum" not "come"** (climax sense only);
no narrated interiority / no body-as-witness ("the body makes a case", "noted and filed" =
banned); show, don't explain the feeling. Transformation content only inside
`{% if not w.alwaysFemale() %}`.

---

## New scenes (4)

Each new scene: `id = "base::<name>"`, `pack = "base"`, an `[intro]` with FEMININITY- and/or
desire-graded branches, `[[thoughts]]` where useful, `[[actions]]` with `effect` Rhai and
`[[actions.next]] finish = true`. Follow the shape of `marcus_leverage.toml` /
`marcus_repeat_office.toml` exactly.

### 1. `marcus_terms_hers` — "On Her Clock" (once · `TERMS_HERS`)

**Fires once after the player took the HERS door.** The affair continues, but she runs it
now. *She* picks the place and the time, *she* initiates, *she* leaves when she's done. The
competence that read as control in the elevator now serves her — he adjusts, the way he
adjusts to a corrected dataset, because adjusting is what he's good at.

- **Transformation lens, sharpest here** (inside `{% if not w.alwaysFemale() %}`): she knows
  this playbook from the inside — she *ran* it, years ago, in a male body: the assumed access,
  the steer through a doorway, the read of who has less to lose. Now she's running it on him
  and he can't see the seams. Dry amusement under the heat. *Do not* explain this in narrated
  interiority — land it in what she *does* and one or two clean observations, not a thesis.
- **FEMININITY grading:** low-FEM = the strangeness of wielding the very move that was used on
  her; high-FEM = fluent, unremarkable command.
- **Choices:** at least 2 — e.g. *set the place* (her terms made literal: a location/time she
  dictates) and *make him wait* (she controls the tempo). Each is explicit payoff, her in
  charge. One may gate on `CONFIDENT`/`OBJECTIFYING` for a sharper variant.
- **Effects:** set `MARCUS_HERS_ESTABLISHED`; appropriate desire discharge
  (`gd.setDesire(...)` low), `w.changeComposure(+)`, `npc("m").addSexualActivity(...)`,
  relevant `gd.addStat(...)`/`TOTAL_ORGASMS`. Liking neutral-to-slightly-up (he respects it).

### 2. `marcus_terms_his` — "Leverage" (once · `TERMS_HIS`)

**Fires once after the player let it go.** The leverage tightens. He assumes more access,
somewhere riskier than before; the workplace closes in (Priya adjacent — a near-miss, a held
breath, a door that doesn't fully close). The asymmetry she declined to name now names itself
in the prose: what each of them stands to lose is not equal, and he moves like someone who
knows it without ever being crude about it (he is never crude — that's what makes it worse).

- **Composure pressure:** low composure unlocks a more reckless variant (mirror the spiral
  pattern used elsewhere — gate a branch on `w.composure() < N`).
- **Internal sub-choice (the important branch):**
  - *Draw a late line* → sets `MARCUS_HIS_LINE_DRAWN` — a quieter version of the end-it move,
    routes toward cooling (see schedule wiring). She finds the spine she didn't in the elevator.
  - *Sink in* → sets `MARCUS_HIS_ESTABLISHED` — the affair continues on his terms; recurrence
    keeps running but now in the HIS register.
- **Effects:** common spine sets the chosen flag; sink-in path discharges desire and costs
  composure (`w.changeComposure(-)`, `w.changeStress(+)`); line-drawn path recovers some
  composure. Explicit content on the sink-in path; the line-drawn path is a cost/restraint beat.

### 3. `marcus_cooling` — "The Monday After" (once · `AFFAIR_COOLING`)

**Fires once after the player ended it.** The first week back to professional-only. Both
performing normalcy across an open floor: the three-line Slack messages stay three lines, the
"got 10 minutes?" doesn't come, the conference room stays booked-and-empty. The quiet costs
something — restraint is not the same as not wanting. Priya is at her desk; she keeps her head
down; she is very good at her job. (No explicit content here — this is the cost/frost beat.)

- **Internal sub-choice:**
  - *Hold the line* → sets `MARCUS_COOLING_HELD` — the boundary settles. Relief and loss in the
    same breath; composure up, anxiety down.
  - *The door cracks* → sets `MARCUS_COOLING_RELAPSE` — she lets a charged exchange reopen the
    gap. Routes to `marcus_cooling_relapse`.
- **Effects:** set the chosen flag. HELD: `w.changeComposure(+)`, `w.changeAnxiety(-)`.
  RELAPSE: `gd.addDesire(+)`, no discharge (the wanting reopened, unspent).
- **FEMININITY grading:** low-FEM reads the professional frost partly through the old male
  fluency in "keeping it strictly business"; high-FEM reads it as plain self-denial.

### 4. `marcus_cooling_relapse` — "Gravity" (once · `COOLING_RELAPSE`)

**Fires once after the door cracked.** Coming back means coming back on *his* terms — the
leverage was real, the wanting was real, and she's the one who reopened it, which he registers
without gloating (he doesn't gloat; he just resumes). The affair re-establishes in the HIS
register. Explicit. The note is the gravity she didn't beat — not self-punishing, just honest
about the pull.

- **Effects:** set `MARCUS_HIS_ESTABLISHED` and **clear the cooling suppression** by setting
  `MARCUS_RELAPSED` (the schedule re-enables recurrence on this flag — see wiring). Discharge
  desire, cost composure. Liking up slightly (he's pleased in his economical way).

---

## Edits to existing content (the "checks" half)

These are surgical edits to densely-branched files. **The lead (director) makes these, not a
subagent.**

### `marcus_repeat_office.toml`
- **Add a HERS-vs-HIS register layer** to the intro prose (a new `{% if %}` block, layered
  over the existing location/desire branches — do not disturb those):
  - `MARCUS_HERS_ESTABLISHED`: *she* closes the door / sets the place; he follows her lead.
  - `MARCUS_HIS_ESTABLISHED`: he sets it; the asymmetry is present in the framing.
  - neither (pre-leverage): unchanged.
- Schedule gate gains `&& !MARCUS_AFFAIR_COOLING` with a relapse exception (see wiring).

### `marcus_pushes.toml`
- Schedule gate gains the same cooling-suppression-with-relapse exception.
- Light HERS/HIS flavor only if it slots cleanly; do not force it.

### `marcus_leverage.toml`
- No content change. (The `push_back` HERS door already gates on CONFIDENT‖ANALYTICAL — that is
  intentional characterization; a player without those traits cannot seize control, and that is
  correct. Leave it.)

## Schedule wiring (`packs/base/data/schedule.toml`)

Bind the 4 new once-scenes and fix the recurring gates. Cooling-suppression idiom used
throughout: `(!gd.hasGameFlag("MARCUS_AFFAIR_COOLING") || gd.hasGameFlag("MARCUS_RELAPSED"))`.

- `marcus_terms_hers` — trigger: `MARCUS_TERMS_HERS && !MARCUS_HERS_ESTABLISHED`.
- `marcus_terms_his` — trigger: `MARCUS_TERMS_HIS && !MARCUS_HIS_ESTABLISHED && !MARCUS_HIS_LINE_DRAWN`.
- `marcus_cooling` — trigger: `MARCUS_AFFAIR_COOLING && !MARCUS_COOLING_HELD && !MARCUS_COOLING_RELAPSE`
  (also reachable from `MARCUS_HIS_LINE_DRAWN` — decide one entry; simplest is to have the
  line-drawn path set `MARCUS_AFFAIR_COOLING` too, so it converges on this scene).
- `marcus_cooling_relapse` — trigger: `MARCUS_COOLING_RELAPSE && !MARCUS_RELAPSED`.
- `marcus_repeat_office`, `marcus_pushes` — append the cooling-suppression idiom to existing
  `condition`. Recurrence resumes after relapse because `MARCUS_RELAPSED` re-opens the gate.

## Flag map (new)

| Flag | Set by | Read by |
|---|---|---|
| `MARCUS_HERS_ESTABLISHED` | `marcus_terms_hers` | `marcus_repeat_office` (register), schedule (once-guard) |
| `MARCUS_HIS_ESTABLISHED` | `marcus_terms_his` (sink), `marcus_cooling_relapse` | `marcus_repeat_office` (register), schedule |
| `MARCUS_HIS_LINE_DRAWN` | `marcus_terms_his` (line) | schedule (routes to cooling) |
| `MARCUS_COOLING_HELD` | `marcus_cooling` (hold) | schedule (once-guard) |
| `MARCUS_COOLING_RELAPSE` | `marcus_cooling` (crack) | `marcus_cooling_relapse`, schedule |
| `MARCUS_RELAPSED` | `marcus_cooling_relapse` | schedule (re-opens recurrence) |

(The 3 inputs `MARCUS_TERMS_HERS` / `MARCUS_TERMS_HIS` / `MARCUS_AFFAIR_COOLING` were already
set by `marcus_leverage`; this work is their first consumer.)

## Fan-out plan (director + scene-writers)

- **4 parallel `scene-writer` subagents**, one per new scene. Each receives: this brief, the
  target scene's spec section, `docs/characters/marcus.md`, `docs/writing-guide.md`, and
  `marcus_leverage.toml` + `marcus_repeat_office.toml` as voice/structure references. They
  **write the one TOML file only** — no cargo, no git, no edits to other files.
- **Lead (director)** does everything else: the two existing-scene edits, all `schedule.toml`
  wiring, `roadmap.toml` update + `story-map` regen, validation, and commits.
- **Review:** `writing-reviewer` on each of the 4 new scenes → lead applies Critical fixes.
- **Playtest:** `playtester` walks all three branches (HERS, HIS-sink + HIS-line, COOLING-hold +
  COOLING-relapse) via dev-IPC, confirming recurrence reflects/stops correctly.

## Verification gates

1. `cargo run --bin validate-pack` — clean (scene count rises by 4; prose gate active).
2. `cargo test --workspace` — green (esp. reachability / scheduler suites).
3. `cargo run --bin story-map -- --check` after regen — the 3 fork flags no longer dangling.
4. `writing-reviewer` Critical findings resolved.
5. `playtester` PASS — each door demonstrably changes the back-half in-game; cooling stops
   recurrence; relapse re-opens it.
6. `cargo fmt`/scene TOML validated via `mcp__minijinja__jinja_validate_prose` where applicable.

## Scope guard (explicitly out)

Jake / Cal / Theo threads, all other dangling flags, and the opening-callback flags are **out of
scope** this session. This is the Marcus terms-fork vertical slice only.