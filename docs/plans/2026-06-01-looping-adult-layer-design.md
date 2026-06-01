# Looping Adult Layer — Design Spec

> **Status:** Approved 2026-06-01. Executable plan — drives a prolific multi-agent
> content session (engine first, then parallel scene-writer fan-out).
> **Author:** brainstormed with user, 2026-06-01.
> **Source-of-truth order:** live code > CLAUDE.md > creative-direction.md > this file.

---

## The Problem

The opening arcs (workplace / campus) are deep and well-built. The **adult** content is not —
it is almost entirely **terminating one-shots**:

- **Jake** chain: first date → second → apartment → morning-after → stays-over, then nothing
  but repeatable *text messages*.
- **Marcus** chain: coffee → favor → late → drinks → closet → aftermath → apartment, then ends.
- **Bar stranger, party stranger, Theo:** all `once_only`. Fire once, gone forever.

Once a player exhausts those threads the adult side is **dry** — the free-play loop is grocery
store, park, bookstore, laundromat. There is no *repeatable* sexual content, no ongoing sexual
relationship with variety, nothing that keeps "the world happening to her" — which
`creative-direction.md` names as the **core erotic logic** (loss of control; the world exceeds
her choices).

Meanwhile `stats.toml` / `skills.toml` define a *huge* sexual vocabulary — RIDING, DEEPTHROAT,
DIRTY_TALK, TEASE, ANAL_SKILL, BONDAGE_SKILL, PAIN_TOLERANCE, COMPOSURE, plus consequence stats
(TIMES_PUBLIC_SEX, TIMES_CUMMED_IN, DUBCON, …). **Almost none is exercised by current scenes.**
The deep system exists; content never developed or branched on it. That is *why* it would get
boring — sex scenes don't progress, don't escalate, don't develop skill.

**This session does not "write more sex scenes." It builds the engine that makes desire recur
and escalate, then fans out scenes onto it.**

---

## Locked Decisions (from brainstorming)

| Decision | Choice |
|---|---|
| Looping pillars | Ongoing-partner sex · The world initiates · Kink escalation system |
| Desire mechanic | **Yes** — build a desire need-state that drives scheduling |
| Desire stakes | **Giving in lowers COMPOSURE** (downward spiral of loss-of-control) |
| Escalation model | **Mixed by partner** — Jake: she seeks; Marcus + gym regular: done to her |
| Registers to push | **Submission / being used** · **Power inversion** (she ran this script once) |
| Scope | Robin / workplace focus + **one new recurring partner** (the gym regular) |
| Sprint size | Big swing (~13 scenes) |

Registers **not** chosen (deprioritized this sprint, easy to add later): public/exhibition,
anonymous/degradation, photos. Escalation stays in the submission lane.

---

## Part 1 — The DESIRE / COMPOSURE Engine

The spine that turns "the body happens to her" into a *system*. Integration points were mapped
against live code; file:line anchors below are the implementation targets.

### 1.1 DESIRE need-state

- New field on `GameData` (`crates/undone-world/src/game_data.rs`, struct at ~`:5-24`).
  Representation: **`BoundedStat` 0–100** (matches `stress` / `anxiety`), `#[serde(default)]`.
- **Accumulation:** in `GameData::advance_time_slot()` (~`:56`), desire rises each consumed
  time-slot with no release. Base **+8 / slot**, scaled by composure (see 1.2). Clamped 0–100.
- **Discharge:** orgasm/release scene effects set or subtract desire (a satisfied scene drops it
  toward 0; a teasing/denied scene leaves it high or *raises* it).

### 1.2 COMPOSURE activation

COMPOSURE already exists as a dormant skill (`skills.toml:327`, 0–100, **unused anywhere** —
confirmed by grep). Activate it:

- **Starting value:** seed COMPOSURE to **60** at game start (new-game init, alongside FEMININITY).
- **Giving in lowers it:** acting on desire (the submissive/reckless choices) applies a COMPOSURE
  decrease via scene effects.
- **The spiral:** desire accumulation rate scales inversely with composure —
  `+8` per slot at composure 60, climbing toward `+14` as composure falls below ~30. Low composure
  → desire builds faster → more adult scenes surface → more chances to give in → composure falls
  further. Resisting holds composure but desire keeps climbing and surfacing scenes anyway.
- COMPOSURE **slowly recovers** on idle, non-charged slots (+2/slot) so the spiral is a pull, not
  a death-march.

