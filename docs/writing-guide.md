# Undone — Writing Guide

The prose standard for all content in this project. Read before writing a single line of text.

---

## Purpose

This guide applies to all scene prose in any pack. Every rule here applies equally whether
you are writing a solo street encounter or an intimate scene. The engine exists to serve
the writing. The writing exists to make the world feel real.

---

## Voice and Register

**The narrator is a dungeon master.** Second-person, present tense, sitting on the player's
shoulder. It points things out, describes what's happening, and then asks "what do you do?"
It has personality — dry, occasionally wry — but it serves the player's experience. It never
performs, never shows off, never tries to be literature.

**Reference voice: the BG3 narrator.** Amelia Tyler described it as "chin on the player's
shoulder, whispering 'look at that,' tilting their chin but never manipulating them." That's
the register. Present. Specific. Grounded. A guide, not an author.

The narrator is **not**:

- A novelist crafting prose (no artful sentences, no atmospheric filler)
- Omniscient (it doesn't know what people are thinking, or the bartender's life story)
- A therapist analyzing the player's experience ("none of this was conscious")
- Literary or self-consciously artistic
- Pornographic in euphemism ("his throbbing need")
- Clinical or mechanical

**This is a game, not a novel.** The player will read thousands of these passages over hours
of play. The prose needs to be readable, engaging, and fast. The engagement comes from the
situations and choices — from what's happening and what you get to do about it — not from
beautiful sentences. Write well, but write to be played, not to be admired.

**Practical test:** Read the prose aloud as if you were a DM narrating to a player at a table.
If it sounds like you're reading from a novel, rewrite it. If it sounds like you're telling
someone what's happening right now and then asking what they do, it's right.

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
His hand closes over yours and for a second your brain just — blanks. Like a word you know
but can't find.
*Huh*, you think. And then you're back.
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
| `gd` | `hasGameFlag("FLAG")`, `week()`, `day()`, `timeSlot()`, `isWeekday()`, `isWeekend()`, `arcState("arc_id")`, `arcStarted("arc_id")`, `npcLiking("ROLE")` |
| `scene` | `hasFlag("FLAG")` |

> **Note:** `m` (male NPC) and `f` (female NPC) are available in **action and NPC-action
> prose only** — NOT in intro prose, intro_variants, or thoughts. NPC bindings are not
> established until after scene start. To vary intro prose based on NPC state, use
> `gd.npcLiking("ROLE_X")` which reads from persistent world state.

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
| `m.` | `hasTrait('ID')`, `isPartner()`, `isFriend()`, `hadOrgasm()`, `isNpcLoveCrush()`, `getLiking()`, `getLove()`, `getAttraction()`, `getBehaviour()`, `hasFlag('FLAG')`, `hasRole('ROLE')` |
| `f.` | `isPartner()`, `isFriend()`, `isPregnant()`, `isVirgin()`, `hasFlag('FLAG')`, `hasRole('ROLE')` |
| `gd.` | `hasGameFlag('FLAG')`, `week()`, `getStat('ID')`, `isWeekday()`, `isWeekend()`, `arcStarted('arc_id')`, `arcState('arc_id')`, `npcLiking('ROLE_X')`, `getJobTitle()` |
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
| `OUTGOING` | Approaches people, fills silences. | Initiates conversations. Comfortable in crowds. Opposite of SHY. |
| `ANALYTICAL` | Observes patterns, thinks before acting. | Notices inconsistencies. Internal monologue runs heavy. |
| `CONFIDENT` | Self-assured, doesn't second-guess. | Takes up space. Makes decisions fast. Doesn't apologize for existing. |
| `SEXIST` | Internalized misogyny from the before-life. | Judges women by male standards. Catches herself doing it. Self-awareness optional. |
| `HOMOPHOBIC` | Discomfort with same-sex attraction. | Male attention registers as desire before shame kicks in. The conflict is the content. |
| `OBJECTIFYING` | Evaluates bodies automatically. | Notices proportions, attractiveness. The male gaze she used to own, now turned inward. |

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

