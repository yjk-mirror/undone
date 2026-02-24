# Writing Samples — Quality Calibration

> Read this before touching any scene. These are approved prose excerpts that define
> the target register. Every new scene should match or exceed the quality demonstrated here.
>
> See also: `docs/writing-guide.md` for the full checklist and rules.

---

## Sample 1: Airport arrival (Robin) — transformation grounding

**Demonstrates:** Transformation content that is earned, not announced. The body as a
recurring fact, not a dramatic revelation. The world's indifference. Wry narrator distance.

---

*The seat belt sign clicks off before she's ready.*

*She's been awake since — she does the math — thirty-one hours. The man in 14C reclined
into her space somewhere over Ohio and she didn't say anything. She would have, before.
The thought arrives without context.*

*The flight attendant crouches at the row ahead, smiling at something the woman there
said. The aisle is narrow and people are already standing before they're supposed to,
pulling bags from overheads with the focused expression of people who have not slept
enough and have somewhere to be.*

*She stands.*

*The first thing she notices is the weight — she keeps forgetting and remembering the
weight, a rhythm she's been doing since Saturday morning and apparently hasn't stopped
doing. Forget. Stand up. Remember. The carry-on is in the overhead. She reaches for it
and a man behind her reaches past her and takes it down.*

*"Got it," he says, and hands it to her.*

*He means it kindly. She knows this. She says thank you and means it, and files the
interaction somewhere, and keeps moving.*

*Through the jetbridge. Into the terminal. The airport is large and bright and indifferent
in the specific way airports are: it has been processing people since before she got here
and will continue after. The signs for Ground Transportation are adequate. She follows them.*

*At the exit checkpoint, the agent looks at her ID. Then at her. The ID is correct. The
face in the photo is a white man in his early thirties.*

*"I know," she says, before he can figure out the sentence. "I just look young."*

*The line behind her is long. He waves her through. She walks out into the city.*

---

**Annotation:** The transformation content (the weight she keeps forgetting) appears as a
recurring background fact, not an announcement. The interaction with the helpful man
is noted without editorializing — she files it. The airport scene is the world, not her
feelings. The ID beat is compressed to three lines: it's a solved problem, not a crisis.

---

## Sample 2: Rain shelter intro — the city has its own life

**Demonstrates:** World texture. Trait-branching that changes what happens (not adjective
swaps). Transformation interiority that comes *after* the scene sets up, not before.
Second-person voice.

*(From `packs/base/scenes/rain_shelter.toml`)*

---

> The sky opened up three blocks from your apartment. Not a drizzle — the kind of rain that
> turns gutters into rivers and sends people running for awnings with their bags over their
> heads. A delivery driver on a moped blasts through a puddle that soaks your left side from
> the hip down.
>
> The bus shelter on Clement Ave is the nearest cover. Glass walls, metal bench, an ad for a
> personal injury lawyer peeling off the back panel. There's a man already in there — mid-twenties,
> decent jacket, one of those compact umbrellas people in this city carry because they've learned.
>
> He looks up when you step in.

*(Then trait branches — SHY, POSH, CUTE, BITCHY, default — each changing what the player does,
not just how they feel about it. Then:)*

> *[for non-alwaysFemale, cis-male-start]*
> The man glances at you — quick, automatic, the whole read done in under two seconds. You know
> that look. You've made that look, in spaces like this, more times than you've ever thought to
> count. You just never stood on the receiving end of it before.
>
> *You used to do this*, you think, *without knowing you were doing anything.*

---

**Annotation:** The city (rain, moped, puddle, the lawyer ad) exists before the player arrives.
The transformation content — the recognition of the gaze — comes *after* the scene establishes
the world, earned by the setup. The trait branches change the scene: SHY ends the scene socially;
CUTE opens a conversation; BITCHY closes one down.

---

## Sample 3: Umbrella offer — NPC action prose

**Demonstrates:** NPC action prose that feels like a complete beat. The narrator stays
outside the NPC's head. Player choice implicit in the next step.

*(From `packs/base/scenes/rain_shelter.toml`, NPC action `offer_umbrella`)*

---

