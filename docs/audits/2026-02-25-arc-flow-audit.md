# Arc Flow & Engine Interaction Audit — 2026-02-25

Complete audit of how arcs, schedule, scenes, and engine interact.

---

## Engine Bugs (Structural — must fix)

✅ **RESOLVED** (writing-pipeline / Sprint 1) — `NpcActionDef` and `NpcAction` now have a `next: Vec<NextBranchDef>` / `next: Vec<NextBranch>` field. The engine evaluates `next` branches after NPC action effects fire. All six scenes with `[[npc_actions.next]]` blocks now route correctly.
### B1. `NpcAction` struct has no `next` field — all `[[npc_actions.next]]` blocks silently dropped

`NpcActionDef` in `types.rs` defines `id`, `condition`, `prose`, `weight`, `effects` — no `next`.
TOML's `[[npc_actions.next]]` entries are silently ignored by serde. Six scene files have NPC actions
with `next` blocks that do nothing:

| Scene | NPC Action | Intended `next` | Actual behavior |
|---|---|---|---|
| `robin_landlord` | `frank_radiator` | `finish = true` | Player still sees action buttons |
| `robin_first_clothes` | `shopper_notices` | `goto = "get_basics"` | No navigation happens |
| `robin_first_day` | `dan_explains` | `finish = true` | Player still sees action buttons |
| `camila_arrival` | `ra_checks_in` | `finish = true` | Player still sees action buttons |
| `camila_orientation` | `adam_talks` | `goto = "engage"` | No navigation happens |
| `camila_library` | `theo_looks_up` | `goto = "keep_working"` | No navigation happens |

The scenes still work (player manually picks the action the NPC was supposed to navigate to),
but intended NPC-driven scene flow is broken.

**Fix:** Either add `next: Vec<NextBranch>` to `NpcActionDef` and wire it in the engine, or
remove the dead `[[npc_actions.next]]` blocks from TOML and redesign those scenes.

✅ **RESOLVED** (Sprint 1) — The caller of `pick_next()` now sets `ONCE_<scene_id>` game flags when `result.once_only == true`, at both call sites in `left_panel.rs:205` and `lib.rs:247`.
### B2. `once_only` flag mechanism is inert — `ONCE_<scene_id>` flags never set

The scheduler checks `world.game_data.has_flag(&format!("ONCE_{}", scene_id))` for once_only events.
But no code in the engine or any scene ever sets these flags. The `once_only` field in schedule.toml
is documentation-only — it has zero runtime effect.

Affected scenes: `robin_first_clothes`, `camila_call_raul`, `camila_library` (all have `once_only = true`).

Currently this doesn't break arcs because `advance_arc` changes state, which changes conditions,
which removes scenes from the eligible pool. But `camila_library` has no arc advancement and no
flag protection — it can fire repeatedly during `dorm_life`.

**Fix:** The caller of `pick_next()` must set the `ONCE_` flag when the scene fires.

✅ **RESOLVED** (playtest-fix session) — `start_scene()` helper in `lib.rs:385` wraps `StartScene` with `SetActiveMale`/`SetActiveFemale` commands before the scene runs. All 5 call sites replaced. NPC liking effects no longer fail silently.
### B3. `add_npc_liking npc = "m"` silently fails — no active NPC set

`coffee_shop` and `rain_shelter` both use `add_npc_liking npc = "m"` effects. This requires
`ctx.active_male` to be set, but no code sets an active male NPC before these scenes run.
`apply_effect` returns `EffectError::NoActiveMale`, which is logged via `eprintln!` but not fatal.
The liking delta is silently lost.

**Fix:** Either wire NPC spawning before these scenes, or remove the `add_npc_liking` effects
until the NPC system is ready for these encounters.

---

## Reachability Gaps

✅ **RESOLVED** (Sprint 1) — `workplace_first_clothes` is now a `trigger` for `arcState == 'week_one'`. `workplace_first_day` fires on `arcState == 'clothes_done'` (a distinct state advanced by first_clothes). Both scenes are reachable in sequence.
### R1. `robin_first_clothes` is unreachable (Critical)

`robin_first_clothes` has `weight = 10`, no trigger, condition `arcState == 'week_one'`.
In the same slot, `robin_first_day` has a **trigger** for the same state. `pick_next()` Pass 1
evaluates triggers first — it finds `robin_first_day`'s trigger and fires immediately. After
that scene fires, arc advances to `working`, making `robin_first_clothes`'s condition false.
The scene will never run.

**Fix:** Either give `robin_first_clothes` a trigger that fires before `robin_first_day`,
or restructure the arc so both scenes can fire during `week_one`.

✅ **RESOLVED** (Sprint 1) — `ArcDef.initial_state` field removed from `data.rs` and all TOML arc definitions. The field no longer exists to be misread or misunderstood by pack authors.
### R2. `initial_state` in `arcs.toml` is dead data

Both arcs declare `initial_state = "arrived"` but the engine never reads this field at game start.
Arc state is `None` until the first `advance_arc` effect fires. The field has no effect.

**Fix:** Either implement auto-initialization from `initial_state`, or remove the field.

---

## Arc Flow — Robin Opening

