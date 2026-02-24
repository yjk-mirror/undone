# Arc: base::robin_opening

## Narrative Purpose

Establish Robin in the city. Ground the transformation in specific physical and
social experiences. Set the tone for her route: pragmatic, wry, quietly overwhelmed.
Introduce the city as a place with its own life.

## State Machine

```
arrived → week_one → working → settled
```

**arrived** (Saturday, day 0–1)
Scenes: `robin_arrival`, `robin_landlord`, `robin_first_night`

**week_one** (Sunday before first workday)
Scenes: `robin_first_clothes`, plus universal slot scenes as available

**working** (Monday onward, has started job)
Scenes: `robin_first_day`, universal work-adjacent scenes

**settled** (after ~week 2, has basic routines)
Universal scenes fire normally. Robin-specific flavor via `[[intro_variants]]`.

## Scene List

| Scene ID | Arc state gate | Location | Content | Sets |
|---|---|---|---|---|
| `base::robin_arrival` | none (once-only, ROUTE_ROBIN) | Airport → subway | VANILLA | `ROUTE_ROBIN`, arc→arrived |
| `base::robin_landlord` | arrived | Her apartment building | VANILLA | `MET_LANDLORD` |
| `base::robin_first_night` | arrived | Her apartment | VANILLA | arc→week_one |
| `base::robin_first_clothes` | week_one | Clothing store | VANILLA | — |
| `base::robin_first_day` | week_one | Tech office | VANILLA | arc→working, `STARTED_JOB` |

## Tone Notes

- **Narrator register:** Companion on Robin's shoulder, watching with wry attention.
  The city has its own life independent of Robin's distress. The narrator notices both.
- **Inner voice:** Male pronouns internally at low FEMININITY ("*you*, he thinks"),
  with occasional slippage. The catching-himself is not dramatized — it's just noted.
- **Fetishization:** Present from arrival. Not melodramatic. Just there. The narrator
  doesn't comment on it; Robin's recognition of it does the commentary.
- **The professional/physical gap:** She knows things. Her body signals something else.
  The gap is constant. Never resolved in this arc — it's the texture of the route.

## Schedule Integration

Add these to `packs/base/data/schedule.toml` in a `robin_opening` slot:

```toml
[[slot]]
name = "robin_opening"

  [[slot.events]]
  scene     = "base::robin_arrival"
  condition = "gd.hasGameFlag('ROUTE_ROBIN')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_ROBIN')"
  once_only = true

  [[slot.events]]
  scene     = "base::robin_landlord"
  condition = "gd.hasGameFlag('ROUTE_ROBIN') && !gd.hasGameFlag('MET_LANDLORD')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_ROBIN') && !gd.hasGameFlag('MET_LANDLORD')"
  once_only = true

  [[slot.events]]
  scene     = "base::robin_first_night"
  condition = "gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'arrived'"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'arrived'"
  once_only = true

  [[slot.events]]
  scene     = "base::robin_first_clothes"
  condition = "gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'week_one'"
  weight    = 10
  once_only = true

  [[slot.events]]
  scene     = "base::robin_first_day"
  condition = "gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'week_one'"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'week_one'"
  once_only = true
```
