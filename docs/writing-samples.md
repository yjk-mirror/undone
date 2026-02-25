# Writing Samples — Quality Calibration

> Read this before touching any scene. These are approved prose excerpts that define
> the target register. Every new scene should match or exceed the quality demonstrated here.
>
> **All prose is second-person, present tense.** No exceptions. If a sample uses "she"
> instead of "you", it is wrong and should be reported.
>
> See also: `docs/writing-guide.md` for the full checklist and rules.

---

## Sample 1: Rain shelter intro — the city has its own life

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

## Sample 2: Umbrella offer — NPC action prose

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

## Sample 3: Inner voice at FEMININITY < 20

**Demonstrates:** Correct `inner_voice` thought style. Male-internal-monologue used
intentionally, not reflexively. The pronoun slippage as a texture, not a lesson.

---

*Okay*, you think — he thinks — *okay*. There is a problem and you are going to solve
the problem. The problem is that you're in a city you don't know, in a body you don't
know, with a job starting Monday, and you don't have any clothes that fit. These are
solvable problems. You are going to solve them in order.

*Start with bras*, you decide, and then catch the pronoun halfway through and let
it go. The order matters more than the grammar.

---

**Annotation:** The pronoun slippage ("you think — he thinks") is noted and then released
without drama. "The order matters more than the grammar" — this is your voice, not the narrator
commenting on your voice. The thought style is `inner_voice`. Never `anxiety` for Robin
unless she's actually spiraling — the baseline is this pragmatic problem-inventory.

---

## Sample 4: Trait branch that changes what happens (correct)

**Demonstrates:** A branch where the trait changes the scene's events, not just the adjectives.

*Scene: First day at the office, coworker explaining something you invented.*

*[AMBITIOUS]*
> You wait until he finishes the sentence. Then you say, "I know. I wrote the original
> spec — the one in the repo, not the presentation deck." You say it without venom.
> He stops. You move to your desk. This is established, now, and you don't have to do it again.

*[SHY]*
> He finishes explaining. You say, "Got it, thanks," and write down what he said even
> though you already know it. Later, at your desk, you look at the note. You know exactly
> what's wrong with his explanation. You didn't say it. You write a ticket in silence.

*[Default]*
> You let him finish. "I'll take a look at it," you say, which is true, and you don't
> volunteer that you already have. He walks away satisfied. You open the codebase you
> already have open.

---

**Annotation:** AMBITIOUS gets a confrontation that establishes her position. SHY gets
the interaction that doesn't happen and the internal fallout. Default gets the efficient
non-event. These are three different scenes. Not three ways of feeling the same scene.

---

## Sample 5: Anti-pattern with correction

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
