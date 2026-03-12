# Arc: base::workplace_opening

## Narrative Purpose

Establish the player character in the city as a working professional. Ground the
transformation in specific physical and social experiences. Set the tone for this
arc: pragmatic, wry, quietly overwhelmed. Introduce the city as a place with its
own life, indifferent to her arrival.

## Who This Arc Fires For

Any player character carrying `ROUTE_WORKPLACE`. The Workplace preset (Robin) is the
first character configured with this flag, but a custom character who receives it
through other means gets the same arc. Scene content branches on traits, not on which
preset started the game.

## State Machine

```
arrived → week_one → working → settled
```

**arrived** (Saturday, day 0–1)
Scenes: `workplace_arrival`, `workplace_landlord`, `workplace_first_night`

**week_one** (Sunday before first workday)
Scenes: `workplace_first_clothes`, plus universal slot scenes as available

**working** (Monday onward, has started job)
Scenes: `workplace_first_day`, universal work-adjacent scenes

**settled** (after ~week 2, has basic routines)
Universal scenes fire normally. Workplace-specific flavor via `[[intro_variants]]`.

## Scene List

| Scene ID | Arc state gate | Location | Content | Sets |
|---|---|---|---|---|
| `base::workplace_arrival` | none (once-only, ROUTE_WORKPLACE) | Airport → subway | VANILLA | `ROUTE_WORKPLACE`, arc→arrived, first scrutiny flags |
| `base::workplace_landlord` | arrived | Her apartment building | VANILLA | `MET_LANDLORD`, landlord handling flags |
| `base::workplace_first_night` | arrived | Her apartment | VANILLA | arc→week_one, first-night coping flags |
| `base::workplace_first_clothes` | week_one | Clothing store | VANILLA | first-clothes self-management flags |
| `base::workplace_first_day` | week_one | Tech office | VANILLA | arc→working, `STARTED_JOB`, status/lunch flags |
| `base::workplace_work_meeting` | working | Tech office — meeting room | VANILLA | `FIRST_MEETING_DONE` |
| `base::workplace_evening` | working | Her apartment (evening) | VANILLA | arc→settled |
| `base::opening_callback_status_assertion` | settled, week ≥ 2 | Workday follow-up | VANILLA | `OPENING_CALLBACK_STATUS_ASSERTION` |
| `base::opening_callback_mirror_afterglow` | settled, week ≥ 2 | Home / before commute | VANILLA | `OPENING_CALLBACK_MIRROR_AFTERGLOW` |
| `base::opening_callback_first_week_solitude` | settled, week ≥ 2 | Apartment | VANILLA | `OPENING_CALLBACK_FIRST_WEEK_SOLITUDE` |
| `base::opening_callback_transactional_defense` | dormant hook | City errand | VANILLA | `OPENING_CALLBACK_TRANSACTIONAL_DEFENSE` |

## Week-One Memory Contract

These flags are the first persistent memory layer for the workplace route. They are
not flavor-only. They exist so later prose, route gating, and erotic escalation can
remember *how* the player stabilized rather than only that the arc was completed.

### Arrival and scrutiny

| Flag | Source | Meaning | Status |
|---|---|---|---|
| `OPENING_ID_PREEMPTED` | `workplace_arrival:id_preempt` | She learned to get ahead of scrutiny before it formed into a question. | reserved callback |
| `OPENING_ID_WAITED_OUT` | `workplace_arrival:id_wait` | She tolerated scrutiny and let the room arrive at the answer slowly. | reserved callback |

### Landlord handling

| Flag | Source | Meaning | Status |
|---|---|---|---|
| `LANDLORD_WAITED_HIM_OUT` | `workplace_landlord:wait_him_out` | She absorbed social friction by outlasting it. | reserved callback |
| `LANDLORD_EXPLAINED_BRIEFLY` | `workplace_landlord:explain_briefly` | She used a small socially smoothing explanation to keep the interaction moving. | reserved callback |
| `LANDLORD_KEPT_TRANSACTIONAL` | `workplace_landlord:keep_it_transactional` | She pushed the moment back onto paperwork and logistics. | dormant callback authored |

### First-night coping

| Flag | Source | Meaning | Status |
|---|---|---|---|
| `FIRST_NIGHT_CRASHED` | `workplace_first_night:order_food_sleep` | She survived by collapsing the problem into food and sleep. | used now, active callback |
| `FIRST_NIGHT_RESEARCHED` | `workplace_first_night:research_bra_situation` | She converted overwhelm into information gathering. | used now, active callback |
| `FIRST_NIGHT_CALLED_BACK_HOME` | `workplace_first_night:call_someone` | She reached for continuity with her earlier life. | used now in prose, more callbacks planned |
| `FIRST_NIGHT_STAGED_TOMORROW` | `workplace_first_night:unpack_and_stage_tomorrow` | She imposed order through practical preparation. | used now, active callback |

### First-clothes handling