### 11. Overused words and phrases

AI prose gravitates to the same vocabulary across scenes. Ration these aggressively —
at most once per scene, ideally once per several scenes:

| Word/phrase | Problem | Fix |
|---|---|---|
| "specific" / "specifically" | The #1 AI-prose tell. Empty intensifier. | Say the specific thing instead of calling it specific. |
| "something about" | Vague hand-wave | Name the something or show it |
| "the way" / "in the way that" | Filler connector | Cut or restructure the sentence |
| "a quality" / "a certain" | Empty frame | Name it or show it |
| "you notice" / "you realize" | Filters through observation | Just state the thing directly |
| "somehow" | Vague qualifier | Explain how or don't qualify |
| "deliberate" / "deliberately" | Tells intent | Show the action; let intent be inferred |
| "something shifts" | Vague atmospheric gesture | What shifted? Show it. |
| "the weight of" (metaphorical) | Abstract. Overused for gravity/significance. | Find a concrete detail that carries the weight instead. |

**Post-writing test:** Search the scene for these words. More than 2–3 hits means
rewriting those sentences. Scenes should not sound like each other.

### 12. Scene distinctiveness across the game

Before writing, answer: **what makes this scene impossible to confuse with any other
scene in the game?** If the answer relies on the location name, it's not distinct enough.

Two bar scenes should feel like completely different experiences — different NPCs,
different dynamics, different stakes. If you swapped the location names and nothing
broke, they're the same scene.

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

**Transformation prose IS the prose. No guards needed.**

Since we are only writing CisMale→Woman content, transformation texture is woven directly
into the prose. Do NOT wrap it in `{% if not w.alwaysFemale() %}` blocks. That pattern
cluttered the code and broke the flow of the writing. Just write it.

Use `{% if not w.alwaysFemale() %}` ONLY for content that would be genuinely wrong or
nonsensical for an AlwaysFemale player (e.g., before-body comparisons using `w.beforeName()`).
For the vast majority of transformation texture — physical sensations, body awareness, the
world treating you differently — write it directly. It reads naturally for any player.

AlwaysFemale-specific content is a future pass requiring its own dedicated quality bar.
Do not write `{% else %}` branches.

#### How transformation works in the calibrated register

Transformation is physical and immediate. The narrator doesn't analyze it, doesn't explain
it, doesn't name it. It just describes what the player experiences.

**Good:** "The stool takes a small hop to get onto. The bar top comes up higher than you
expect and your hands look small against the wood."

**Bad:** "You notice you're doing the thing. The shoulders angled forward, the posture that
reads as *occupied*. You didn't learn this from a manual. The armor went up without you
deciding to put it on."

The first example is a physical fact the player notices. The second is the narrator
analyzing the player's body language and explaining where it came from. The narrator
doesn't know where it came from. The player connects those dots.

**The rule:** If the narrator is explaining the transformation, the scene hasn't shown it
yet. Find the physical moment. Write that. Delete the explanation.

---

### FEMININITY as a Writing Dial

`w.getSkill("FEMININITY")` tracks adaptation. The range is 0 to 100.
`CisMaleTransformed` PCs begin at 10. Always-female PCs begin at 75.

## FEMININITY Intervals (0-100)

