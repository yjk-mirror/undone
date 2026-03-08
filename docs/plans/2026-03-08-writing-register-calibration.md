# Writing Register Calibration — Design

**Date:** 2026-03-08
**Status:** Approved

## Problem

The writing documentation and agent instructions allow patterns the user has rejected through
hands-on calibration. Three draft scenes (neighborhood_bar, transformation_intro, workplace_arrival)
were stashed because the prose was:

- Too literary / novelistic — narrator performing cleverness instead of serving the player
- Too omniscient — narrator knowing things the player can't know
- Too much interiority — putting full articulated thoughts in the player's head
- Deciding for the player — ordering drinks, choosing how to sit, in the intro
- Using `{% if not w.alwaysFemale() %}` guards unnecessarily — cluttering transformation prose
- Shallow actions — filler choices like "check your phone" that don't lead anywhere

## The Calibrated Register

Established through iterative writing attempts with user feedback:

### 1. DM narrator, not novelist

The narrator is on the player's shoulder, pointing things out. Like a good D&D dungeon master:
present, concise, specific, with personality but not showing off. Not omniscient. Not dramatic.
Not literary. Not performing.

From Angry GM research: "Narration is about clear and concise verbal communication. It is about
imparting information." The player's imagination does the heavy lifting.

From BG3 narrator (Amelia Tyler): "With my chin on the player's shoulder, whispering 'look at
that', tilting their chin but never manipulating them."

### 2. Intro = world acting. Actions = player deciding.

The intro describes where you are, what's happening around you, and what's happening TO you.
It never decides what you do — not what you order, how you sit, what you say, or what you think.

Actions are the player's choices. Each action should be a real decision that leads somewhere
meaningful. Trait branches in actions change how the world responds to the player's choice,
not what the player chooses.

### 3. Physical, not meta

Transformation is shown through immediate physical experience:
- "The stool takes a small hop to get onto. The bar top comes up higher than you expect and your hands look small against the wood."
- NOT: "None of this was conscious." "Your body is making calculations." "You're doing the thing women do."

The narrator reports what the player experiences. It doesn't analyze, explain, or editorialize
about the experience.

### 4. Drop alwaysFemale guards

Since only CisMale→Woman content is being written, transformation prose IS the prose. No
`{% if not w.alwaysFemale() %}` wrapping needed. Write it directly. The guards cluttered code
and broke prose flow.

### 5. Every scene earns its place through depth

Scenes must be intentional, deep, and richly branched. If a scene or action doesn't go
somewhere meaningful, it doesn't exist. No filler actions ("check your phone"). No scenes
that are just "woman goes to location, notices things are different."

### 6. Rich branching that goes deep

Traits don't change flavor text — they open and close entire paths. A SHY character and a
CONFIDENT character should have genuinely different scenes unfold, not the same scene with
different adjectives. Actions lead to real consequences and further decision points.

### What the register sounds like (approved example)

```
Donovan's is half-empty on a Tuesday, which is probably why you picked it.

Warm lighting, fryer oil and hops. A couple sharing nachos in the corner booth, leaned
in close enough that whatever they're saying is just for them. Three guys at the far end
of the bar with a pitcher and a game on TV. A woman sitting alone near the taps with a
glass of white wine, reading something on her phone with the posture of someone who came
here specifically to be left alone.

The door shuts and the cold drops off you. It's warm in here, warm enough that your jacket
is already too much. The bartender looks up when you come in. She's already put a coaster
down by the time you reach the bar.

[BEAUTIFUL branch]
One of the guys at the end has stopped watching the game. You can feel it on the side of
your face before you look over. His friend follows his eyes to you. Now there are two.

[FEMININITY < 25 branch]
The stool takes a small hop to get onto. The bar top comes up higher than you expect and
your hands look small against the wood.

The coaster is waiting. The bartender is waiting.
```

Why this works:
- Casual, readable, not trying to impress
- Describes the room as you'd notice it walking in
- BEAUTIFUL changes what's happening TO you (men watching), not what you do
- Transformation is physical and immediate (stool, hands), no meta-commentary
- Ends by handing control to the player (bartender is waiting)
- No narrator explaining significance, no full thoughts inserted

### What the register does NOT sound like (rejected patterns)

- "A position that says *I'm here and I'm fine and I haven't decided yet.*" — narrator putting a full thought in the player's head
- "The armor went up without you deciding to put it on." — narrator analyzing what the body is doing
- "the opinion is *more of that, please*" — narrator deciding what the player desires
- "You know the architecture of this interaction from the inside." — the banned "you used to do this" pattern rephrased
- "A bus passes outside." — filler detail that adds nothing
- "You wrap your hands around the glass because your hands need something to do." — narrator explaining the player's motivation

## Files to Update

| File | Changes |
|---|---|
| `docs/writing-guide.md` | Add DM narrator register section. Rewrite player agency (intro vs actions). Remove alwaysFemale guard instructions — transformation is direct prose. Add "no filler actions" rule. Update transformation textures to physical-only. |
| `docs/creative-direction.md` | Update voice with DM register. Remove guard instructions. Add scene depth requirement. |
| `docs/writer-core.md` | Rewrite voice (DM not novelist). Drop guard syntax from examples. Emphasize intro vs action split. Add rejected patterns as anti-patterns. |
| `docs/review-core.md` | New Critical: narrator deciding player actions in intro. New Critical: filler actions. New Important: meta-commentary about body/transformation. |
| `docs/writing-samples.md` | Add calibrated bar example as primary sample. Annotate what works and what failed. |
| `.claude/agents/scene-writer.md` | Add register guidance. Emphasize depth over coverage. |
| `.claude/agents/writing-reviewer.md` | Add detection for: narrator deciding actions, meta-commentary, filler actions, omniscient narrator. |

## Success Criteria

After updates, an autonomous writing session using these docs should produce prose that:
1. Reads like a DM narrating, not a novelist crafting
2. Never decides player actions in intro prose
3. Has no meta-commentary about transformation ("none of this was conscious")
4. Uses no alwaysFemale guards
5. Contains only actions that lead somewhere meaningful
6. Branches deeply on traits, changing what happens not how it's described
