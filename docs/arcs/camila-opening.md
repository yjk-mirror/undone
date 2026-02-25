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
| `base::camila_study_session` | dorm_life | Study room / dorm | VANILLA | `STUDY_SESSION_DONE` |
| `base::camila_dining_hall` | dorm_life | The Ivy dining hall | VANILLA | arc→first_week |

## Week-2 Scenes — Design Notes

**`camila_study_session`**
Camila trying to study. Something is harder to concentrate on than it should be.
The material isn't the problem — she's always been good at school. Her body is
the problem. She keeps noticing things she didn't used to notice. Maybe it's a
classmate. Maybe it's just the fact of existing in this new body in a room full
of people and not knowing how to sit.

SEXIST and HOMOPHOBIC fire here as specific interiority: she had assumptions about
who does well at certain subjects, what certain types of students are like. She
catches one of those assumptions against the reality she's now inside.

Branch on: `w.hasTrait("SEXIST")`, `w.hasTrait("HOMOPHOBIC")`, `w.hasTrait("AMBITIOUS")`

**`camila_dining_hall`**
The dining hall. Camila had a clear read on social hierarchies in high school.
This place has different ones and she's not in the position she expected to be in.
Someone is kind to her in a way that would have confused Raul. Someone is dismissive
in a way that would have been impossible for Raul.

This scene advances arc → first_week. She has been here a week. She does not yet
know what she doesn't know, but the shape of what she doesn't know has become visible.

Branch on: `w.hasTrait("SEXIST")`, `w.hasTrait("CONFIDENT")`, `w.hasTrait("OUTGOING")`

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

  [[slot.events]]
  scene     = "base::camila_study_session"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'dorm_life' && !gd.hasGameFlag('STUDY_SESSION_DONE')"
  weight    = 10
  once_only = true

  [[slot.events]]
  scene     = "base::camila_dining_hall"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'dorm_life' && gd.hasGameFlag('STUDY_SESSION_DONE')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'dorm_life' && gd.hasGameFlag('STUDY_SESSION_DONE')"
  once_only = true
```