### 1.3 Desire → scheduler bias (data-driven, principle-compliant)

The engine must **never learn what "adult" means** (Engineering Principle 2). So:

- Add an optional per-event field **`desire_scaled = true`** to `ScheduleEventToml`
  (`scheduler.rs:50-62`) and `ScheduleEvent` (`:72-79`).
- In `pick_weighted_candidate()` (`:357-378`), an event with `desire_scaled = true` has its
  effective weight computed as `weight * desire_multiplier(current_desire)` where the multiplier
  ramps ~`1.0×` at desire 0 to ~`4.0×` at desire 100. Plain math; the *pack data* opts a scene in.
- Scenes may **also** gate desire-tier variants via `condition` / `trigger` using `gd.desire()`
  (e.g. a high-desire ambush only eligible at `gd.desire() >= 60`).

### 1.4 Rhai accessors (the API scene-writers will use)

Read API (`script/read_api/game_data.rs`, register at ~`:130-148`):

| Accessor | Returns |
|---|---|
| `gd.desire()` | i64, 0–100 |
| `w.composure()` | i64, 0–100 (thin wrapper over the COMPOSURE skill) |

Write API (`script/write_api/game_data.rs`, register at ~`:80-90`):

| Effect call | Meaning |
|---|---|
| `gd.addDesire(n)` | add/subtract desire (clamped 0–100) |
| `gd.setDesire(n)` | set desire exactly (use `gd.setDesire(0)` after full release) |
| `w.changeComposure(n)` | add/subtract composure (clamped 0–100). **Giving in = negative.** |

Existing accessors scene-writers already use (confirmed live): `gd.setGameFlag("X")`,
`gd.hasGameFlag("X")`, `w.skillIncrease("FEMININITY", n)`, `w.changeStress(n)`,
`w.addArousal(n)`, `w.getSkill("ID")`, `w.hasTrait("ID")`, `gd.addStat("STAT", n)`,
`w.skillIncrease("<sexual skill>", n)`. (`add_arousal`/skill names per `content-schema.md` +
the live Rhai write layer.)

### 1.5 Save + UI

- **Save:** bump `SAVE_VERSION` 6 → 7 (`undone-save/src/lib.rs:9`); no-op `migrate_v6_to_v7`
  (serde default fills `desire`); chain it in the load path (`:144-169`).
- **UI:** render **Desire** and **Composure** meters in the stats sidebar
  (`crates/undone-ui/src/` sidebar/stat-panel + `runtime_snapshot.rs` serialization). Desire as a
  warm bar, composure as a cool bar. Both visible from the start so the player feels the system.

### 1.6 Engineering tasks (TDD — tests before content uses any of it)

1. `GameData.desire: BoundedStat` + accumulation in `advance_time_slot` (composure-scaled). Unit
   tests: idle slots raise desire; rate rises as composure falls; clamps at 100.
2. COMPOSURE seed at new-game (60); `w.changeComposure` clamps. Unit test.
3. Rhai read/write accessors registered; round-trip test (`addDesire` then `gd.desire()` reads it;
   `changeComposure(-10)` reflected in `w.composure()`).
4. `desire_scaled` field parses from schedule TOML; `pick_weighted_candidate` scales weight; unit
   test proves a `desire_scaled` event out-competes a fixed-weight peer at high desire and not at
   low desire.
5. Save v7 migration round-trips a v6 save (desire defaults to 0). Test.
6. UI sidebar meters + snapshot fields; runtime snapshot test asserts desire/composure surface.
7. **Doc fix (Principle 10):** update `content-schema.md` to the Rhai `effect = '...'` API and add
   `gd.desire()` / `w.composure()` / `gd.addDesire` / `w.changeComposure` / `desire_scaled` to the
   accessor + schedule references; note in `engine-design.md`.

Engine lands and is **fully green** before any scene-writer is dispatched.

---

## Part 2 — Kink Escalation Model

Per-partner **act-unlock flags** (persistent game flags). New acts open as a thread deepens; the
*dynamic* differs by partner (the "mixed" decision):

- **Jake — she seeks (agency / safe).** Within the established relationship, *she* initiates the
  new act when desire is high / composure is low enough to ask. Player-chosen. Unlock flags like
  `JAKE_ACT_RIDE`, `JAKE_ACT_DIRTY_TALK`, `JAKE_ACT_TIED` set by her choice. Develops the relevant
  sexual skill (`RIDING`, `DIRTY_TALK`, `BONDAGE_SKILL`).
