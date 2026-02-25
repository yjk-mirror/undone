# Undone — Writing Guide

The prose standard for all content in this project. Read before writing a single line of text.

---

## Purpose

This guide applies to all scene prose in any pack. Every rule here applies equally whether
you are writing a solo street encounter or an intimate scene. The engine exists to serve
the writing. The writing exists to make the world feel real.

---

## Voice and Register

The narrator is second-person, present tense, slightly detached — dry, observational,
with occasional dark humour. It is **not**:

- Literary or self-consciously artistic
- Pornographic in euphemism ("his throbbing need")
- Clinical or mechanical
- Chipper and upbeat ("What a fun adventure!")

It sits somewhere between a wry novel and someone telling you exactly what happened to them.
The narrator notices things. It has opinions but doesn't editorialize directly. Casual-to-
conversational without being sloppy.

**Reference voice: the BG3 narrator.** Dry. Present-tense. Second-person. Wry. Matter-of-fact.
Trusts the scene. Doesn't signal significance before the moment earns it. Plain English. Nothing
artistic. Nothing performative. Just what happened, with occasional dry observation.

---

## POV and Tense

- Always second-person: "You go...", "You see...", "He says..."
- Always present tense: "You walk home" not "You walked home"
- **Vary sentence starters** — don't open every sentence with "You"

---

## American English (NE US Setting)

The base pack is set in a fictional Northeast US city. Use American spellings and American
cultural references throughout. Never use British equivalents.

| Use | Not |
|-----|-----|
| bar | pub |
| apartment | flat |
| sidewalk | pavement |
| cell phone | mobile |
| trash / garbage | rubbish |
| line | queue |
| wasted, drunk | sloshed, pissed (drunk sense) |
| CVS, Walgreens, Rite Aid | Boots, Superdrug |
| Dunkin', Starbucks, local café | Costa, Pret |
| Trader Joe's, Stop & Shop, ShopRite | Lidl, Aldi (Aldi exists but feels European) |
| TJ Maxx, Marshall's | Primark |
| dive bar, sports bar | Wetherspoons |
| local deli, bodega, Panera | Greggs |
| a twenty, five bucks | quid, fiver, tenner |
| awesome, terrible | brilliant, rubbish (as adjective) |
| assholes, punks | yobs |

Brands that exist in this world: CVS, Walgreens, Dunkin', Starbucks, Trader Joe's, Target,
Stop & Shop, ShopRite, TJ Maxx, Panera. A bodega is a corner store in this city's vocabulary.
The city has good public transit and a distinctly Northeastern class geography.

---

## Markdown in Prose

The game renders markdown in all prose fields. Use it deliberately and sparingly.

| Markup | Use for |
|--------|---------|
| `*italics*` | The PC's inner voice diverging from narration; emphasis; a word she's thinking rather than saying |
| `**bold**` | Very rarely — a single word carrying real weight. Not decoration. |

**When italics earn their place:**

```
He's still talking. You stopped listening a while ago.
*You used to do this too*, you think. Not listen. Assume the other person would catch up.
```

**When they don't:**

```
You feel *nervous*. (Don't announce emotions — show the evidence instead.)
```

Never use markdown for decoration. If it doesn't change meaning, remove it.

---

## Template Syntax Quick Reference

### Prose templates (inline in `.toml` scene files)

