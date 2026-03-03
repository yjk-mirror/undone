# Undone — Creative Direction

> The authoritative source for all creative decisions. This document captures what the
> user has decided. It is not a suggestion — it is a specification. Agents must follow
> it. When in doubt, ask the user. Never invent creative direction to fill a gap.

---

## What This Game Is

A life-simulation adult text game with a transgender/transformation premise. Built in
Rust. Inspired by Newlife (Splendid Ostrich Games) but fully redesigned.

**The premise:** A player character navigates adult life — relationships, work, social
dynamics. She may have started life as a man. The transformation is not backstory; it is
a lens that changes how every socially-gendered experience lands. She knows how men think
because she was one.

**The core erotic logic:** Loss of control. The world happens to her. She responds.
Unpredictability, the world exceeding her choices, situations she didn't invite — this is
baked into the design. Content that leans into this is more aligned than content where
she's in full control.

**The engine is a platform.** All content lives in packs. The engine is setting-agnostic.
The base game is set in a fictional Northeast US city. Nothing setting-specific is
hardcoded into engine code.

---

## The Player Experience — Full Flow

This is the intended experience from launch to gameplay. Every step has been decided.

### 1. Landing Page (not yet built)

New Game / Continue / Load / Settings. The game does not launch straight into char
creation. A player returning to the game can load a save before starting anything new.

### 2. Character Creation: "Who Were You?" (BeforeCreation phase)

The player creates the man they were before the transformation.

- Origin selection (radio buttons with subtitles)
- Before-name, age, race
- Sexuality (for transformed origins)
- Personality traits (grid selection)
- Content preferences (BLOCK_ROUGH opt-out)
- Preset selection available (Robin, Camila, or Custom)

Presets lock all attributes and route the player through a specific arc. Custom players
start freeform with no arc.

### 3. The Plane Scene (TransformationIntro phase)

**This is the decided opening scene.** After creating your character, you board a plane.
You are still a man. The scene reflects your background — who you are, why you're going
to this city, what you're leaving behind. The scene ends when you fall asleep on the
flight.

This scene should:
- Use the before-identity (name, age, traits) established in step 2
- Ground the player in who this person *was* before everything changes
- Establish the move to the new city (workplace preset: new job; campus preset: college)
- End with falling asleep — the transformation happens in the gap

**The transformation happens offscreen, between two screens.** The player creates a man,
watches him fall asleep on a plane, then wakes up as someone else. The gap IS the
transformation. No waking-up-in-a-bedroom scene. No alarm clock. No mirror discovery
montage. The next thing the player sees is the FemCreation screen.

### 4. Character Creation: "Who Are You Now?" (FemCreation phase)

The player wakes up transformed. The gap between falling asleep and this screen is the
transformation — implicit, not shown.

- Feminine name, androgynous name
- Figure, breasts, race (may differ from before)
- Physical attributes
- "Begin Your Story" button assembles the full character and starts the game

### 5. First In-Game Scene (workplace_arrival / campus_arrival)

**For Robin:** The seat belt sign clicks off. She's landing. Airport exit (ID mismatch
beat — "I know, I just look young"), subway or cab to the apartment. The city begins.

**For Camila:** Arrives at campus. Check-in. The world she expected to dominate reads
her differently now.

The first in-game scene is NOT the transformation discovery — that happened in the gap.
The first scene is arriving somewhere and having to function.

### 6. Opening Arc → Settled State → Free Play

The arc state machine drives the first ~2 weeks of scenes. After the arc reaches
"settled" / "first_week", universal scenes fire normally. The game continues
indefinitely.

---

## The Two Presets

### Robin — Workplace Arc

| Field | Value |
|---|---|
| Before name | Robin (gender-neutral, kept) |
| Current name | Robin (all three variants) |
| Age (before) | Early 30s |
| Apparent age | Late teens |
| Race (before) | White |
| Race (now) | East Asian |
| Figure | Petite, huge breasts, stunning |
| Job | Software engineer, 10 years experience |
| Route | `ROUTE_WORKPLACE` |
| FEMININITY start | 10 |

**Player register:** Thirty-two inside. Processes the world like a senior engineer:
systematic, calm, methodical. Doesn't panic — inventories. Not performing competence —
IS competent. The gap is between her internal state and how strangers read her (teenager).