- **0–9**: Total alienation. This body is a stranger's body. She doesn't know how to walk in it. Breasts are foreign objects strapped to her chest. Every sensation is wrong. Being perceived as female is disorienting. Arousal happens in a body she doesn't recognize.
- **10–19**: First adaptations. She's stopped reaching for a cock that isn't there. Female pronouns don't make her flinch every time. She's starting to understand why women cross their arms. Her reflection looks almost familiar if she doesn't look too long.
- **20–29**: Functional. She passes. She's learning the rhythms — the way women move through space, the social shorthand, the danger calculus that operates below conscious thought. Male attention is uncomfortable but no longer alien. She's had her first orgasm in this body and it confused her for days.
- **30–39**: Adapting. Catches herself thinking like a woman before correcting to thinking like a man pretending to be a woman. The correction comes later every week. She owns clothes she chose because she liked how they looked on her. She's felt desire in this body and didn't hate it.
- **40–49**: The tipping point. The before-life is something that happened to someone else. She still knows she was a man — the knowledge is there — but it's *knowledge*, not *identity*. She looks in the mirror and sees *herself*. The gender dysphoria runs the other way now — imagining going back feels wrong.
- **50–59**: Comfortable. Being a woman is normal. She has female friendships, female routines, female complaints. Transformation flickers are rare — a man's gesture that reminds her of who she used to be, a phantom memory during sex. She knows her body. She knows what she likes.
- **60–69**: Settled. Only extreme gendered situations bring the before-life forward — being catcalled and recognizing the man she would have been, encountering a situation where her male history gives her unexpected insight. These moments are bittersweet, not traumatic.
- **70–79**: Native with residue. She doesn't think about having been male in daily life. The transformation is biographical — something that happened, like being born in a different city. Sex is sex. Desire is desire. But sometimes, in the right moment, the *knowing* surfaces and makes everything sharper. She understands men in a way other women can't, and that knowledge is power.
- **80–89**: The before-life is academic. She was a man the way someone was a child — technically true, experientially distant. Her body is *her* body. No flickers. No phantom memories. The transformation gave her life context but doesn't define it. The richest writing here is about what she's *built*, not what she lost.
- **90–100**: Complete. The transformation is a fact about her past, not a lens on her present. She is a woman. Full stop. Writing at this tier should not reference the transformation unless something extreme forces it — a blood test, a magical echo, meeting someone from the before-life. At this level, the character is post-transformation. The game continues because life continues.

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

### Physical Attributes

The PC has a full set of physical attributes available for use in scene prose. These are
set at character creation and remain stable unless explicitly changed by a scene effect.

**Physical enums (accessible via template methods):**

| Attribute | Accessor | Notes |
|-----------|----------|-------|
| Age | `w.getAge()` | |
| Race | `w.getRace()` | |
| Height | `w.getHeight()` | |
| HairLength | `w.getHairLength()` | |
| SkinTone | `w.getSkinTone()` | |
| Complexion | `w.getComplexion()` | |
| ButtSize | `w.getButt()` | |
| WaistSize | `w.getWaist()` | |
| LipShape | `w.getLips()` | |
| NippleSensitivity | `w.getNippleSensitivity()` | |
| ClitSensitivity | `w.getClitSensitivity()` | |
| PubicHairStyle | `w.getPubicHair()` | |
| InnerLabiaSize | `w.getInnerLabia()` | |
| WetnessBaseline | `w.getWetness()` | |
| EyeColour | `w.getEyeColour()` | |
| HairColour | `w.getHairColour()` | |
| PlayerFigure | `w.getFigure()` | 7 variants |
| BreastSize | `w.getBreasts()` | 7 variants |
| Appearance | `w.getAppearance()` | Plain → Devastating (6 tiers) |
| NaturalPubicHair | `w.getNaturalPubicHair()` | Bare → Heavy (5 tiers) |
| ActiveName | `w.getName()` | Selects masc/fem name by FEMININITY |
| SmoothLegs | `w.hasSmoothLegs()` | true if NATURALLY_SMOOTH or SMOOTH_LEGS |
| PenisSize | `w.beforePenisSize()` | On BeforeIdentity only |

**Before-body accessors** (what she looked like before transformation — use inside
`{% if not w.alwaysFemale() %}` blocks only):

