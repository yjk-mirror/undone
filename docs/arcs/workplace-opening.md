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
| `base::workplace_arrival` | none (once-only, ROUTE_WORKPLACE) | Airport → subway | VANILLA | `ROUTE_WORKPLACE`, arc→arrived |
| `base::workplace_landlord` | arrived | Her apartment building | VANILLA | `MET_LANDLORD` |
| `base::workplace_first_night` | arrived | Her apartment | VANILLA | arc→week_one |
| `base::workplace_first_clothes` | week_one | Clothing store | VANILLA | — |
| `base::workplace_first_day` | week_one | Tech office | VANILLA | arc→working, `STARTED_JOB` |
| `base::workplace_work_meeting` | working | Tech office — meeting room | VANILLA | `FIRST_MEETING_DONE` |
| `base::workplace_evening` | working | Her apartment (evening) | VANILLA | arc→settled |

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