> He looks at the rain. Then at you. Then at the umbrella leaning against the bench. He makes
> a decision about what kind of person he's going to be today.
>
> "Rain's not letting up. I've got this, if you want."
>
> *[if BEAUTIFUL trait]:*
> He says it like the offer was obvious, which for him it was. The umbrella is about the rain
> and also not about the rain.
>
> *[if PLAIN trait]:*
> It takes him a beat to offer — the kind of beat you might not notice if you weren't paying
> attention. But you were. You always are.

---

**Annotation:** "The thing where he looks at the rain and then at you and then at the umbrella
and then makes a decision" — this is the narrator watching behavior, not reporting feelings.
The man's interiority is inferred from his movement, not stated. The trait branch changes
what the player notices about *his* delivery, not what they feel about it.

---

## Sample 4: Inner voice at FEMININITY < 20

**Demonstrates:** Correct `inner_voice` thought style. Male-internal-monologue used
intentionally, not reflexively. The pronoun slippage as a texture, not a lesson.

---

*Okay*, she thinks — he thinks — *okay*. There is a problem and we are going to solve
the problem. The problem is that she is in a city she doesn't know, in a body she doesn't
know, with a job starting Monday, and she doesn't have any clothes that fit. These are
solvable problems. She is going to solve them in order.

*You start with bras*, she decides, and then catches the pronoun halfway through and lets
it go. The order matters more than the grammar.

---

**Annotation:** The pronoun slippage ("she thinks — he thinks") is noted and then released
without drama. "The order matters more than the grammar" — this is her voice, not the narrator
commenting on her voice. The thought style is `inner_voice`. Never `anxiety` for Robin
unless she's actually spiraling — her baseline is this pragmatic problem-inventory.

---

## Sample 5: Trait branch that changes what happens (correct)

**Demonstrates:** A branch where the trait changes the scene's events, not just the adjectives.

*Scene: Robin at the office, coworker explaining something she invented.*

*[AMBITIOUS]*
> She waits until he finishes the sentence. Then she says, "I know. I wrote the original
> spec — the one in the repo, not the presentation deck." She says it without venom.
> He stops. She moves to her desk. This is established, now, and she doesn't have to do it again.

*[SHY]*
> He finishes explaining. She says, "Got it, thanks," and writes down what he said even
> though she already knows it. Later, at her desk, she looks at the note. She knows exactly
> what's wrong with his explanation. She didn't say it. She writes a ticket in silence.

*[Default]*
> She lets him finish. "I'll take a look at it," she says, which is true, and doesn't
> volunteer that she already has. He walks away satisfied. She opens the codebase she
> already has open.

---

**Annotation:** AMBITIOUS gets a confrontation that establishes her position. SHY gets
the interaction that doesn't happen and the internal fallout. Default gets the efficient
non-event. These are three different scenes. Not three ways of feeling the same scene.

---

## Sample 6: Anti-pattern with correction

**Demonstrates:** What NOT to write, and why.

**BAD:** (adjective-swap branching, interiority instead of scene)

> *[SHY]*
> You feel nervous as you approach the counter. You're a bit embarrassed.
>
> *[CONFIDENT]*
> You feel confident as you approach the counter. You're not embarrassed at all.

**Why this fails:** Both branches have the same event (approaching the counter) and only
differ in how the player feels. The trait has no structural effect. The scene is the same.
"Feel nervous" and "feel confident" are telling, not showing.

**GOOD:** (structural difference — the trait changes what happens)

> *[SHY]*
> You take a number from the dispenser instead of walking up to the counter directly,
> and wait with three other people even though one register is open. When the clerk calls
> your number she gives you a look — not unkind, just curious — like she'd seen you standing
> there for five minutes.
>
> *[CONFIDENT]*
> You go straight to the open register. The clerk has the patience of someone who has been
> here eight hours and knows how to make twelve interactions feel like none of them.
> You get what you need in ninety seconds.

**Why this works:** SHY produces a delay, a different arrival, a social observation.
CONFIDENT produces efficiency and a specific world detail (the clerk's professional flatness).
The world responded to the trait, not just the player's feeling about it.

---

*End of calibration samples. See `docs/writing-guide.md` for full checklist.*