| Attribute | Accessor |
|-----------|----------|
| Height | `w.beforeHeight()` |
| HairColour | `w.beforeHairColour()` |
| EyeColour | `w.beforeEyeColour()` |
| SkinTone | `w.beforeSkinTone()` |
| PenisSize | `w.beforePenisSize()` |
| Figure | `w.beforeFigure()` |
| Name | `w.beforeName()` |
| Voice | `w.beforeVoice()` | High, Average, Deep, VeryDeep |

**Usage notes:**
- Use physical attributes to add specificity to body-awareness moments — but sparingly.
  One well-placed physical detail grounds a scene. A catalogue of them reads like a
  character sheet.
- Before-body accessors are for transformation contrast only. A scene comparing her
  current body to her before-body earns the detail. A scene that isn't about
  transformation does not.
- At low FEMININITY (0–29), physical unfamiliarity is high. Breast size, sensitivity,
  wetness — these are things she's still learning about. Use accordingly.
- At high FEMININITY (60+), physical attributes are simply *hers*. No contrast needed.

---

### Four Transformation Textures

All transformation writing follows one rule: **describe the physical experience. Don't
explain it.** The narrator reports what happens in the body. The player connects the dots.
No meta-commentary, no "none of this was conscious," no analyzing what the body is doing.

Write these directly in prose — no `{% if not w.alwaysFemale() %}` guards needed.

**1. The body acts first**

Physical reactions the player didn't choose. Write the sensation, not the analysis.

Good: "His hand closes over yours on the railing. Your stomach drops."
Good: "Something loosens between your hips when his hand lets go. Faint, warm."
Bad: "Your body responding to the proximity of a man who leaned in close and smells like
something your brain is filing under *want* before your brain gets to review the filing."

The bad example is the narrator analyzing the body's reaction in real time. The good
examples just report what happened. The reader does the analysis.

**2. Scale and space**

The world is physically different when you're smaller. Write the fact, not the commentary.

Good: "The stool takes a small hop to get onto. The bar top comes up higher than you expect."
Good: "The coffee cup looks enormous in your hands."
Bad: "You're learning the positioning math — where to sit, how to sit, the specific body
language that reads as 'occupied' vs 'available.'"

The bad example is the narrator explaining a social system. The good examples are just
physical facts the player notices.

**3. Desire arriving uninvited**

Attraction to men is new. Write the desire concretely — what specifically, what it does
to the body — before any reaction to the desire.

Good: "He smells like beer and something warmer underneath. Cologne, maybe, or just the
way men smell up close. You're close enough to notice this, which means he's closer than
you realized."
Bad: "You have an opinion about the particular frequency of his voice and how it sits in
your body and the opinion is *more of that, please*."

The bad example is the narrator putting a fully articulated desire-thought in the player's
head. The good example describes proximity and smell — the player supplies the desire.

**4. Intrusive fragments**

A thought that pops up and goes. Half-formed, not a thesis.

Good: "*Huh.*"
Good: "The mirror behind the bottles catches you and for a second you look at the woman
in it before you realize she's looking back."
Bad: "You never made these calculations before. You never had to. Your body is making them.
You're just watching it work."

The bad example is a three-sentence essay about body autonomy. The good examples are
moments that happen and pass.

---

### Anti-Patterns (Transformation-Specific)

| Don't write | Why |
|---|---|
| "You used to do this" / "You know what he's doing" / "You were on the other side" | **The #1 failure mode.** Preachy, moralistic, repetitive. Show the physical reaction, not intellectual commentary. |
| "None of this was conscious" / "Your body is making calculations" / "The armor went up without you deciding" | **The #2 failure mode.** The narrator analyzing what the body is doing instead of just describing it. Write the physical fact. Delete the analysis. |
| "the opinion is *more of that, please*" / "*I'm here and I'm fine*" / any full thought in italics | **The #3 failure mode.** Putting fully articulated thoughts in the player's head. Inner voice should be fragments (*Huh.* / *Okay.* / *Right.*), not sentences. |
| Narrating the transformation explicitly: "as a former man", "you remember being a man" | The reader knows. The PC knows. Spelling it out kills it. |
| The same beat in every scene (the male gaze, "you recognize this") | Each scene must find its OWN angle. No two scenes should use the same device. |
| Abstract meta-framing: "there's something about being in a body you didn't choose" | Name the specific thing or don't write the line. |
| The same transformation branch at all FEMININITY levels | A FEMININITY 10 PC and a FEMININITY 60 PC are different people. |
| Always-female players left with a blank or gap | They must always get a complete, valid path. |
| Transformation as comedy | Wry is fine; slapstick is not. |