**The fetishization thread:** Was a white man with the casual gaze that fetishizes Asian
women. Unexamined, not malicious. Now receives that exact gaze daily. Can read it
perfectly — knows the internal monologue because it was hers.

**The misrecognition thread:** Looks late teens, has ten years of experience. Gets carded.
Gets explained things she invented. Constant low-grade friction, not comedy.

**Narrator register:** Wry companion on her shoulder. The city has its own life independent
of her distress. The narrator notices both.

**Inner voice:** Male pronouns internally at low FEMININITY ("*you*, he thinks"). The
catching-himself is not dramatized — just noted.

See `docs/presets/robin.md` for full trait list and physical attributes.

### Camila — Campus Arc

| Field | Value |
|---|---|
| Before name | Raul |
| Default name | Camila |
| Age | 18 (freshman) |
| Ethnicity | Latino/Latina (same before and after) |
| University | The Ivy |
| Route | `ROUTE_CAMPUS` |
| FEMININITY start | 10 |
| Sexuality (before) | Straight, ambient homophobia |
| Sexuality (now) | Bi, strong attraction to men |

**Player register:** Eighteen and proud of it. Smart, hasn't been tested. The
transformation is the first thing that happened TO her rather than BY her. Processes it
badly, then better, then badly again.

**The misogyny/homophobia arc:** Made gay jokes (ambient, not violent). Had unexamined
contempt for women. The transformation is a crash course in being wrong. NOT a redemption
narrative. She is a person encountering the reality of positions she held casually.
Whether she learns is up to the player.

**The sexual arousal thread:** Body is responsive in ways that ambush. Attraction to men
arrived immediately and without warning. The shame is specific: she used to make those
jokes. The desire doesn't care about the shame. Write desire before shame, always. Not
comedy — specific bewilderment.

**Narrator register:** Closer to the PC's perspective than Robin's arc. Less wry, more
present in the collision. When something goes wrong, the narrator is in there with her.

See `docs/presets/camila.md` for full profile.

### Contrast

| | Robin | Camila |
|---|---|---|
| Coping | Systematic inventory | Explosive, then recalibration |
| Self-concept | Low investment (pragmatic) | High investment (identity was core) |
| The gaze | Understood immediately | Ambushed by her own response |
| Context | Professional (established) | Academic (beginning) |
| Transformation as | Unexpected constraint | Attack on self |

---

## Content Focus

**CisMale→Woman is the only origin being written.** All other origins (AlwaysFemale,
TransWomanTransformed, CisFemaleTransformed) are deprioritized. Do not write branches
for them.

Pattern: `{% if not w.alwaysFemale() %}` blocks only. No `{% else %}` AlwaysFemale
branches. No `TRANS_WOMAN` branches.

AlwaysFemale players see the default (non-branched) prose, which should read naturally
for anyone. Dedicated AlwaysFemale content is a future pass requiring its own quality bar.

---

## Voice

**Reference: the BG3 narrator.** Dry. Present-tense. Second-person. Wry. Matter-of-fact.
Trusts the scene. Plain English. Nothing artistic. Nothing performative.

**Always second-person present tense.** "You go..." Never "she."

See `docs/writing-guide.md` for full voice rules, anti-patterns, and the complete
authoring checklist. See `docs/writing-samples.md` for calibration prose.

---

## Player Agency and the Engine's Purpose

The engine exists to serve a specific kind of experience. Understanding this is mandatory
for anyone writing scenes or building features.

### What the engine does

The engine runs scenes. A scene has an intro (the world acting), player actions (her
choices), and effects (consequences). The scheduler picks scenes based on game state,
flags, arc progression, and conditions. The player's experience is: the world presents a
situation → she chooses how to respond → consequences ripple forward.

This is a **choice-driven life simulation**, not a visual novel. The player is not
reading a story — she is living a life, making decisions that reflect who her character
is. Every action button is a moment of self-expression.

### Choices must align with character

Actions should feel like things *this specific character* would consider. A SHY character
doesn't get a "confidently confront him" option — she gets "say nothing and leave" or
"try to say something, fail, try again." An AMBITIOUS character doesn't get "drift
through the afternoon" — she gets "use the time" or "push back."

This means:
- **Trait-gated actions are good.** Show a choice only if the character would plausibly
  consider it. Use `condition` fields on actions.