| Step | Scene | Trigger/Weighted | Key Effects | Advances to |
|---|---|---|---|---|
| 1 | `robin_arrival` | Trigger: `ROUTE_ROBIN` | `set_game_flag: ROUTE_ROBIN` (redundant), `advance_arc: arrived` | `arrived` |
| 2 | `robin_landlord` | Trigger: `ROUTE_ROBIN && !MET_LANDLORD` | `set_game_flag: MET_LANDLORD` | — |
| 3 | `robin_first_night` | Trigger: `ROUTE_ROBIN && arcState == 'arrived'` | `advance_arc: week_one` | `week_one` |
| 4 | ~~`robin_first_clothes`~~ | Weighted (week_one) — **unreachable** | — | — |
| 5 | `robin_first_day` | Trigger: `ROUTE_ROBIN && arcState == 'week_one'` | `set_game_flag: STARTED_JOB`, `advance_arc: working` | `working` |
| 6 | `robin_work_meeting` | Trigger: `ROUTE_ROBIN && arcState == 'working' && !FIRST_MEETING_DONE` | `set_game_flag: FIRST_MEETING_DONE` | — |
| 7 | `robin_evening` | Trigger: `ROUTE_ROBIN && arcState == 'working' && FIRST_MEETING_DONE` | `advance_arc: settled` | `settled` |

**Terminus: `settled`** — No schedule events gate on this state. Robin arc complete.
No Robin-specific `intro_variants` in universal scenes post-settled (despite arc doc suggesting this).

### Note: `robin_landlord` trigger doesn't check arc state

Trigger is `gd.hasGameFlag('ROUTE_ROBIN') && !gd.hasGameFlag('MET_LANDLORD')`. If `MET_LANDLORD`
were ever removed, `robin_landlord` could fire again after arc has advanced. Low risk but inconsistent
with other arc scenes that check arc state.

---

## Arc Flow — Camila Opening

| Step | Scene | Trigger/Weighted | Key Effects | Advances to |
|---|---|---|---|---|
| 1 | `camila_arrival` | Trigger: `ROUTE_CAMILA` | `set_game_flag: ROUTE_CAMILA` (redundant), `advance_arc: arrived` | `arrived` |
| 2 | `camila_dorm` | Trigger: `ROUTE_CAMILA && arcState == 'arrived'` | `advance_arc: orientation` | `orientation` |
| 3 | `camila_orientation` | Trigger: `ROUTE_CAMILA && arcState == 'orientation'` | `advance_arc: dorm_life` | `dorm_life` |
| 4a | `camila_library` | Weighted: 10 (dorm_life) | No flags, no arc advancement | — |
| 4b | `camila_call_raul` | Weighted: 8 (dorm_life, `!CALL_HOME_DONE`) | `set_game_flag: CALL_HOME_DONE` | — |
| 4c | `camila_study_session` | Weighted: 10 (dorm_life, `!STUDY_SESSION_DONE`) | `set_game_flag: STUDY_SESSION_DONE` | — |
| 5 | `camila_dining_hall` | Trigger: `dorm_life && STUDY_SESSION_DONE` | `advance_arc: first_week` | `first_week` |

**Terminus: `first_week`** — No schedule events gate on this state. Camila arc complete.

### Note: `camila_library` can fire repeatedly

No `once_only` protection (mechanism is inert anyway), no flag set, no arc advancement.
Will keep appearing in weighted pool every turn during `dorm_life`.

---

## Game Flags — Set But Never Read

| Flag | Set by | Read by |
|---|---|---|
| `STARTED_JOB` | `robin_first_day` | Nothing |
| `MET_JAKE` | `coffee_shop` | Nothing |
| `COFFEE_SHOP_VISITED` | `coffee_shop` | Nothing |
| `RAIN_SHELTER_MET` | `rain_shelter` | Nothing |

These are presumably hooks for future content but currently have no mechanical effect.

---

## Character Roster — Named NPCs without docs

Characters with full character docs (`docs/characters/`): **Robin**, **Camila** (both are PCs, not NPCs).

Named characters appearing in scene prose with **no character doc**:

| Name | Appears in | Role | Notes |
|---|---|---|---|
| Marcus | `robin_arrival`, `robin_work_meeting` | Two different people (friend + coworker) | **Name collision** — same name for unrelated characters |
| Frank | `robin_landlord` | Landlord | Named in NPC action |
| Dan | `robin_first_day` | Backend team coworker | |
| Kevin Marsh | `robin_work_meeting` | Platform engineer | |
| Carter | `robin_work_meeting` | Product manager | |
| Devin | `robin_work_meeting` | Platform engineer | |
| David | `rain_shelter` | Stranger sharing shelter | Sets `RAIN_SHELTER_MET` flag |
| Jake | `coffee_shop` | Coffee shop encounter | Sets `MET_JAKE`, `add_npc_liking` |
| Janette | `camila_dorm`, `camila_arrival` | Roommate | |
| Priya | `camila_arrival` | RA (Resident Advisor) | |
| Diego | `camila_call_raul` | Friend from home | |
| Adam | `camila_orientation` | Freshman at icebreaker | |
| Theo | `camila_library` | Library study partner | |

---

## Unused Definitions

### Traits defined but unused in scene content:
- `BLOCK_ROUGH`, `LIKES_ROUGH` — hidden, no rough content exists yet
- `NOT_TRANSFORMED` — injected but never checked
- `TRANS_WOMAN` — deprioritized

### Skills defined but unused in scene content (8 of 9):
- `FITNESS`, `CHARM`, `FASHION`, `DANCE`, `COOKING`, `ADMIN`, `MANAGEMENT`, `CHILDCARE`
- Only `FEMININITY` is used

### Stats defined but unused (all 3):
- `TIMES_KISSED`, `DATES_ATTENDED`, `WEEKS_WORKED`

### Traits with minimal usage (1–2 scenes only):
- `REFINED` (1), `ROMANTIC` (1), `OUTGOING` (1), `CONFIDENT` (1), `SEXIST` (1),
  `HOMOPHOBIC` (1), `OBJECTIFYING` (1), `OVERACTIVE_IMAGINATION` (1)
