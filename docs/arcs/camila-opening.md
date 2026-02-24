# Arc: base::camila_opening

## Narrative Purpose

Establish Camila at the Ivy. Ground the transformation in the specific experience
of a privileged 18-year-old whose self-concept was their identity, now encountering
the world from a position they didn't choose and didn't expect. The city and university
are vivid and indifferent.

## State Machine

```
arrived → orientation → dorm_life → first_week
```

**arrived** (move-in day)
Scenes: `camila_arrival`, `camila_dorm`

**orientation** (day 1–3)
Scenes: `camila_orientation`, early social scenes

**dorm_life** (first week, settled in dorm)
Scenes: `camila_library`, social scenes

**first_week** (completed first full week)
Universal scenes fire normally. Camila-specific flavor via `[[intro_variants]]`.

## Scene List

| Scene ID | Arc state gate | Location | Content | Sets |
|---|---|---|---|---|
| `base::camila_arrival` | none (once-only, ROUTE_CAMILA) | Campus, check-in | VANILLA | `ROUTE_CAMILA`, arc→arrived |
| `base::camila_dorm` | arrived | Dorm room | VANILLA | arc→orientation |
| `base::camila_orientation` | orientation | Quad + orientation events | VANILLA | arc→dorm_life |
| `base::camila_library` | dorm_life | The Ivy library | VANILLA | — |
| `base::camila_call_raul` | dorm_life | Dorm room (phone) | VANILLA | `CALL_HOME_DONE` |

## Tone Notes

- **Narrator register:** Closer to Camila's perspective than Robin's — less wry, more
  present in the moment of collision. When something goes wrong, the narrator doesn't
  observe from a distance; it's in there with her.
- **The ambush:** Camila's moments of recognition are less prepared. Things ambush her.
  The attraction to men arrived without warning. The shame arrived after the desire.
  Write it in that order.
- **Privilege and its inversions:** Some things that were hard for Raul are easy for
  Camila. Some things that were easy are hard. Neither is framed as justice — it's
  just how the world distributes its frictions.
- **The phone call home:** One of the sharpest moments in the arc. She cannot explain
  anything. She has to be normal. Write the conversation around the missing explanation.

## Contrast with Robin arc

Robin's arc is about competence in the wrong body. Every scene, she knows more than
the world thinks she knows. The tension is between her knowledge and the world's read.

Camila's arc is about learning the wrong assumptions. Every scene, she discovers she
thought she knew something and didn't. The tension is between her certainty and the
gap that opens when certainty meets reality.

## Schedule Integration

Add these to `packs/base/data/schedule.toml` in a `camila_opening` slot:

```toml
[[slot]]
name = "camila_opening"

  [[slot.events]]
  scene     = "base::camila_arrival"
  condition = "gd.hasGameFlag('ROUTE_CAMILA')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_CAMILA')"
  once_only = true

  [[slot.events]]
  scene     = "base::camila_dorm"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'arrived'"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'arrived'"
  once_only = true

  [[slot.events]]
  scene     = "base::camila_orientation"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'orientation'"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'orientation'"
  once_only = true

  [[slot.events]]
  scene     = "base::camila_library"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'dorm_life'"
  weight    = 10
  once_only = true

  [[slot.events]]
  scene     = "base::camila_call_raul"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'dorm_life' && !gd.hasGameFlag('CALL_HOME_DONE')"
  weight    = 8
  once_only = true
```