- **Marcus & the gym regular — done to her (submission / loss-of-control).** *He* leads. The
  escalation happens before she decides; the scene presents resist-vs-give-in. **Giving in lowers
  COMPOSURE** and sets the act-unlock flag (`MARCUS_ACT_USED`, `GYM_ACT_*`). Resisting holds the
  line but desire stays high and the thread re-offers later.

Escalation gates on a mix of: thread depth (prior flags), `gd.desire()`, `w.composure()`, and the
relevant sexual skill. Acts stay in the **submission** register (being used, being told, being
tied, giving over) — not public/photos this sprint.

---

## Part 3 — Scene Manifest (~13)

Each brief gives a scene-writer enough to write **without inventing creative direction**: the
inciting situation (world acts first), the 2–4 traits that matter, the transformation/register
angle, the escalation dynamic, repeatable-or-once, and the flags/stats/desire effects to fire.
All prose: **second-person present, DM-narrator voice, transformation written directly** (no
`alwaysFemale` guards except around before-body accessors). Writers follow `docs/writing-guide.md`.

> **Distinctness mandate (creative-direction §2):** no two scenes may use the same transformation
> device or the same beat. Each brief names a different angle. Reviewers enforce this.

### Thread A — Jake (ongoing partner · she seeks · romantic-but-filthy)

**A1 · `jake_repeat_night` — repeatable** (`desire_scaled`). The first *repeatable* Jake sex.
Inciting: an ordinary evening at his or her place tips over. Must **vary every fire** — branch the
structure on `gd.desire()` (urgent vs slow), `w.composure()` (who reaches first), and which of her
arousal-response traits dominate (HAIR_TRIGGER / EASILY_WET / SENSITIVE_NECK → structurally
different events, not adjectives). Power-inversion thread present but quiet: this is the safe one,
where she has the most agency. Effects: `gd.addStat("TIMES_HAD_SEX",1)`, orgasm stats,
`gd.setDesire(0)` on satisfying release (or leaves a remainder if denied), small `RIDING`/`KISSING`
skill gains. Develops the relationship without terminating.

**A2 · `jake_morning_quick` — repeatable** (`desire_scaled`, weekend/morning-gated). The quickie
variant — different rhythm, different constraints (time, half-asleep, has-to-leave-for-work). Body
that already knows his. Distinct device: *familiarity* (the opposite of the gym regular's novelty).

**A3 · `jake_seeks_more` — escalation, once per act.** Desire/composure high enough that *she*
asks for the new thing. Player picks which act to initiate (gated on built skill) → unlock flag +
skill gain. The agency pole of the escalation model. She used to be the one being asked — now she's
asking; note that inversion without narrating it.

### Thread B — Marcus (ongoing affair · done to her · office / power / submission)

**B1 · `marcus_repeat_office` — repeatable** (`desire_scaled`, weekday/work-slot, secrecy-gated).
The affair as a recurring charged encounter — stolen, secret, the door, the listening. Submission
register: he sets the terms. Branch on composure (how much she resists the pull) and desire.
Distinct device: *secrecy and the professional mask cracking* (COMPOSURE made literal — she's
maintaining a facade at work). Giving in nudges `w.changeComposure(-n)`.

**B2 · `marcus_pushes` — escalation, done to her.** He introduces an act she didn't initiate.
Resist-vs-give-in. Give in → `w.changeComposure(-n)`, unlock flag, `gd.addStat` consequence, the
submission lands hard. Resist → holds, thread re-offers. The power-inversion is sharpest here: she
recognizes the exact move because she *made* it once — show it through her body answering anyway,
never through narration.

**B3 · `marcus_leverage` — the affair has a cost.** Not explicit-first; the complication. The
secret has weight — a near-miss at work, his assumption of access, the asymmetry of what each
stands to lose. Consequence scene that makes the repeatable affair *mean* something. Branches on
trait (CONFIDENT pushes back; SHY absorbs) and prior flags. May set a flag that gates whether B1
stays available or sours.

### Thread C — The Gym Regular (NEW recurring partner · done to her · power inversion + submission)

See Part 4 for the NPC profile. Role tag `ROLE_GYM`, display name **Cal** (provisional).

**C1 · `gym_regular_intro` — once.** Establish him. She's a gym regular now (ties to existing
`gym_changing_room`). He's there the same hours, takes up space the way she used to. He notices her
early; doesn't perform about it. The whole scene is the *inversion*: she ran this patient-watcher
script on women once and knows every beat of it — and her body responds to it being run on her.
Sets `MET_GYM_REGULAR` + `set_npc_role ROLE_GYM` + `set_npc_name "Cal"`. No explicit content yet —
this is the hook (the draw that makes her come back).