- **Greyed-out actions (BG3 style) are also good.** Show the action but make it
  unavailable with a reason: "[Requires CONFIDENT]" — this teaches the player what their
  character build unlocks without hiding the possibility space.
- **Universal actions need universal plausibility.** If an action appears for all
  characters, it must feel natural for all of them. "Leave" always works. "Flirt openly"
  doesn't — gate it.

### Choices must matter

If two actions lead to the same outcome with different flavor text, the player has no
real choice. Different actions must produce:
- Different consequences (flags, stat changes, NPC reactions)
- Different narrative beats (structurally different scenes, not adjective swaps)
- Different information (the player learns something different about the world or herself)

A choice between "say yes politely" and "say yes enthusiastically" is not a choice. A
choice between "say yes" and "say nothing and see what he does" — that's a choice.

### The world moves first

The intro and NPC actions establish the situation. The world acts on her before she gets
to respond. A man approaches. The rain starts. A coworker says something that changes the
room. The player is reactive, not proactive — the world is the initiator.

This is the core loop: **world acts → player chooses → consequences land → world acts
again.** The player's agency is in HOW she responds, not in WHAT situations she
encounters. The engine (scheduler, triggers, conditions) controls what situations arise.
The player controls what she does about them.

### Scene-level principles (summary)

These are covered in depth in `docs/writing-guide.md`. The essentials:

- **Every scene needs an inciting situation** — something happens before the player
  decides anything
- **1–3 genuine choices** — not cosmetic, with different outcomes
- **At least one lasting consequence** — flag, stat, NPC change
- **Trait branches change what happens** — never just what adjective is used
- **The "fine" test** — if a path summarizes to "she did the thing and it was fine," it
  needs work

---

## Writing Principles That Keep Getting Violated

The writing guide (`docs/writing-guide.md`) has the full rules and checklist. These are
the principles that agents consistently fail to follow. They are surfaced here because
they are the difference between prose that works and prose that doesn't.

### 1. Commit to specific trait axes and let them define the scene

Before writing a scene, decide which 2–4 traits matter HERE. Not all traits matter
everywhere. A grocery store scene doesn't need AMBITIOUS — but it might need SHY (takes
a number instead of walking to the counter) and OBJECTIFYING (catches herself evaluating
the cashier the way she used to be evaluated).

**Commit to those axes.** Write branches where those traits produce structurally different
scenes. Don't scatter shallow references to many traits — go deep on a few. The trait
should change what HAPPENS, not how she FEELS about what happens.

Bad: "{% if SHY %}You feel nervous{% elif CONFIDENT %}You feel confident{% endif %}"
Good: SHY takes a number and waits. CONFIDENT goes to the open register. Different event.

### 2. Every scene must be distinct from every other scene

Not just "earn its place" — actively distinct. Before writing, answer: **what makes this
scene impossible to confuse with any other scene in the game?**

If the answer relies on the location name, it's not distinct enough. "A woman goes to a
bar and notices things are different" could be ANY scene. What's the specific person, the
specific exchange, the specific thing that only happens HERE?

Two bar scenes should feel like two completely different experiences. Different NPCs,
different dynamics, different things at stake. If you wrote them and swapped the location
names and nothing broke, they're the same scene.

### 3. Avoid word and phrase repetition across scenes

AI prose gravitates toward certain words and patterns. Across a body of 30+ scenes, this
creates a homogeneous texture where every scene sounds like the same narrator having the
same day.

**Words that get overused and should be rationed:**
- "specific" / "specifically" — the single most overused AI-prose word. Use it once per
  5 scenes maximum. Replace with the actual specific thing.
- "something about" — vague hand-wave. Replace with what the something IS.
- "the way" / "in the way that" — filler connector. Cut or restructure.
- "a quality" / "a certain quality" — empty frame. Name the quality or show it.
- "you notice" / "you realize" — filtering through observation. Just say the thing.
- "somehow" — vague. Either explain how or don't qualify it.
- "deliberate" / "deliberately" — tells intent instead of showing action.

**The test:** After writing a scene, search for these words. If you find more than 2–3,
rewrite those sentences. The prose should not sound like other scenes in the game.

### 4. No purple prose, no generic prose — find the plain middle

Purple prose: "The amber light cascaded through the window, painting her silhouette
against the gossamer curtains of possibility." — Never.

