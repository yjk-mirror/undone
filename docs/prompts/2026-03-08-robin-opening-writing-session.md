# Robin Opening Sequence — Writing Session

## Read first (in order)

1. `docs/plans/2026-03-08-writing-register-calibration.md` — what the register is and why
2. `docs/writing-samples.md` — Sample 0 is the calibration target. Match that register.
3. `docs/writing-guide.md` — full rules, checklist, trait tables
4. `docs/creative-direction.md` — creative bible, Robin profile, opening flow
5. `docs/presets/robin.md` — Robin's full trait list, physical attributes, thematic threads
6. `docs/arcs/workplace-opening.md` — workplace arc state machine, scene specs
7. `HANDOFF.md` — current state, what exists

## The task

Rewrite the Robin opening sequence to the calibrated register. These are the scenes
a player encounters from the moment they finish character creation through the first
few in-game days. Each scene must be intentional, deep, and richly branched.

**Quality over quantity.** 3 excellent scenes are worth more than 8 shallow ones. Take
the time to make every action meaningful, every branch go somewhere, every trait produce
a genuinely different experience.

## Scenes to write (in play order)

### 1. `transformation_intro.toml` — The plane

The decided opening. She boards a plane as the man she was. Route-aware setup (Robin:
new software job in the city). Falls asleep over Ohio. The transformation happens in
the gap before FemCreation.

This scene has NO actions — it's a read-through intro. But it needs to be rich enough
that the player feels grounded in who this person was. Use before-body accessors
(`w.beforeName()`, `w.beforeHeight()`, etc.) — this is one of the few places
`{% if not w.alwaysFemale() %}` is needed for those accessors.

The stashed version exists (`git stash show stash@{0}`) but does NOT match the register.
Write fresh.

### 2. `workplace_arrival.toml` — Landing in the city

The seat belt sign clicks off. Airport. ID mismatch beat. Subway or cab to the apartment.
The city begins. This is her first scene as a woman in the world.

Existing file needs full rewrite to new register. Multiple rounds of player choices:
how she handles the airport, getting to the apartment, first moments alone in the new
place.

### 3. `workplace_first_day.toml` — First day at work

Already exists. Needs rewrite to new register. Should be deep — meeting coworkers,
navigating being the new person, the specific tension of being a 32-year-old engineer
who looks like a teenager. Multiple decision points.

### 4. One more scene of your choice

Pick whichever scene from the early Robin experience would benefit most from the new
register. Could be `neighborhood_bar.toml` (we calibrated on this), `morning_routine.toml`,
`plan_your_day.toml`, or something else. Write it deep.

## Register rules (non-negotiable)

- **DM narrator.** On the player's shoulder. Casual, specific, present. Not a novelist.
- **Intro = world. Actions = player.** The intro never orders drinks, sits down, or speaks.
- **No meta-commentary.** "None of this was conscious" — banned. Physical facts only.
- **No full thoughts.** Inner voice = fragments. Not "*more of that, please.*"
- **No filler actions.** Every action leads somewhere. Decision chains, not dead ends.
- **Transformation is physical.** Stool too tall. Hands look small. Not analyzed.
- **No `{% if not w.alwaysFemale() %}` guards** except for before-body accessors.
- **Depth.** Traits open/close paths. Branches go deep. Choices produce different scenes.

## Workflow per scene

1. Read the existing file and the arc docs
2. Design the scene structure: what happens, what choices matter, what traits apply
3. Write the full TOML with deep branching
4. Validate with `mcp__minijinja__jinja_validate_template`
5. Self-review against `docs/review-core.md` criteria
6. If any Critical findings, fix them before moving to next scene

## After writing

- Run `cargo run --bin validate-pack` to check all scenes
- Update `HANDOFF.md` with what was written and current state
- Commit with a descriptive message
- Do NOT push — user will review first