**C2 · `gym_regular_recurs` — repeatable** (`desire_scaled`). He's there again. Escalating,
patient attention across visits — a spotter's hand, a conversation that goes a beat too long, the
charge building. She keeps coming back (loss-of-control: she tells herself it's the workout). Each
fire ratchets a tension counter / liking. Distinct device: *being watched and drawn back*.

**C3 · `gym_regular_first` — trigger, once.** It tips physical, on his terms (done to her). The
first explicit encounter — submission register, the thing she set up by returning. Resist-vs-carried
structure. Sets `GYM_INTIMATE`, sex/orgasm stats, `UNIQUE_PARTNERS`, `w.changeComposure(-n)`.

**C4 · `gym_regular_deepens` — repeatable** (`desire_scaled`, post-`GYM_INTIMATE`). The ongoing
thing with him — recurring, escalating acts (done-to-her unlocks). The anonymous-vector engine
realized: he is the world that keeps initiating. Varies by desire/composure/which act is unlocked.

### Thread D — Desire engine support (the body's loop)

**D1 · `desire_solo_night` — repeatable** (`desire_scaled`, high-desire gated, fires when no
partner available). The release valve. Alone, the body demanding, desire too high to ignore.
Submission-inflected (she gives in to her own body). Branch on arousal-response traits + composure.
Effects: `gd.addStat("TIMES_MASTURBATED",1)`, `gd.setDesire(low)`, small `w.changeComposure(-n)`
(giving in, even alone). The mechanical floor that keeps desire from one-way ratcheting.

**D2 · `desire_ambush` — repeatable** (`desire_scaled`, high-desire gated, fires inside mundane
slots — commute, desk, errand). The body betraying her mid-day. No partner, no release — desire
surfacing where she can't act on it. Pure power-inversion texture: she used to *have* a body that
didn't do this to her. Distinct device: *wrong-place wanting*. Leaves desire high (no discharge),
maybe `w.changeComposure(-small)` from the effort of holding the mask. Trains the player that the
meter is real and has teeth.

### Manifest summary

| # | Scene | Thread | Repeatable | `desire_scaled` | Escalation | Key register |
|---|---|---|---|---|---|---|
| A1 | `jake_repeat_night` | Jake | ✓ | ✓ | — | romantic/agency |
| A2 | `jake_morning_quick` | Jake | ✓ | ✓ | — | familiarity |
| A3 | `jake_seeks_more` | Jake | per-act | — | she seeks | agency / inversion |
| B1 | `marcus_repeat_office` | Marcus | ✓ | ✓ | — | secrecy / submission |
| B2 | `marcus_pushes` | Marcus | per-act | — | done to her | submission / inversion |
| B3 | `marcus_leverage` | Marcus | once | — | — | consequence / power |
| C1 | `gym_regular_intro` | Gym | once | — | — | inversion (hook) |
| C2 | `gym_regular_recurs` | Gym | ✓ | ✓ | — | drawn back |
| C3 | `gym_regular_first` | Gym | once (trigger) | — | done to her | submission |
| C4 | `gym_regular_deepens` | Gym | ✓ | ✓ | done to her | the world initiates |
| D1 | `desire_solo_night` | Desire | ✓ | ✓ | — | giving in (release valve) |
| D2 | `desire_ambush` | Desire | ✓ | ✓ | — | power-inversion texture |

That's **12 scenes** + the engine. (A1 split or a 13th Marcus/gym variant can be added if fan-out
has headroom.)

---

## Part 4 — The New NPC: the Gym Regular ("Cal")

Provisional profile — `docs/characters/cal.md` to be written alongside the scenes. Name is a
display-name override; trivially renamed.

- **Who:** late 30s / early 40s. A *regular*, not a trainer (trainer is service-coded; this must
  be a man who is simply *there*, on his own terms). At the gym the same hours she is.
- **Register:** patient. He does not chase. He takes up space and waits — exactly the unhurried,
  entitled watcher she used to be. He notices her early and is unbothered about being caught.