| Flag | Source | Meaning | Status |
|---|---|---|---|
| `FIRST_CLOTHES_FUNCTIONAL` | `workplace_first_clothes:get_basics` | She solved the wardrobe problem efficiently and impersonally. | reserved callback |
| `FIRST_CLOTHES_MIRROR` | `workplace_first_clothes:dwell_on_mirror` | She stayed with self-recognition long enough for it to matter. | used now, active callback |
| `FIRST_CLOTHES_ASKED_HELP` | `workplace_first_clothes:ask_for_help_outright` | She accepted expert feminine help instead of improvising alone. | used now, active callback |
| `FIRST_CLOTHES_MINIMUM` | `workplace_first_clothes:buy_minimum_and_leave` | She scoped the problem down to survival and bought time. | used now in prose |

### First-day status handling

| Flag | Source | Meaning | Status |
|---|---|---|---|
| `FIRST_DAY_ASSERTED_STATUS` | `workplace_first_day:assert_expertise` | She corrected the room directly. | used now, active callback |
| `FIRST_DAY_DEFERRED_STATUS` | `workplace_first_day:let_it_go` | She let later work make the point. | used now in prose |
| `FIRST_DAY_REDIRECTED_STATUS` | `workplace_first_day:redirect_to_work` | She grounded status in technical substance instead of social correction. | used now, active callback |
| `FIRST_DAY_LUNCH_DESK` | `workplace_first_day:lunch_at_desk` | She consolidated authority through competence-first isolation. | planned erotic/social relevance |
| `FIRST_DAY_LUNCH_GROUP` | `workplace_first_day:lunch_with_group` | She accepted early social exposure and table-read pressure. | planned erotic/social relevance |
| `FIRST_DAY_LUNCH_ALONE` | `workplace_first_day:lunch_alone` | She built urban self-command through controlled withdrawal. | planned erotic/social relevance |

## Callback Layer

### Active callbacks

- `base::opening_callback_status_assertion`
  Fires in `free_time` from week 2 onward once the route is settled if she either
  asserted status directly or redirected the room back onto the code.
- `base::opening_callback_mirror_afterglow`
  Fires in `free_time` from week 2 onward if her first-clothes path created a stronger
  presentation/self-recognition memory.
- `base::opening_callback_first_week_solitude`
  Fires in `free_time` from week 2 onward if her first-night coping style left a clear
  apartment-solitude signature.

### Dormant authored hook

- `base::opening_callback_transactional_defense`
  Authored now, intentionally unscheduled. Reserved for future city/work interactions
  that need to remember early defensive transactionality.

## Week-2 Scenes — Design Notes

**`workplace_work_meeting`**
Her first real meeting. She's been in standups — this is a design review where she
has to speak. She knows the technical content cold. The room will not read her the
way she was read before: she is visibly a woman. Someone will explain something she
originated. She will notice.

The OBJECTIFYING trait fires here — she can read the male gaze on her in this room
with precision because she used to be on the other side of it. Use it for specific
interiority, not a lecture. She recognizes a look. She knows what it means. She
continues presenting her slides.

Branch on: `w.hasTrait("OBJECTIFYING")`, `w.hasTrait("ANALYTICAL")`

**`workplace_evening`**
After work. The apartment. She has been competent all day. She is now alone.
Something small undoes her — not the meeting, not the gaze, something smaller and
more specific. A moment of physical disorientation. Or just the silence.

This scene advances arc → settled because it marks the end of the initial shock
period. She's not okay, but she's operational. There is a difference.

Branch on: `w.hasTrait("ANALYTICAL")`, `w.hasTrait("AMBITIOUS")`

## How This Arc Works

- **Narrator register:** Companion on her shoulder, watching with wry attention.
  The city has its own life independent of her distress. The narrator notices both.
- **Inner voice:** Male pronouns internally at low FEMININITY ("*you*, he thinks"),
  with occasional slippage. The catching-himself is not dramatized — it's just noted.
- **Fetishization:** Present from arrival. Not melodramatic. Just there. The narrator
  doesn't comment on it; her recognition of it does the commentary.
- **The professional/physical gap:** She knows things. Her body signals something else.
  The gap is constant. Never resolved in this arc — it's the texture of the route.

## Schedule Integration

Add these to `packs/base/data/schedule.toml` in a `workplace_opening` slot:

```toml
[[slot]]
name = "workplace_opening"

  [[slot.events]]
  scene     = "base::workplace_arrival"
  condition = "gd.hasGameFlag('ROUTE_WORKPLACE')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE')"
  once_only = true

  [[slot.events]]
  scene     = "base::workplace_landlord"
  condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && !gd.hasGameFlag('MET_LANDLORD')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && !gd.hasGameFlag('MET_LANDLORD')"
  once_only = true

  [[slot.events]]
  scene     = "base::workplace_first_night"
  condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'arrived'"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'arrived'"
  once_only = true

  [[slot.events]]
  scene     = "base::workplace_first_clothes"
  condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'week_one'"
  weight    = 10
  once_only = true

  [[slot.events]]
  scene     = "base::workplace_first_day"
  condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'week_one'"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'week_one'"
  once_only = true

  [[slot.events]]
  scene     = "base::workplace_work_meeting"
  condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'working' && !gd.hasGameFlag('FIRST_MEETING_DONE')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'working' && !gd.hasGameFlag('FIRST_MEETING_DONE')"
  once_only = true

  [[slot.events]]
  scene     = "base::workplace_evening"
  condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'working'"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'working' && gd.hasGameFlag('FIRST_MEETING_DONE')"
  once_only = true
```