Prose fields use [Minijinja](https://docs.rs/minijinja) (Jinja2-compatible).

```jinja
{% if w.hasTrait("SHY") %}
You find a spot at the far end and study the sidewalk.
{% elif w.hasTrait("FLIRTY") %}
You lean against the wall and smile at him before you've decided to.
{% else %}
You nod at the man already there. He nods back.
{% endif %}
```

**Available template objects:**

| Object | Methods |
|--------|---------|
| `w` | `hasTrait("ID")`, `isVirgin()`, `alwaysFemale()`, `pcOrigin()`, `isSingle()`, `isOnPill()`, `isPregnant()`, `getSkill("ID")`, `getMoney()`, `getStress()`, `wasMale()`, `wasTransformed()` |
| `gd` | `hasGameFlag("FLAG")`, `week()`, `day()`, `timeSlot()`, `isWeekday()`, `isWeekend()`, `arcState("arc_id")` |
| `scene` | `hasFlag("FLAG")` |

**PC origin helpers:**
- `w.alwaysFemale()` — `true` if `CisFemaleTransformed` or `AlwaysFemale`; `false` if transformed from male
- `w.pcOrigin()` — returns the origin string (`"CisMaleTransformed"`, `"TransWomanTransformed"`, `"CisFemaleTransformed"`, `"AlwaysFemale"`) — for future use, not needed in current content

**Arc state branching:**
- `gd.arcState("arc_id")` — returns the current state string for the arc, or `""` if not yet started. Use `== ""` to check if an arc has not started yet.

```jinja
{% if gd.arcState("base::robin_opening") == "working" %}
She's settling into the rhythm now.
{% elif gd.arcState("base::robin_opening") == "" %}
Everything is still new.
{% endif %}
```

### Condition expressions (in `.toml` `condition` fields)

Conditions use the custom expression language (not Minijinja).

```toml
condition = "w.hasTrait('SHY')"
condition = "!w.alwaysFemale() && w.getSkill('FEMININITY') < 25"
condition = "gd.hasGameFlag('MET_DAVID')"
condition = "w.getMoney() > 50"
```

**Expression receiver reference:**

| Receiver | Examples |
|----------|---------|
| `w.` | `hasTrait('ID')`, `getSkill('ID')`, `getMoney()`, `getStress()`, `isVirgin()`, `alwaysFemale()`, `isDrunk()`, `isSingle()` |
| `m.` | `hasTrait('ID')`, `isPartner()`, `isFriend()`, `hadOrgasm()`, `isNpcLoveCrush()` |
| `gd.` | `hasGameFlag('FLAG')`, `week()`, `getStat('ID')` |
| `scene.` | `hasFlag('FLAG')` |

---

## Trait Branching: The Fundamental Rule

**Branches must change what happens — not what adjective is used.**

### Bad (adjective swap — never do this):

```jinja
{% if w.hasTrait("POSH") %}
You smile gracefully at him.
{% elif w.hasTrait("CUTE") %}
You smile cheerily at him.
{% else %}
You smile at him.
{% endif %}
```

Every path is structurally identical. The player experiences the same scene regardless of
who she is. This is the most common failure mode.

### Good (structural difference):

```jinja
{% if w.hasTrait("POSH") %}
You give him the slight, closed-lip smile you reserve for strangers who need to feel
acknowledged but not encouraged. He takes it as an invitation anyway.
{% elif w.hasTrait("CUTE") %}
You beam before you can stop yourself. He looks pleased in a way that makes you feel
vaguely responsible for his afternoon.
{% elif w.hasTrait("BITCHY") %}
You don't smile. He reads it correctly and moves on.
{% else %}
You catch his eye by accident. The moment stretches until one of you looks away.
{% endif %}
```

The scene changes. The outcome changes. The character is present in what *happens*, not
just in which word colors one moment.

Pick 2–4 player traits that genuinely change whether this situation is enjoyable,
uncomfortable, awkward, or dangerous for that type of person. Write a solid `{% else %}`
that works for everyone else.

---

## PC Traits Quick Reference

Use these to build branches that structurally change what happens.

| Trait | What she's like | How scenes shift |
|-------|----------------|-----------------|
| `POSH` | Notices class signals. Faintly superior. | Avoids anything slovenly. Reads the room for status. Goes to the nice coffee shop. |
| `CUTE` | Genuine enthusiasm, easily delighted. | Can be taken advantage of through naivety. Beams first, thinks later. |
| `SULTRY` | Aware of her effect on people. | Operates with deliberate ease. Turns attention into a resource. |
| `DOWN_TO_EARTH` | Practical, unselfconscious. | Good value matters. No pretension. Comfortable with bluntness. |
| `BITCHY` | Low tolerance for nonsense. | Notices what's wrong and makes it known. Often right. Situations end faster. |
| `SHY` | Avoids eye contact, defers, gets flustered. | Actions cost more than they look. May fail to do what she wanted to do. |
| `REFINED` | Sensitive to vulgarity. | Dislikes crudeness. Has opinions about quality and presentation. |
| `ROMANTIC` | Takes things slightly more seriously than warranted. | Notices possibility in small moments. Attaches meaning. |
| `FLIRTY` | Can't entirely help it. | Context doesn't always matter. |
| `AMBITIOUS` | Goal-focused, impatient. | Evaluates everything. Situations that waste time irritate her. |
| `OVERACTIVE_IMAGINATION` | Takes situations to their logical conclusion. | Gets ahead of herself. Anticipates outcomes before they exist. |
| `PLAIN` | Not conventionally attractive. | Some male attention routes don't fire or land differently. |
| `BEAUTIFUL` | Draws attention. | More male attention, more often, and she knows it. |

---

## NPC Personality and Voice

Every NPC line must reflect that NPC's personality and what they want from this specific
interaction. Never write generic NPC dialogue that any personality could say.

**Core personalities:**

| Personality | Voice and behaviour |
|-------------|-------------------|
| `JERK` | Transactional, contemptuous. Performs warmth only when he wants something. Drops it immediately when he doesn't get it. |
| `SELFISH` | Self-absorbed. Relates everything to himself. Genuinely doesn't notice what others need. |
| `AVERAGE` | Ordinary. No particular edge or warmth. The baseline of humanity trying to fill the time. |
| `ROMANTIC` | Earnest, attentive, occasionally overwrought. Notices things about her and says so. May mean too much by small gestures. |
| `CARING` | Actively interested. Asks follow-up questions. Adjusts to what she needs. Remembers things. |

**Modifier traits:**

| Trait | Effect on voice |
|-------|----------------|
| `SLEAZY` | More sexually forward, less tactful, pushes past comfort faster |
| `CHARMING` | Reads the room well, can fake warmth convincingly |
| `BOASTFUL` | Redirects to himself, misses signals, needs an audience |
| `CRUDE` | Less filter between thought and mouth, swears more |
| `TACITURN` | Minimal dialogue, communicates through gesture and implication |
| `INTERESTING` | Has something worth saying; draws her in even when she didn't plan to engage |

---

## Anti-Patterns (Never Write These)

### 1. Emotion announcement

- ❌ "You feel a wave of embarrassment."
- ❌ "You're nervous."
- ❌ "A pang of loneliness hits you."
- ✅ Show the physical or behavioural evidence. Let the reader feel it.

### 2. Heart and pulse clichés

- ❌ "Your heart skips a beat."
- ❌ "Your pulse quickens."
- ❌ "Your heart races."
- ❌ "A shiver runs down your spine."

### 3. Generic NPC dialogue

- ❌ "You look beautiful tonight."
- ❌ "Want to get out of here?"
- ❌ "You're amazing, you know that?"
- ✅ Dialogue should reflect this NPC's personality, his current goal, and the specific situation.

### 4. Passive observation chains

- ❌ "You notice a man. You see he is tall. You observe that he's looking at you."
- ✅ Enter mid-action. Pick one detail that matters. Let it mean something.

### 5. Step-by-step narration without texture

- ❌ "You walk to the counter. You order a coffee. You pay. You wait. Your coffee arrives."
- ✅ Skip the mechanical steps. Write what's actually interesting about this moment for this character.

### 6. Perfect structural symmetry

Every scene doesn't need the same number of branches or the same beats. A POSH branch
might be two sentences. A SHY branch might run longer and end worse. Real situations
are asymmetric.

### 7. Resolving everything neatly

Not every encounter ends with resolution. The man can just walk away. The awkward moment
can stay awkward. The world doesn't owe the player a tidy conclusion.

### 8. AI erotic clichés

| ❌ Never | Why it fails |
|---|---|
| "She bit her lip" | Universal AI tic, signals nothing |
| "She felt heat building inside her" | Tells not shows, vague |
| "She couldn't help herself" | Removes agency without interest |
| "His hands explored her body" | Which hands, which body, what specifically |
| "She moaned softly" | Default; every encounter doesn't sound the same |
| "He looked at her hungrily" | Stock phrase; what does hungry look like on *this* man |
| "She was lost in the moment" | Abstraction where the moment itself should be |

### 9. Staccato declaratives for dramatic effect

The single biggest AI prose tell. Using sentence structure to signal importance.

- ❌ Single-sentence pause drops: "He grabs the counter."
- ❌ Anaphoric repetition: "It happens fast. It happens the way a mirror breaks."
- ❌ Em-dash reveals: "Not danger, exactly — more like being *placed*."
- ❌ Trailing staccato closers: "The city goes on."
- ✅ Say the thing. The weight comes from the event, not from how the sentence sits on the page.

**Test:** read it aloud. If it sounds like a movie trailer voiceover, rewrite it.

### 10. Over-naming experiences

The narrator names the emotional category instead of showing the thing.

- ❌ "the universal stranger-in-shared-misery nod" — labelling it, not showing it
- ❌ "There's a specific quality to being looked at by a strange man in a small space." — announcing significance before delivering it
- ❌ "more like being *placed*" — italicised coinage in place of observation
- ✅ Show the nod. Show the look. The reader names it themselves.

---

## The World Has Its Own Life

The narrator should occasionally notice things that have nothing to do with the player's
choices. The city has weather. The bar has other people doing things. Background events
interrupt. Not everything resolves because of what the player chose.

**Small specific details over sweeping atmosphere:**

- ❌ "The park is beautiful and peaceful."
- ✅ "A kid is being dragged away from the fountain by her dad while her ice cream melts down her wrist."

The player is a person in a city, not the centre of that city. The world was happening before
she arrived and continues after.

---

## The Transformation Dimension

This is the game's deepest and most distinctive creative territory.

The PC may have been transformed from male to female. Most games treat such a character as
if she always was a woman. That is a missed opportunity of the first order.

The transformation is not a premise to acknowledge once and forget. It is a **lens** that
changes how every socially-gendered experience lands. A catcall, a hand held open, being
talked over, being protected, feeling desire for a man — these things mean something
*different* for a woman who remembers being a man. She *knows* how men think. She's about
to find out what that means from the other side.

**Use this. It is what makes this game specific.**

---

### The PC Origins

The PC origin is determined by the `PcOrigin` enum, accessible via `w.pcOrigin()` (returns
a string). The primary origins for current content are:

| Origin | `w.alwaysFemale()` | Description |
|--------|---------------------|-------------|
| `CisMaleTransformed` | `false` | Cis man transformed. The disorientation register. **The only origin being written right now.** |

**Deprioritized origins (do not write branches for these yet):**

| Origin | Notes |
|--------|-------|
| `AlwaysFemale` | No transformation frame. Will be a future content pass — requires its own complete, nuanced path. |
| `TransWomanTransformed` | Relief/recognition register. 4th or 5th driving origin. Requires dedicated writing effort. |
| `CisFemaleTransformed` | Female-start with transformation element. Future. |

**The current pattern for all scenes — CisMale only:**

```jinja
{% if not w.alwaysFemale() %}
    {# Cis-male-start — disorientation and alienation register. This is still new. #}
{% endif %}
```

Write transformation content inside `{% if not w.alwaysFemale() %}` blocks. If the moment
doesn't feel different for a woman who used to be a man, don't add a transformation block.
AlwaysFemale players will see the default (non-branched) prose, which should read naturally
for anyone. Do not add `{% else %}` AlwaysFemale-specific branches yet — that is a future
content pass requiring its own quality bar.

#### The Cis-Male-Start Register

The primary transformation experience. She is adjusting to a body and a social position she
did not choose. Male attention lands strangely. The mirror is a fact that needs restating.
Every gendered social interaction is a lesson she didn't ask for. The transformation is
something that happened *to* her. Writing cue: alienation, recalibration, wry observation
of what she used to be on the other side of.

---

### FEMININITY as a Writing Dial

`w.getSkill("FEMININITY")` tracks adaptation. The range is −100 to 100.
Male-start PCs begin around 0–10. Always-female PCs begin at 75.

| Range | What she's like | Example texture |
|-------|----------------|-----------------|
| < 25 | Still strongly adjusting | Female experiences feel like thresholds crossed. Sex with a man is conceptually enormous. The body still surprises her sometimes. |
| 25–49 | Conflicted | Recognises female feelings, doesn't fully own them. *Am I actually like this now?* — but less certainly than before. |
| 50–74 | Adapted, not erased | Mostly inhabits female life. Occasional flicker of her former self. The past is real but not dominant. |
| ≥ 75 | Fully adapted | Barely thinks about having been male. Don't impose transformation content here unless it's genuinely earned. |

Use `w.getSkill("FEMININITY")` directly in prose templates to branch on adaptation level:

```jinja
{% if w.getSkill("FEMININITY") < 25 %}
The mirror is still a fact that needs restating every morning.
{% elif w.getSkill("FEMININITY") < 50 %}
You've stopped flinching. Mostly.
{% else %}
You're not thinking about it.
{% endif %}
```

---

### Four Transformation Textures

**1. Insider knowledge**

She knows how men think because she was one. This gives her unusual clarity about male
behaviour — she can read what a man wants, what he's performing, what he actually means.
It can be erotic (she knows exactly how much trouble she's in), uncomfortable (she sees
the gap between what he's saying and what he's doing), or simply wry (she's watched this
play out before, from the other side).

```jinja
{% if not w.alwaysFemale() %}
She knows that look. She used to wear it.
{% endif %}
```

**2. Body unfamiliarity**

She's still, at some level, learning what this body does. Not constant commentary — but in
specific moments where something about being in this body is genuinely new. Her own
reflection. The specific vulnerability of being smaller. Having to learn her own anatomy
the way someone learns a foreign language.

```jinja
{% if not w.alwaysFemale() %}
There's still the occasional moment where her own reflection takes her slightly by surprise.
This is one of them.
{% endif %}
```

**3. Social reversal**

She used to be on the side that holds doors, pays, interrupts, takes up space. Now she's
on the other side of all of it. This can be:

- Dissonance (being talked over when she used to do the interrupting)
- Revelation (being physically protected when she used to be the one offering protection)
- Irony (experiencing exactly what she used to dish out)
- Charged eroticism (being the object of male desire when she used to be the one desiring)

**4. Desire crossover**

Male-start PCs were heterosexual before transformation. Attraction to men is genuinely new.
At low FEMININITY this can be destabilising — finding herself responding to a man physically
and not knowing what to do with that. At high FEMININITY it's simply desire, unqualified.
Calibrate by FEMININITY level.

---

### Anti-Patterns (Transformation-Specific)

| Don't write | Why |
|---|---|
| Transformation reference in every scene | Becomes noise. Reserve it for scenes where it changes something. |
| "As a former man, you..." | Clunky. Show the transformation through experience, not announcement. |
| The same transformation branch at all FEMININITY levels | A FEMININITY 10 PC and a FEMININITY 60 PC are different people. |
| Always-female players left with a blank or gap | They must always get a complete, valid path. |
| Transformation as comedy | It's not a gag. Wry is fine; slapstick is not. |
| Ignoring it when it would genuinely change the scene | The biggest failure mode — treating her as if she has no history. |

---

### When Transformation Content Is Earned

Ask: **would this moment feel different to a woman who used to be a man than to a woman
who always was one?**

If yes: write the branch.

**Scenes that almost always earn it:** male attention; body-awareness moments; being treated
as a woman in a way she'd have been invisible to before; desire for a man; first-time sexual
experiences; social dynamics where gender is active (being talked over, being protected,
being excluded).

**Scenes that often don't need it:** choosing a film, dealing with a broken appliance,
navigating a work task with no gendered dimension. Include it if it's genuinely earned,
not as a quota.

---

## Adult and Erotic Content

This is an adult game. Sexual content should be genuinely arousing — not mechanically
explicit, not clinically descriptive, and not the generic AI version of eroticism.

### What makes it work

**Tension over description.** What hasn't happened yet is more powerful than what has.
The look across the room, the hand that stops just short, the decision not yet made — these
carry more charge than a catalogue of actions.

**Desire is specific.** Not "she wants him" — *what* specifically does she want, in what
way, and how is that complicated by who she is and who he is? A SHY PC wants things she
won't say. A SULTRY PC wants things she'll say too easily. A ROMANTIC PC wants things
wrapped in meanings that might not be there. The desire is character.

**The PC's traits are present during sex.** A REFINED woman experiencing something she
finds crude doesn't stop being refined — the friction is the content. A CUTE woman in
over her head is still herself. The erotic charge often comes from the gap between who
she is and what's happening to her.

**NPC desire is specific too.** A ROMANTIC man and a JERK man both want her — but what
they want from her, how they show it, and what satisfies them are completely different.
These are not interchangeable.

**The game's core erotic logic is loss of control.** The world happens to her. She
responds. Unpredictability, the world exceeding her choices, situations she didn't invite —
this is baked into the design. Content that leans into this is more aligned than content
where she's in full control of a scripted sequence.

### Vocabulary register

Direct, non-euphemistic language for bodies and sex without being clinical. Plain English.
Avoid purple prose ("her feminine core") and the AI middle ground of vague eroticism
("she felt desire building inside her"). Write what actually happens. Name the parts
plainly. Let the situation create the charge.

### The erotic weight of non-sexual scenes

Non-sexual scenes can carry erotic charge. The street encounter she handled badly and the
specific way her body remembers it. The ex's missed text at 11:47pm. The mundane weight
of the world making claims on her. This ambient texture — the world being interested in
her, imposing itself — is part of what makes explicitly sexual scenes land when they arrive.

Don't oversell non-sexual scenes as erotic. Let the charge be ambient.

---

## Content Gating

Two hidden player traits control content gating:

| Trait | Meaning |
|-------|---------|
| `BLOCK_ROUGH` | Player opt-out: no rough or non-consensual content |
| `LIKES_ROUGH` | Player preference: include rougher paths when available |

**Standard three-level pattern:**

```jinja
{% if w.hasTrait("LIKES_ROUGH") %}
The most intense version — she's into this and it shows.
{% elif not w.hasTrait("BLOCK_ROUGH") %}
The default rough version — pushed past comfortable, not entirely unwanted.
{% else %}
The clean alternative — fully written, nothing implicit. Never a blank.
{% endif %}
```

**Two-level pattern (when there's no LIKES_ROUGH gradient):**

```jinja
{% if not w.hasTrait("BLOCK_ROUGH") %}
The darker path.
{% else %}
A complete alternative. Not a shorter version of the same scene. A genuinely different
path that works without the darker content.
{% endif %}
```

**Rules:**
- The `{% else %}` path must always be fully written — never a blank, never a one-liner
  that implies the same thing happened without describing it
- VANILLA and SEXUAL content needs no gating at all
- Gate at the TOML action `condition` field when the entire action is ROUGH/NONCON:

```toml
condition = "!w.hasTrait('BLOCK_ROUGH')"
```

**Content levels:**

| Level | Description | Gate required |
|-------|-------------|---------------|
| VANILLA | No sexual content | None |
| SEXUAL | Consensual sex or explicit content | None |
| ROUGH | Consensual rough/BDSM elements | `BLOCK_ROUGH` opt-out |
| DUBCON | Ambiguous consent | `BLOCK_ROUGH` opt-out |
| NONCON | Non-consensual | `BLOCK_ROUGH` opt-out |

A single scene can span multiple levels across different paths. Tag the scene by its
maximum level and note which paths require gating.

---

## Scene Design Principles

### The core question

Ask of every scene: **does this feel like something that happens TO the player, or
something the player does?**

We want the first. The world interrupts. Events have consequences the player didn't choose.
NPCs have their own agendas. The player is a person in a city that has its own life.

### Anatomy of a good scene

1. **An inciting situation** — something happens before the player decides anything.
   The world moves first.

2. **1–3 genuine choices** — not cosmetic. Different paths must produce genuinely different
   outcomes and consequences.

3. **At least one lasting consequence** — a game flag, NPC stat change, or PC stat change.
   Something that means this moment existed. The world must be able to remember.

4. **Trait coverage that changes the situation** — 2–4 player traits that genuinely alter
   what happens, not what adjective describes it.

### Consequence persistence

Every scene must change at least one of:

- A world flag (`gd.hasGameFlag("FLAG")`) — world memory, persists across sessions
- An NPC stat (liking, love) — relationship memory
- A scene flag (`scene.hasFlag("FLAG")`) — within-scene state only

Scenes that leave no trace did not happen. The world cannot feel real if it has amnesia.

**Repeat-visit variation:** If a player might encounter the same location or NPC twice,
use a game flag to vary the text on return:

```toml
# In scene TOML, set a flag on first encounter
[[actions.effects]]
type = "set_game_flag"
flag = "MET_DAVID"
condition = "!gd.hasGameFlag('MET_DAVID')"
```

Then branch in prose:
```jinja
{% if not gd.hasGameFlag("MET_DAVID") %}
He introduces himself. David, he says, like you should already know who that is.
{% else %}
David again. He gives you the nod of someone who expects you to be pleased.
{% endif %}
```

### What makes a moment memorable

A scene with three of these five qualities is a good scene:

1. **Specificity** — a particular detail that couldn't be about anyone else or anywhere else
2. **Consequence** — something changed; the world is different after this moment
3. **Unpredictability** — the world surprised her; she had to respond to something she didn't expect
4. **Character revelation** — different traits lead to genuinely different self-knowledge
5. **World texture** — the world around her was doing something independent

---

## Scene Authorship Checklist

Before submitting any scene, verify:

**Design:**
- [ ] Does something happen in the intro before the player makes any choice?
- [ ] Are there 1–3 choices where different paths produce genuinely different outcomes?
- [ ] Does at least one path set a lasting game flag or NPC/PC stat?
- [ ] Is the inciting situation something that happens TO her, not something she chose?
- [ ] Does the world behave as if it has its own life, independent of her?

**Prose:**
- [ ] All trait branches are structurally different (not adjective swaps)
- [ ] American English throughout — no British spellings or references
- [ ] Second-person present tense throughout
- [ ] Sentence structure varies (not every line starting with "You")
- [ ] No emotion announcements, no heart/pulse clichés, no generic NPC dialogue
- [ ] NPC dialogue reflects that NPC's personality and current goal, not a generic type

**Transformation:**
- [ ] Does this scene earn a transformation branch? If yes, is it written for CisMale→Woman?
- [ ] Transformation content inside `{% if not w.alwaysFemale() %}` blocks only
- [ ] No `{% else %}` AlwaysFemale-specific branches (deprioritized — future content pass)
- [ ] Transformation content calibrated to appropriate FEMININITY range (not one-size)?
- [ ] No `TRANS_WOMAN` inner branches (deprioritized — future content pass)

**Content gating:**
- [ ] Is the content level tagged (VANILLA / SEXUAL / ROUGH / DUBCON / NONCON)?
- [ ] All ROUGH/DUBCON/NONCON prose wrapped in `BLOCK_ROUGH` gate?
- [ ] Every gated `{% else %}` path fully written (not blank, not implied)?

**Technical:**
- [ ] All conditions referencing traits use the correct trait ID (matches `traits.toml`)?
- [ ] All conditions referencing skills use the correct skill ID (matches `skills.toml`)?
- [ ] Minijinja template validates without errors?