---

### When Transformation Content Is Earned

Ask: **would this moment feel different to a woman who used to be a man than to a woman
who always was one?**

If yes: write the branch.

**Scenes that almost always earn it:** male attention that registers in the body; physical
sensation she has no reference for; desire arriving uninvited; the body being a different
size/shape/weight than expected; being touched and the touch landing differently; arousal
that surprises her.

**Scenes that often don't need it:** choosing a film, dealing with a broken appliance,
navigating a work task with no gendered dimension. Include it if it's genuinely earned
through a physical or emotional moment — not as an intellectual observation about gender
dynamics.

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

### Player agency — the intro/action split

This is the most important structural rule in the game. Get it wrong and the player stops
feeling like they're playing.

**The intro describes the world. Actions are what the player decides to do.**

The intro puts the player in a situation: where she is, what's around her, what's happening
to her, who's there. It can describe men looking at her, rain starting, a coworker saying
something that changes the room. The world acts freely. But the intro never decides what
the player does — not what she orders, not how she sits, not what she says, not what she
thinks.

Actions are the player's choices. Every `[[actions]]` entry is a button. When the player
clicks it, that's her decision. The action prose describes what happens as a result of her
choice, including dialogue, consequences, and how the world responds.

**What the intro CAN do:**
- Describe the room, the people, the atmosphere
- Show men reacting to her presence (BEAUTIFUL branch)
- Show her body experiencing the space (stool too tall, bar top at chest height)
- Set up tension (someone is watching, the bartender is waiting, a situation is developing)
- End with something that invites a decision

**What the intro CANNOT do:**
- Order her a drink
- Decide how she sits or positions herself consciously
- Put full thoughts in her head ("*I'm here and I'm fine and I haven't decided yet*")
- Have her speak to anyone
- Analyze what her body is doing ("none of this was conscious," "you're doing the thing")
- Explain her motivations ("which is what you came here for")

**What actions MUST do:**
- Lead somewhere meaningful. Every action should produce real consequences, open new paths,
  or change the situation. "Check your phone" is not an action — it's filler. If an action
  can be summarized as "you do the thing and nothing changes," cut it.
- Branch deeply on traits. A SHY character accepting a drink from a stranger is a completely
  different scene from a FLIRTY character doing it. The trait should change what happens,
  who says what, and what it leads to.
- Create decision chains when appropriate. "Order a drink" can lead to "what kind?" which
  leads to a conversation with the bartender. Break decisions into beats — each one a moment
  of player agency.

**Dialogue in action prose is fine.** When the player picks "Accept the drink," writing what
she says is expected — she chose to engage, and the prose shows what that looks like for this
character. The line is between the player *choosing to act* (agency) and the writer *deciding
she acts* (stolen agency).

This is a game. Every moment is "what do you do?"

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

### The scene must earn its place

Ask of every scene and every action: **what is the one thing that happens here that
couldn't happen anywhere else?**

If the answer is "a woman goes to a park / bar / grocery store and notices it's different
now" — that's not a scene. That's a description. Every location can produce that.

**Scenes must be intentional, deep, and richly branched.** If a scene doesn't go somewhere
meaningful — if it doesn't change the world, develop a relationship, create a decision with
real consequences — it doesn't exist. Half-assed scenes are worse than no scenes. They
train the player to stop caring about what happens.