Generic prose: "You walk into the bar. It's a bar. There are people. You sit down." —
Also never.

The target: "The bar is half-empty on a Tuesday. A man at the end is watching a game he
doesn't care about. The bartender is reading something on her phone behind the register."
— Plain, specific, alive. The details are chosen because they tell you something about
this place at this moment. Not decorated. Not empty.

Every sentence should carry information. If a sentence is only atmosphere with no
information, it needs to either say something or go.

### 5. Show, then trust

The biggest macro-level failure: the narrator TELLS the reader what to feel or understand,
instead of SHOWING the moment and trusting the reader to get it.

- ❌ "You know what he's thinking because you used to think it too."
- ✅ Show what he's doing. The reader connects it. She doesn't need to announce the
  connection — the scene already made it.

- ❌ "There's something about being in a body you didn't choose that makes every
  ordinary moment slightly surreal."
- ✅ Show the ordinary moment. Show what's surreal about it concretely. The reader
  feels the surrealism without the narrator framing it as surreal.

The rule: if the narrator is naming the emotional category, the scene hasn't earned the
emotion yet. Go back and earn it with a concrete moment. Then delete the narrator's
label. The reader will supply it themselves, and it will land harder.

---

## What Agents Must Never Do

1. **Never invent creative direction.** If a scene needs to be written and no creative
   spec exists for it, stop and ask. Do not fill the gap with generated content. The
   `transformation_intro` bedroom scene is the example of this failure — an agent wrote
   a scene for a slot without asking what the scene should be. The result was technically
   functional and creatively wrong.

2. **Never decide what the player's opening experience is.** The flow from launch to
   first scene is specified above. Any change to this flow requires user approval.

3. **Never write scenes that only exist to demonstrate a system.** Every scene must earn
   its place (see writing guide). A scene that exists because "we needed a free_time scene"
   is not a scene — it's a placeholder. Ask for creative direction for the specific
   moment.

4. **Never substitute telling for showing the transformation.** "You know what men think
   because you were one" is telling. Showing her recognize the exact calculation a man is
   running because she used to run it — that's showing. The insight should be demonstrated
   through what the PC notices, not announced by the narrator.

5. **Never hammer the same transformation beat explicitly.** Multiple scenes saying "you
   used to be a man so you understand" is repetitive. Each scene should find a NEW
   concrete observation that demonstrates the insider knowledge. The reader should think
   "oh — she knows" without the narrator saying "she knows."

---

## Engineering Guardrails for Creative Integrity

These are rules for how the engineering side protects creative decisions.

### Scene slots require creative specs

The engine has scene slots (TransformationIntro, opening scene, free_time, etc.). When
a slot needs a new scene, the engineering response is to create the slot infrastructure
and flag the content gap. The engineering response is NOT to write a scene to fill the
slot. Scenes are creative artifacts. They require user direction.

### The char creation flow is specified

The phase sequence (BeforeCreation → TransformationIntro → FemCreation → InGame) is
decided. Changes to this sequence require user approval. Adding, removing, or reordering
phases is a creative decision, not an engineering one.

### Preset data is creative data

Preset trait lists, physical attributes, names, and personality configurations are
creative decisions. Agents may implement infrastructure to load/display presets, but
must not modify preset content without direction.

### Writing quality is non-negotiable

Every scene goes through the writing-reviewer agent. All Critical findings are fixed
before merge. This is not optional and is not skipped for velocity. See the sprint
roadmap quality protocol.

---

## Documents That Flesh Out These Decisions

| Document | What it covers |
|---|---|
| `docs/writing-guide.md` | Full prose standard, anti-patterns, checklist, template syntax |
| `docs/writing-samples.md` | Approved prose excerpts — quality calibration |
| `docs/world.md` | Setting facts (city, technology, neighborhoods) |
| `docs/presets/robin.md` | Robin's full trait list, physical attributes, thematic threads |
| `docs/presets/camila.md` | Camila's full profile, contrast with Robin |
| `docs/arcs/workplace-opening.md` | Workplace arc state machine, scene specs, tone notes |
| `docs/arcs/campus-opening.md` | Campus arc state machine, scene specs, contrast |
| `docs/content-schema.md` | TOML schema reference (effects, conditions, expressions) |
| `docs/plans/2026-02-25-sprint-roadmap.md` | Sprint plan and quality protocol |