- **The thread's engine:** loss-of-control is that *she keeps coming back* and tells herself it's
  the workout. When it tips physical it's **on his terms** (done-to-her). Power inversion is the
  spine: she ran this precise script on women in gyms once; now she's inside it and her body
  answers the script faster than her judgment can object — **shown through the body, never
  narrated** (creative-direction §5, "Show, then trust").
- **Contrast with Jake & Marcus:** Jake = safe, chosen, she leads. Marcus = secret, transactional,
  workplace stakes. Cal = novelty, patience, pure body — the anonymous-but-recurring vector.

---

## Part 5 — Execution Plan (director-driven fan-out)

Phased so the engine lands before content depends on its accessors. Lead (Opus) directs, verifies
between phases, synthesizes. **Project rules:** parallel scene-writers edit **TOML only** — they do
not run cargo/git/validate; the lead runs the single verification pass. Background/parallel agents
get precise scoped prompts.

### Phase 0 — Setup ✅ in progress
- Commit prior verified work (clean base). Write + commit this spec.
- Worktree per project convention: `~/.config/ops/worktrees/undone/looping-adult/`.

### Phase 1 — Engine (lead, TDD, sequential)
- Implement Part 1 §1.6 tasks 1–7 with tests-first. `cargo fmt` + `cargo check -p <crate>` after
  each file; `cargo test` green across `undone-world`, `undone-save`, `undone-scene`, `undone-ui`.
- **Gate:** full workspace `cargo test` green + `validate-pack` clean before Phase 2.

### Phase 2 — Content fan-out (parallel `scene-writer` agents)
- One agent per scene (~12), dispatched in parallel batches (~4 at a time to respect machine
  limits). Each gets: its brief (Part 3), the new accessor list (§1.4), the NPC profile (Part 4),
  `writing-guide.md`, and the relevant character doc. Writers validate templates via the minijinja
  MCP and **write TOML only**.
- Lead reviews each returned scene against its brief for drift / invented direction.

### Phase 3 — Review (parallel `writing-reviewer` agents)
- One reviewer per scene. Enforce: voice, distinctness mandate, no narrated transformation,
  fragments-not-thoughts, register fidelity, repetition ban. Lead applies **all Critical** fixes.

### Phase 4 — Wire + verify
- Lead wires `schedule.toml`: new events with correct `condition`/`trigger`/`weight`/`once_only`/
  `desire_scaled`/`npc_role`; repeatable scenes in the weighted pool, escalation as trigger-gated
  beats. Seed COMPOSURE at new-game.
- `cargo run --bin validate-pack` clean; full `cargo test` green.
- **`playtester`** plays the loop end-to-end and reports whether it is actually *hot, varied, and
  non-repetitive* — and whether desire/composure visibly drive the experience. Treat as player
  feedback, not a checklist.

### Phase 5 — Finish
- Update HANDOFF.md (Current State + Session Log). Update `content-schema.md` + `engine-design.md`.
- Feed any new anti-patterns back into `writing-guide.md` + agent checklists (per project rule).
- Merge the branch (project override: always merge).

---

## Verification Gates (hard)

- Engine: unit tests for desire accumulation, composure spiral, scheduler bias, save v7, snapshot.
- Content: every scene passes minijinja validation; `validate-pack` clean (no new reachability or
  prose-audit regressions beyond known creative-gated warnings).
- Runtime: playtester confirms (a) repeatable scenes re-fire and vary, (b) desire rises on idle and
  biases the pool, (c) giving in drops composure and the spiral is felt, (d) escalation unlocks
  fire on their gates, (e) no scene reads as filler or a twin of another.

---

## Open Items / Risks

- **Desire tuning** (+8/slot, multiplier ramp, composure coupling) are first-pass numbers; the
  playtester pass calibrates. Keep constants in one place for easy tuning.
- **Repeatability vs. staleness:** a `desire_scaled` repeatable scene that doesn't *structurally*
  branch will feel like filler fast. The distinctness mandate applies *within* a scene's variants,
  not just across scenes — A1/A2/B1/C2/C4/D1/D2 each need genuine structural variation by
  desire/composure/trait, or they fail their purpose.
- **Cal naming** provisional — confirm or rename at spec review (cheap: display-name override).
- **Scope realism:** 12 scenes + engine + review + wire + playtest is a large session. Incremental
  commits in the worktree protect progress; if budget runs short, the engine + one fully-realized
  thread (Jake or Cal) is the minimum shippable proof-of-pattern.