**Positive requirements:**

1. **One irreplaceable detail** — a specific person, event, or exchange pinned to this
   time and place. The kind of detail that only exists in THIS scene.

2. **At least one beat of unresolved tension or desire** — not everything resolves to
   "and that was fine." Something slightly open when the scene ends.

3. **Transformation as physical experience** — not referenced, not announced. The body
   reacts. The player connects the dots.

**The "fine" test:** If you can summarize an action as "she did the thing and it was fine,"
the action needs work.

### No filler actions

Every action must lead somewhere. If an action doesn't produce real consequences, open
new paths, or change the situation, cut it. Examples of filler:

- "Check your phone" — nothing happens. Not an action.
- "Look around the room" — this is description, not a choice. Put it in the intro.
- "Wait" — unless waiting causes something to happen (NPC approaches, situation develops).
  A "wait" that just produces ambient description is filler.

Good actions create decision chains: "Order a drink" leads to a conversation with the
bartender. "Accept the drink from him" leads to him sitting down, which leads to how you
handle his proximity. Each beat is a moment of player agency that matters.

**The depth test:** Can you trace a path from this action through at least one more
decision point? If the action is a dead end with no consequences, it's filler.

---

## Scene Authorship Checklist

Before submitting any scene, verify:

**Design:**
- [ ] Does something happen in the intro before the player makes any choice?
- [ ] Does the intro ONLY describe the world? (No deciding player actions, ordering drinks, choosing how to sit)
- [ ] Does the intro end by handing control to the player? (Bartender waiting, someone approaching, situation set up)
- [ ] Are there 1–3 choices where different paths produce genuinely different outcomes?
- [ ] Does at least one path set a lasting game flag or NPC/PC stat?
- [ ] Is the inciting situation something that happens TO her, not something she chose?
- [ ] Is every action intentional? (No filler — every action leads somewhere meaningful)
- [ ] Do actions create decision chains where appropriate? (Order → what kind → conversation)

**Register:**
- [ ] Does the prose read like a DM narrating, not a novelist crafting?
- [ ] No narrator meta-commentary ("none of this was conscious," "you're doing the thing")
- [ ] No full articulated thoughts put in the player's head
- [ ] No narrator explaining player motivations ("which is what you came here for")
- [ ] No omniscient details the player couldn't know
- [ ] American English throughout — no British spellings or references
- [ ] Second-person present tense throughout
- [ ] Sentence structure varies (not every line starting with "You")

**Prose quality:**
- [ ] All trait branches are structurally different (not adjective swaps)
- [ ] Trait branches go deep — different scenes unfold, not different adjectives
- [ ] No emotion announcements, no heart/pulse clichés, no generic NPC dialogue
- [ ] NPC dialogue reflects that NPC's personality and current goal
- [ ] At least one beat of unresolved tension or desire
- [ ] At least one specific detail that is irreplaceable — pinned to this place, person, moment

**Transformation:**
- [ ] Transformation texture written directly in prose (no `{% if not w.alwaysFemale() %}` guards unless using before-body accessors)
- [ ] Transformation is physical and immediate, not analyzed or explained
- [ ] No `{% else %}` AlwaysFemale-specific branches
- [ ] Calibrated to appropriate FEMININITY range (not one-size)
- [ ] No `TRANS_WOMAN` inner branches

**Content gating:**
- [ ] Is the content level tagged (VANILLA / SEXUAL / ROUGH / DUBCON / NONCON)?
- [ ] All ROUGH/DUBCON/NONCON prose wrapped in `BLOCK_ROUGH` gate?
- [ ] Every gated `{% else %}` path fully written (not blank, not implied)?

**Technical:**
- [ ] All conditions referencing traits use the correct trait ID (matches `traits.toml`)?
- [ ] All conditions referencing skills use the correct skill ID (matches `skills.toml`)?
- [ ] Minijinja template validates without errors?
