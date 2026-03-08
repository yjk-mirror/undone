# Prolific Writing Session — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Write ~18 new scenes across 4 tracks (Jake romance, Marcus tension, stranger encounters, content deepening) to prove the game's explicit adult premise and expand post-arc content.

**Architecture:** Each scene is a TOML file in `packs/base/scenes/`. Scenes are written by the `scene-writer` custom agent, audited by `writing-reviewer`, then fixed and committed by the lead. Schedule integration happens in `packs/base/data/schedule.toml`. Character docs go in `docs/characters/`.

**Tech Stack:** TOML scene files, minijinja templates, custom expression conditions, scene-writer/writing-reviewer agents.

**Session type:** Autonomous. Dispatch scene-writer agents in parallel batches of 3-4. Run writing-reviewer on each result. Fix Criticals. Commit per batch.

---

## Required Reading (for every scene-writer agent)

Every scene-writer agent MUST read these before writing:
- `docs/writing-guide.md` — full prose rules
- `docs/writing-samples.md` — Sample 0 is the calibration target
- `docs/creative-direction.md` — creative bible
- `docs/content-schema.md` — TOML schema reference

Additionally, each agent reads the specific reference scenes listed in its task.

## Flag Progression Chains

These are the game flags that gate scene availability. New scenes MUST use these consistently.

**Jake chain:**
```
MET_JAKE (coffee_shop) → JAKE_FIRST_DATE → JAKE_SECOND_DATE → JAKE_INTIMATE
```

**Marcus chain:**
```
FIRST_MEETING_DONE (workplace_work_meeting) → MET_MARCUS (implicit from ROLE_MARCUS)
  → MARCUS_REAL_CONVERSATION (work_marcus_coffee) → MARCUS_DRINKS → MARCUS_INTIMATE
```

**Stranger encounters:** No chain. Gated on `gd.week()` only.

**Landlord chain:**
```
MET_LANDLORD (workplace_landlord) → LANDLORD_REPAIR
```

## NPC Role IDs

Existing: `ROLE_JAKE`, `ROLE_MARCUS`
New: None needed — landlord uses `MET_LANDLORD` flag (not an NPC with liking).

---

## Task 0: Jake Character Doc

**Files:**
- Create: `docs/characters/jake.md`

**Step 1: Read Jake's existing scenes**

Read these files to extract Jake's personality, appearance, and speech patterns:
- `packs/base/scenes/coffee_shop.toml`
- `packs/base/scenes/coffee_shop_return.toml`
- `packs/base/scenes/jake_outside.toml`

**Step 2: Write the character doc**

Based on what the scenes establish:
- Name: Jake
- Age: ~30 (about your age, maybe a year or two older)
- Appearance: Good shoulders, jaw that gets noticed, doesn't look at his phone
- Personality: Easy, unhurried, doesn't perform. Smiles without calculating. Remembers you.
- Speech: Dry, occasional humor, doesn't over-talk. "Hey." is his opening every time.
- What he represents: The "she chose this" register. Tenderness, not urgency. A man who is straightforward in a world that keeps ambushing her.
- Confidence: HIGH — established clearly across 3 scenes.

**Step 3: Commit**

```bash
git add docs/characters/jake.md
git commit -m "docs: add Jake character profile from existing scenes"
```

---

## Task 1: Marcus Character Doc

**Files:**
- Create: `docs/characters/marcus.md`

**Step 1: Read Marcus's existing scenes**

- `packs/base/scenes/work_marcus_coffee.toml`
- `packs/base/scenes/work_marcus_favor.toml`
- `packs/base/scenes/workplace_work_meeting.toml` (where he first appears)

**Step 2: Write the character doc**

Based on what the scenes establish:
- Name: Marcus
- Role: Senior colleague at the tech company
- Appearance: Extract from scenes (likely not heavily described — infer what's there)
- Personality: Direct, professional, observant. Asks real questions. Not a flirt — the tension comes from proximity and competence
- What he represents: The "this wasn't supposed to happen" register. Professional context makes everything riskier. The attraction is situational, not romantic.
- Key dynamic: He sees her as competent (unlike Dan/Kevin). That recognition is itself attractive.
- Confidence: MEDIUM — personality is clear, physical description is sparse.

**Step 3: Commit**

```bash
git add docs/characters/marcus.md
git commit -m "docs: add Marcus character profile from existing scenes"
```

---

## Batch 1: Foundation Scenes (parallel: 3 scene-writer agents)

### Task 2: jake_first_date

**Files:**
- Create: `packs/base/scenes/jake_first_date.toml`

**Reference scenes:** `coffee_shop.toml`, `coffee_shop_return.toml`, `jake_outside.toml`
**Reference docs:** `docs/characters/jake.md`

**Scene spec:**
- **Setting:** A restaurant or bar. Evening. He asked, she said yes. First time they're together intentionally.
- **Inciting situation:** She's already here. Or she arrives and he's already here. The transition from "acquaintance" to "date" is the tension — the same man in a different context.
- **Transformation angle:** Physical proximity at a small table. His hand near hers. Scale difference — she's small next to him in a way she wasn't at a coffee counter. Involuntary responses to his voice, his attention, the way he looks at her when she talks.
- **Trait axes (pick 2-3):**
  - SHY: struggles with eye contact, orders too quickly, silence feels enormous
  - FLIRTY: controls the pace, teases, comfortable with the charged air
  - ANALYTICAL: reads every signal, catalogs every micro-expression, can't stop processing
  - CONFIDENT: direct, asks real questions, doesn't hide behind small talk
- **FEMININITY branches:**
  - <25: Physical disorientation — his hand on her back when she sits, how loud her heartbeat is, the dress/outfit feels like a costume
  - 25-50: Responses are faster than processing — she laughs before deciding to, leans in before noticing
- **Actions (3-4):**
  - "Let him lead the conversation" — he talks, she listens, the evening unfolds. Sets JAKE_FIRST_DATE.
  - "Ask him something real" — she steers it somewhere honest. Different conversation. Sets JAKE_FIRST_DATE + extra NPC liking.
  - "Touch his hand" (FLIRTY or FEMININITY>30 gate) — deliberate physical escalation. Sets JAKE_FIRST_DATE + highest NPC liking.
  - "Make an excuse and leave" — she's not ready. Ends scene without JAKE_FIRST_DATE. Does NOT lock out future dates — just not tonight.
- **Effects:** `set_game_flag("JAKE_FIRST_DATE")`, `add_npc_liking(ROLE_JAKE, 1-2)`, `skill_increase(FEMININITY, 3)`, `change_stress(-2)`
- **Confidence:** HIGH

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::jake_first_date"
weight    = 0
trigger   = "gd.week() >= 3 && gd.hasGameFlag('MET_JAKE') && !gd.hasGameFlag('JAKE_FIRST_DATE') && gd.npcLiking('ROLE_JAKE') == 'Like'"
once_only = true
```
(Goes in `free_time` slot)

**Step 1: Dispatch scene-writer agent**

Provide the scene spec above plus required reading docs. The agent writes the full TOML.

**Step 2: Run writing-reviewer**

Audit the output. Fix all Critical findings.

**Step 3: Validate template syntax**

Use `mcp__minijinja__jinja_validate_template` on all prose fields.

**Step 4: Run validate-pack**

```bash
cargo run --release --bin validate-pack
```

---

### Task 3: work_marcus_late

**Files:**
- Create: `packs/base/scenes/work_marcus_late.toml`

**Reference scenes:** `work_marcus_coffee.toml`, `work_marcus_favor.toml`
**Reference docs:** `docs/characters/marcus.md`

**Scene spec:**
- **Setting:** Office, 8 PM. Most people gone. She's finishing something. Marcus is still here too.
- **Inciting situation:** He appears at her desk/doorway. Not planned — he was also working late. The building is quiet. The usual professional distance contracts when the context changes.
- **Transformation angle:** Workplace at night feels different in this body. The emptiness that used to mean "productive quiet time" now has texture — awareness of being alone in a building with a man she finds attractive. Not fear — just awareness. A frequency she never tuned to before.
- **Trait axes:**
  - AMBITIOUS: she was genuinely working, the interruption is unwelcome until it isn't
  - ANALYTICAL: notices the shift in how he talks when no one else is around — less careful, more direct
  - SHY: the sudden intimacy of being the only two people is overwhelming
- **FEMININITY branches:**
  - <30: His voice sounds different in the empty office. Lower? Or she's just closer. Something warm behind her sternum.
  - 30-50: She knows what this feeling is. Knowing doesn't help.
- **Actions (3):**
  - "Keep working" — he sits nearby, they work in parallel. The proximity is enough. Sets MARCUS_LATE_NIGHT.
  - "Take a break together" — vending machine, hallway, a conversation that wouldn't happen during business hours. Sets MARCUS_LATE_NIGHT + NPC liking.
  - "Call it a night" — she leaves. Professional boundary held. Sets MARCUS_LATE_NIGHT but no liking boost.
- **Effects:** `set_game_flag("MARCUS_LATE_NIGHT")`, `add_npc_liking(ROLE_MARCUS, 0-2)`, `skill_increase(FEMININITY, 2)`
- **Confidence:** HIGH — workplace tension is well-established territory

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::work_marcus_late"
weight    = 8
condition = "gd.arcState('base::workplace_opening') == 'settled' && gd.hasGameFlag('FIRST_MEETING_DONE') && gd.npcLiking('ROLE_MARCUS') == 'Ok'"
```
(Goes in `work` slot)

---

### Task 4: bar_closing_time

**Files:**
- Create: `packs/base/scenes/bar_closing_time.toml`

**Reference scenes:** `neighborhood_bar.toml` (same bar, different night, different vibe)

**Scene spec:**
- **Setting:** McAllister's again. But it's 11:30 PM, not 7 PM. Different crowd, different lighting, different energy. She's been here a while or just arrived — the intro can establish either.
- **Inciting situation:** A man she hasn't seen before. Not the drink-offer guy from the earlier scene — someone new. He's closing his tab. She's closing hers. The timing puts them walking out at the same moment. The walk home is two blocks.
- **Transformation angle:** Walking home at night in this body. The city at 11:30 PM is a different city. Not threatening — charged. Awareness of his presence behind/beside her. The body's response to a stranger's proximity in the dark. This is the "world exceeding her choices" register.
- **Trait axes:**
  - CONFIDENT: walks at her own pace, if he's going to say something he can say it
  - SHY: hyperaware of the footsteps behind her, the gap between what she wants and what she'll allow
  - FLIRTY: slows down. Lets the moment happen.
  - BEAUTIFUL: he's already looked. The question is what happens with it.
- **FEMININITY branches:**
  - <25: The walk home used to be nothing. Two blocks. Now two blocks is a negotiation with a body that wants things she hasn't agreed to.
  - 25-50: She knows the walk. She knows what her body does on it. The stranger is a variable she didn't plan for.
- **Actions (3-4):**
  - "Walk home. Don't look back." — She hears him behind her. He peels off to his own street. The relief has an aftertaste. No flag set.
  - "Slow down" — He catches up. They walk together. Conversation. At her door, the question hangs. Sets BAR_STRANGER_WALKED.
  - "Invite him for a drink" (CONFIDENT or FLIRTY or FEMININITY>35) — At her door. One more drink. What happens after that is the next scene. Sets BAR_STRANGER_INVITED. This is the gateway to explicit content.
  - "Tell him to go" — Direct. He goes. She watches him go. Something she can't name. Sets BAR_STRANGER_MET but no invitation.
- **Effects:** Various flags. `skill_increase(FEMININITY, 3)`, `change_stress(-1 or +1 depending on path)`
- **NPC:** This stranger is NOT a persistent NPC. No role assignment. He's the world acting.
- **Confidence:** HIGH for setup, MEDIUM for the explicit follow-up scene (Task 12)

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::bar_closing_time"
weight    = 6
condition = "gd.week() >= 3 && !gd.hasGameFlag('BAR_STRANGER_INVITED')"
```
(Goes in `free_time` slot)

---

### Task 5: Batch 1 commit

**Step 1: Add all Batch 1 scenes to schedule.toml**

Add entries per the specs above to the appropriate slots.

**Step 2: Run validate-pack**

```bash
cargo run --release --bin validate-pack
```

**Step 3: Commit**

```bash
git add packs/base/scenes/jake_first_date.toml packs/base/scenes/work_marcus_late.toml packs/base/scenes/bar_closing_time.toml packs/base/data/schedule.toml
git commit -m "content: batch 1 — jake_first_date, work_marcus_late, bar_closing_time"
```

---

## Batch 2: Escalation Scenes (parallel: 4 scene-writer agents)

### Task 6: jake_second_date

**Files:**
- Create: `packs/base/scenes/jake_second_date.toml`

**Reference scenes:** `jake_first_date.toml` (from Batch 1), `jake_outside.toml`

**Scene spec:**
- **Setting:** His suggestion this time. Something casual — a walk, a market, his neighborhood. Less formal than the first date. Daylight or dusk.
- **Inciting situation:** Comfort has arrived. Not full comfort — the kind where the nervousness has a warm edge instead of a sharp one. He touches her arm when making a point and doesn't pull back.
- **Transformation angle:** Physical contact that she chose to be available for. His hand on her lower back. The difference between coffee-shop-Jake (safe distance) and this-Jake (inside her personal space with her permission). How the body responds to invited touch vs. the surprise touches of daily life.
- **Trait axes:**
  - ROMANTIC: lets herself imagine forward. What this could become.
  - ANALYTICAL: notices herself responding and can't stop cataloging it. Is this real or is this the body?
  - SHY: the physical closeness is simultaneously wanted and overwhelming
- **FEMININITY branches:**
  - <30: His hand on her back and her brain just stops. Three seconds of nothing. Then it comes back and she's leaning into him.
  - 30-50: She knows she's going to kiss him before it happens. The knowing is new. The wanting isn't.
- **Actions (3):**
  - "Kiss him" — She initiates. The first kiss in this body with someone she chose. Sets JAKE_SECOND_DATE.
  - "Let him kiss you" — He reads the moment. She lets it happen. Different agency, same result. Sets JAKE_SECOND_DATE.
  - "Not yet" — Close. Almost. But not yet. The anticipation is its own thing. Sets JAKE_SECOND_DATE but no kiss flag.
- **Effects:** `set_game_flag("JAKE_SECOND_DATE")`, optional `set_game_flag("JAKE_KISSED")`, `add_npc_liking(ROLE_JAKE, 2)`, `skill_increase(FEMININITY, 3)`
- **Confidence:** HIGH

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::jake_second_date"
weight    = 0
trigger   = "gd.week() >= 4 && gd.hasGameFlag('JAKE_FIRST_DATE') && !gd.hasGameFlag('JAKE_SECOND_DATE')"
once_only = true
```

---

### Task 7: work_marcus_drinks

**Files:**
- Create: `packs/base/scenes/work_marcus_drinks.toml`

**Reference scenes:** `work_marcus_late.toml` (from Batch 1), `work_marcus_coffee.toml`

**Scene spec:**
- **Setting:** A bar near the office. After work. Not planned — he suggested it as they were leaving. "I could use a drink, you?" The professional context extends into social space.
- **Inciting situation:** Two drinks in. The conversation has drifted from work to something else. He's leaned back, arm over the booth. She's noticed his forearm. The professional filter is dissolving with the alcohol.
- **Transformation angle:** Alcohol in this body hits different. Faster. Warmer. The inhibitions she built as a man had a different architecture — alcohol loosens a different set of bolts now. She's aware of her body's response to his proximity in a way that the office suppresses.
- **Trait axes:**
  - AMBITIOUS: this is a colleague. The calculus includes career consequences.
  - CONFIDENT: she's here because she wanted to be. No pretense.
  - ANALYTICAL: tracking her own blood alcohol, his body language, the probability of this going somewhere
- **FEMININITY branches:**
  - <30: Two drinks and the professional armor has gaps. His laugh and something loosens in her chest. She hasn't felt this particular warm-drunk before.
  - 30-50: She knows what two drinks does to her now. She came anyway.
- **Actions (3-4):**
  - "Have another round" — The evening extends. The conversation gets closer. His knee against hers under the table. Sets MARCUS_DRINKS.
  - "Split the check" — Professional boundary reasserted. They walk to the subway together. Something unresolved. Sets MARCUS_DRINKS but lower liking.
  - "Tell him about yourself" — Real conversation. Not work. She says something true and watches him recalibrate. Sets MARCUS_DRINKS + extra liking.
  - "Touch his hand" (FLIRTY or FEMININITY>35) — Under the table or on it. A signal. He doesn't pull away. Sets MARCUS_DRINKS + MARCUS_TOUCHED.
- **Effects:** `set_game_flag("MARCUS_DRINKS")`, `add_npc_liking(ROLE_MARCUS, 1-3)`, `skill_increase(FEMININITY, 2)`, `change_alcohol(1)`
- **Confidence:** HIGH

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::work_marcus_drinks"
weight    = 0
trigger   = "gd.arcState('base::workplace_opening') == 'settled' && gd.hasGameFlag('MARCUS_LATE_NIGHT') && !gd.hasGameFlag('MARCUS_DRINKS') && gd.npcLiking('ROLE_MARCUS') == 'Like'"
once_only = true
```
(Goes in `work` slot — it's an after-work event but gated on work context)

---

### Task 8: party_invitation

**Files:**
- Create: `packs/base/scenes/party_invitation.toml`

**Reference scenes:** `neighborhood_bar.toml`, `bar_closing_time.toml`

**Scene spec:**
- **Setting:** A house party. A coworker invited her — not Marcus, someone more peripheral. An apartment somewhere in the city. Music, people, alcohol, the particular energy of 20 strangers in a living room.
- **Inciting situation:** She's here and doesn't know many people. The social dynamics of a party as a woman are different from what she knew. The attention is different. The approach vectors are different. Someone offers her a drink. Someone else corners her in the kitchen.
- **Transformation angle:** Party dynamics as a woman. The way groups form around her. Men positioning themselves. Women sizing her up. The body navigating a room full of people who are reading her in ways she used to be on the other side of. Alcohol accelerating everything.
- **Trait axes:**
  - OUTGOING: works the room, finds her footing, enjoys the energy
  - SHY: kitchen wall, one person at a time, the noise is a lot
  - BEAUTIFUL: the attention is constant and comes from every direction
  - OBJECTIFYING: she reads the room like she used to work a room — knows exactly who's looking and what they want, because she was them
- **FEMININITY branches:**
  - <25: Every interaction is a puzzle with a body she doesn't know the rules of. A man touches her waist passing by and she freezes for a full second.
  - 25-50: She's learning the physics. Where to stand, how to hold a drink, when eye contact is an invitation.
- **Actions (3-4):**
  - "Find the quietest corner" — A conversation with one person. Possibly the most interesting person at the party. No explicit content.
  - "Dance" (OUTGOING or CONFIDENT or FEMININITY>30) — The living room is a dance floor. Bodies close. Music loud. Surrender to it. Sets PARTY_DANCED.
  - "Follow him outside" — A specific man. Balcony or rooftop or stoop. The party noise muffled. Just the two of you. Sets PARTY_STRANGER_OUTSIDE. Gateway to explicit content.
  - "Leave early" — She's had enough. The walk home is its own scene-within-a-scene.
- **Effects:** Various flags. `skill_increase(FEMININITY, 3)`, `change_alcohol(1)`, `change_stress(varies)`
- **Confidence:** HIGH for setup, MEDIUM for prose quality (lots of moving parts)

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::party_invitation"
weight    = 5
condition = "gd.week() >= 4"
once_only = true
```

---

### Task 9: weekend_morning

**Files:**
- Create: `packs/base/scenes/weekend_morning.toml`

**Reference scenes:** `morning_routine.toml`, `evening_home.toml`

**Scene spec:**
- **Setting:** Saturday morning. No alarm. The apartment in weekend light. The first morning she doesn't have to be anywhere.
- **Inciting situation:** Slow waking. The body is warm and heavy and the sheets feel different against skin she's still mapping. No urgency. The morning is hers.
- **Transformation angle:** The body at rest. No social performance required. What she notices when there's no audience. The shower (tactile), getting dressed (or not), the mirror without urgency. This is the "private body" register — distinct from every other scene's "public body."
- **Trait axes:**
  - ROMANTIC: lingering, dreamy, the morning has a quality to it
  - ANALYTICAL: even alone she's cataloging. The body as a system she's learning.
  - AMBITIOUS: can't sit still. Plans the day. The body is the thing she's in while she works.
- **FEMININITY branches:**
  - <25: The shower is still a daily surprise. Water on skin she doesn't recognize. Hair that takes twenty minutes.
  - 25-50: The routine is forming. She knows how long the hair takes. She has products now. The surprise is how much she cares.
- **Actions (3):**
  - "Stay in bed" — The luxury of nowhere to be. What happens in her head. Sets no flag.
  - "Take a long shower" — The private body scene. Tactile, sensory, intimate with herself. Sets WEEKEND_SHOWER. Potential gateway to solo-explicit content.
  - "Go out for breakfast" — The apartment is too quiet. She wants the city. Transitions to a cafe. Sets WEEKEND_OUT.
- **Effects:** `skill_increase(FEMININITY, 2)`, `change_stress(-3)`
- **Confidence:** HIGH — contained, personal, clear transformation angle

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::weekend_morning"
weight    = 10
condition = "gd.week() >= 2 && !gd.isWeekday()"
```

---

### Task 10: Batch 2 commit

Same as Batch 1: add schedule entries, validate, commit.

```bash
git add packs/base/scenes/jake_second_date.toml packs/base/scenes/work_marcus_drinks.toml packs/base/scenes/party_invitation.toml packs/base/scenes/weekend_morning.toml packs/base/data/schedule.toml
git commit -m "content: batch 2 — jake_second_date, work_marcus_drinks, party_invitation, weekend_morning"
```

---

## Batch 3: Explicit Scenes (parallel: 3-4 scene-writer agents)

These are the adult content scenes. The game's premise depends on these being good.

### Task 11: jake_apartment

**Files:**
- Create: `packs/base/scenes/jake_apartment.toml`

**Reference scenes:** `jake_second_date.toml`, all Jake scenes
**Reference docs:** `docs/characters/jake.md`

**Scene spec:**
- **Setting:** Her apartment or his. Evening. After enough dates that this was coming. The question isn't whether — it's how.
- **Inciting situation:** They're inside. The door closes. The social performance of dating drops away. It's just them and the thing they've been building toward.
- **Transformation angle:** THIS IS THE CORE SCENE. First sexual experience in this body with someone she chose. Everything is new — what arousal feels like, what his hands feel like, what her body does that she didn't know it would do. She had a male body's response map. This body has a completely different one. Discovery, not performance.
- **CRITICAL WRITING RULES for this scene:**
  - The body leads, the mind follows. She discovers responses she didn't know she had.
  - Never clinical, never mechanical. Physical specifics without medical language.
  - His perspective is invisible — we only know what she feels and sees.
  - Tenderness register. Jake is not aggressive. He's attentive. He notices her responses.
  - The transformation texture is PHYSICAL: different nerve endings, different arousal curve, different pleasure geography. She is mapping a body she didn't grow up in.
  - FEMININITY level determines how much is surprise vs. anticipation.
- **Trait axes:**
  - SHY: vulnerable, hesitant, every touch is a decision
  - CONFIDENT: direct about what she wants (even when she doesn't fully know yet)
  - FLIRTY: this is what she's been building toward, she's ready
  - ANALYTICAL: can't stop thinking even now, but the body wins
- **FEMININITY branches:**
  - <25: Everything is new. His mouth on her neck and her brain whites out. She makes a sound she's never heard herself make. The arousal is a tide — not the sharp spike of before but something that builds and builds and doesn't stop building.
  - 25-50: She knows what she wants. Getting there is still a negotiation between knowing and this body's particular route.
- **Actions (3-4):**
  - "Let him set the pace" — He's gentle, attentive. She discovers through his lead. EXPLICIT. Sets JAKE_INTIMATE.
  - "Show him what you want" (CONFIDENT or FEMININITY>30) — She guides his hands. Different explicit path. Sets JAKE_INTIMATE.
  - "Stop. Not tonight." — Close. So close. But not tonight. The wanting is real. She's choosing to wait. No flag set beyond JAKE_SECOND_DATE.
  - "Kiss him and see what happens" — No plan. Just this moment. Wherever it goes. EXPLICIT. Sets JAKE_INTIMATE.
- **Effects:** `set_game_flag("JAKE_INTIMATE")`, `add_npc_liking(ROLE_JAKE, 3)`, `skill_increase(FEMININITY, 5)`, `add_arousal`, `change_stress(-5)`
- **Confidence:** MEDIUM — tone is critical. The difference between good and bad here is the difference between the game working and not. Writing-reviewer must be especially thorough.

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::jake_apartment"
weight    = 0
trigger   = "gd.week() >= 5 && gd.hasGameFlag('JAKE_SECOND_DATE') && !gd.hasGameFlag('JAKE_INTIMATE')"
once_only = true
```

---

### Task 12: bar_stranger_night

**Files:**
- Create: `packs/base/scenes/bar_stranger_night.toml`

**Reference scenes:** `bar_closing_time.toml` (from Batch 1)

**Scene spec:**
- **Setting:** Her apartment. After inviting the stranger from the bar in. One drink becomes two. The apartment she knows. The man she doesn't.
- **Inciting situation:** He's in her space. This isn't Jake — there's no history, no tenderness arc. This is pure situation. A man she met an hour ago sitting on her couch. The thing she's about to do is something the person she was a month ago couldn't have imagined.
- **Transformation angle:** The "unpredictability" register. No emotional investment, just the body and what it wants. The difference between this and jake_apartment is intent — Jake was a choice built over time. This is a choice made in an evening. The body's response is the same. The framing is completely different. This tests whether desire in this body is about the person or about the sensation.
- **CRITICAL WRITING RULES:**
  - This is the "loss of control" register from creative direction. The world exceeds her choices.
  - She invited him. She's not a victim. But what happens after that invitation exceeds what she planned.
  - The body responds to a stranger differently than to someone she knows. More raw. Less layered.
  - Minimal dialogue. Physical. Present-tense. The prose should move fast.
- **Trait axes:**
  - CONFIDENT: she's here because she wants to be. No apology.
  - SHY: this should barely be available to SHY characters — if gated, gate at FEMININITY>40
  - FLIRTY: she set this up. She knows the mechanics even if the sensations are new.
- **Actions (3):**
  - "Pull him close" — She initiates. EXPLICIT. Sets BAR_STRANGER_SLEPT.
  - "Let him make the move" — She waits. He reads the room. EXPLICIT. Sets BAR_STRANGER_SLEPT.
  - "Change your mind" — The door. Thank him for walking her home. Close it. Alone. The wanting doesn't go away. It's just hers now.
- **Effects:** `set_game_flag("BAR_STRANGER_SLEPT")`, `skill_increase(FEMININITY, 4)`, `add_arousal`, `change_stress(varies)`
- **Confidence:** MEDIUM — hardest tone to nail. Must not be exploitative. Must not be romantic. Must be honest.

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::bar_stranger_night"
weight    = 0
trigger   = "gd.hasGameFlag('BAR_STRANGER_INVITED') && !gd.hasGameFlag('BAR_STRANGER_SLEPT')"
once_only = true
```

---

### Task 13: work_marcus_closet

**Files:**
- Create: `packs/base/scenes/work_marcus_closet.toml`

**Reference scenes:** `work_marcus_drinks.toml`, `work_marcus_late.toml`
**Reference docs:** `docs/characters/marcus.md`

**Scene spec:**
- **Setting:** The office. A conference room. A stairwell. Somewhere that is NOT a bedroom. The professional context makes everything hotter and more dangerous.
- **Inciting situation:** After Marcus drinks. The tension that's been building. They're alone in the office or near it. One of them says something. The other doesn't break eye contact. The line blurs.
- **Transformation angle:** Authority and desire in a professional context. She has earned her position. This attraction threatens it. The body doesn't care about professional consequences. The contrast between her competence in the meeting room and her body's response to his proximity three feet away.
- **CRITICAL WRITING RULES:**
  - This is NOT romantic. It's situational and urgent.
  - The professional setting is part of the charge — the risk of being seen/heard.
  - His competence attracted her. Her competence attracted him. The mutual recognition is what got them here.
  - No aftermath in this scene. That's the next scene.
- **Trait axes:**
  - AMBITIOUS: the career risk is real and she's doing it anyway
  - CONFIDENT: she closes the distance
  - ANALYTICAL: she knows exactly what this means for work on Monday
- **FEMININITY branches:**
  - <30: His hand on her hip and the professional version of herself just leaves. The body takes over and it is nothing like the controlled, systematic person she is in the meeting room.
  - 30-50: She knew this was coming since the bar. She came to work today knowing this was possible.
- **Actions (3):**
  - "Close the door" — She initiates. Conference room or office. EXPLICIT. Sets MARCUS_INTIMATE.
  - "Let him" — He moves first. She doesn't stop him. EXPLICIT. Sets MARCUS_INTIMATE.
  - "Walk away" — The professional self wins. This time. The tension remains. Sets MARCUS_ALMOST but no INTIMATE.
- **Effects:** `set_game_flag("MARCUS_INTIMATE")`, `add_npc_liking(ROLE_MARCUS, 3)`, `skill_increase(FEMININITY, 4)`, `add_arousal`, `change_stress(+2)` — stress increases because workplace.
- **Confidence:** MEDIUM — workplace explicit content needs to avoid cliche

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::work_marcus_closet"
weight    = 0
trigger   = "gd.arcState('base::workplace_opening') == 'settled' && gd.hasGameFlag('MARCUS_DRINKS') && !gd.hasGameFlag('MARCUS_INTIMATE')"
once_only = true
```

---

### Task 14: shopping_mall

**Files:**
- Create: `packs/base/scenes/shopping_mall.toml`

**Reference scenes:** `workplace_first_clothes.toml`, `grocery_store.toml`

**Scene spec:**
- **Setting:** A mall or shopping district. She needs something — clothes, shoes, something for the apartment. Mundane errand that becomes a body-awareness scene.
- **Inciting situation:** Dressing rooms. Mirrors. Sales associates who call her "miss" without thinking. The public performance of femininity in a consumer space designed for it. She's buying things for a body she's still learning.
- **Transformation angle:** The fitting room mirror. Trying on clothes that are FOR this body. Not the emergency shopping of workplace_first_clothes (necessity) — this is choosing. The difference between "I need clothes that fit" and "I want to look like..." The shift from survival to preference.
- **Trait axes:**
  - POSH: she has opinions. Strong ones. The saleswoman can't keep up.
  - SHY: the fitting room is a confrontation. Every outfit is a question about who she is now.
  - BEAUTIFUL: the mirror confirms something. She watches herself in a way she doesn't when no one is looking.
  - DOWN_TO_EARTH: just needs jeans. Why is this so complicated.
- **FEMININITY branches:**
  - <25: The fitting room is a box with a mirror and a stranger's body. Every outfit asks a question she doesn't have an answer to.
  - 25-50: She's developing preferences. She reaches for things without thinking about it. That's new.
- **Actions (3):**
  - "Try the dress" — Something she wouldn't have considered a month ago. The mirror and a decision.
  - "Stick to basics" — Jeans, shirts, comfortable. No performance. Practical.
  - "Ask the saleswoman" — Surrender expertise to someone who does this. The vulnerability of admitting she doesn't know how to dress herself.
- **Effects:** `change_money(-1)`, `skill_increase(FEMININITY, 2)`, `change_stress(-2)`
- **Confidence:** HIGH — contained, clear angle, distinct from workplace_first_clothes

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::shopping_mall"
weight    = 7
condition = "gd.week() >= 3"
```

---

### Task 15: Batch 3 commit

Same pattern. Add schedule entries, validate, commit.

```bash
git add packs/base/scenes/jake_apartment.toml packs/base/scenes/bar_stranger_night.toml packs/base/scenes/work_marcus_closet.toml packs/base/scenes/shopping_mall.toml packs/base/data/schedule.toml
git commit -m "content: batch 3 — jake_apartment, bar_stranger_night, work_marcus_closet, shopping_mall

First explicit adult scenes. Jake (tenderness), stranger (unpredictability),
Marcus (workplace transgression)."
```

---

## Batch 4: Resolution + Aftermath (parallel: 4 scene-writer agents)

### Task 16: jake_morning_after

**Files:**
- Create: `packs/base/scenes/jake_morning_after.toml`

**Scene spec:**
- **Setting:** Morning. His place or hers. Waking up next to someone.
- **Inciting situation:** She wakes up and he's there. Or he wakes up and she's been awake for a while, watching the ceiling, processing. The aftermath of intimacy. The body is sore in new places. The light is different.
- **Transformation angle:** Post-intimacy body. What she notices first. The physical evidence of last night. His arm across her. The smell of him on her skin. The question isn't "did I enjoy it" — the body answered that. The question is "what does this mean for who I am now."
- **Actions (3):**
  - "Stay" — Morning with him. Coffee. The beginning of something.
  - "Get up quietly" — She needs to process alone. Leaves before he wakes or while he's in the shower.
  - "Wake him" — She's not done. Morning intimacy. A different register than last night — lighter, more playful.
- **Effects:** `skill_increase(FEMININITY, 3)`, `add_npc_liking(ROLE_JAKE, 1-2)`, `change_stress(-3)`
- **Confidence:** HIGH

**Schedule entry:** Trigger on JAKE_INTIMATE, once_only.

---

### Task 17: work_marcus_aftermath

**Files:**
- Create: `packs/base/scenes/work_marcus_aftermath.toml`

**Scene spec:**
- **Setting:** Monday morning. The office. After what happened.
- **Inciting situation:** She walks in. He's already at his desk. The normality of the office and the abnormality of what happened in it. Does he look at her? Does she look at him? The other people in the office don't know.
- **Transformation angle:** The professional mask and what's behind it. She had a professional identity as a man. She built a new one as a woman. The Marcus thing cracks it. Managing desire and reputation simultaneously.
- **Actions (3):**
  - "Act normal" — Professionalism. The mask holds. But there's a moment in the hallway.
  - "Find him" — She needs to know where they stand. Direct conversation. Not about feelings — about logistics.
  - "Avoid him" — Not ready. Reroutes through the day to minimize contact.
- **Effects:** `skill_increase(FEMININITY, 2)`, `change_stress(+2 or -2 depending on path)`, `set_game_flag("MARCUS_AFTERMATH")`
- **Confidence:** HIGH

**Schedule entry:** Trigger on MARCUS_INTIMATE, once_only, work slot.

---

### Task 18: jake_text_messages

**Files:**
- Create: `packs/base/scenes/jake_text_messages.toml`

**Scene spec:**
- **Setting:** Her apartment. Evening. Phone buzzing. Jake.
- **Inciting situation:** Text conversation. The intimacy of a phone screen at night. What she types, what she deletes, what she sends. The gap between thinking and expressing.
- **Transformation angle:** Digital intimacy in a new body. The phone is the one space where the body doesn't matter — but it does. She's choosing words as a woman texting a man she's sleeping with. The patterns are different from how she texted as a man.
- **Format note:** This scene uses a different structure. The "prose" includes text message formatting. Use the `{% raw %}` or direct text to show messages. Not a full scene — more of an interlude.
- **Actions (3):**
  - "Send the honest text" — Says what she's thinking. Vulnerable.
  - "Keep it light" — Flirty, casual. Don't let him see the processing.
  - "Don't reply tonight" — The phone on the nightstand. The glow. She'll reply tomorrow.
- **Effects:** `add_npc_liking(ROLE_JAKE, 1)`, `skill_increase(FEMININITY, 1)`
- **Confidence:** MEDIUM — format is unusual for the engine. May need creative adjustment.

**Schedule entry:** Gated on JAKE_INTIMATE, not once_only (can recur), low weight.

---

### Task 19: landlord_repair

**Files:**
- Create: `packs/base/scenes/landlord_repair.toml`

**Scene spec:**
- **Setting:** Her apartment. The landlord is here to fix something — faucet, radiator, door. Her domestic space invaded by the man who owns it.
- **Inciting situation:** He called ahead. She let him in. Now he's in her bathroom/kitchen with tools and she's standing in her own apartment feeling like a guest. The power dynamic is spatial — he's fixing HER space, he has the key, he decides when things get repaired.
- **Transformation angle:** Domestic vulnerability. A man in her private space. Not threatening — but the awareness of the power differential. She pays rent. He decides the maintenance schedule. In her old life she'd have fixed it herself. Now she's watching a man fix her faucet and the dynamics are different.
- **Trait axes:**
  - CONFIDENT: maintains her space. Directs the repair. "While you're here, the window sticks too."
  - SHY: retreats to the other room. The sounds of him in her bathroom.
  - ANALYTICAL: notes the power structure. Files it. Doesn't like it.
  - DOWN_TO_EARTH: offers him coffee. Normal interaction. Just a repair.
- **Actions (3):**
  - "Watch him work" — Stay in the room. Conversation about the building, the neighborhood.
  - "Leave him to it" — Go to the other room. The sounds of a man in your apartment.
  - "Help" — She knows plumbing. The surprised look on his face. Gender expectation collision.
- **Effects:** `set_game_flag("LANDLORD_REPAIR")`, `skill_increase(FEMININITY, 1)`, `change_stress(varies)`
- **NOT an explicit scene.** The power dynamic is the content. Tension, not action.
- **Confidence:** MEDIUM — landlord personality needs to be established. Not a romantic interest.

**Schedule entry:**
```toml
[[slot.events]]
scene     = "base::landlord_repair"
weight    = 5
condition = "gd.week() >= 3 && gd.hasGameFlag('MET_LANDLORD') && !gd.hasGameFlag('LANDLORD_REPAIR')"
once_only = true
```

---

### Task 20: Batch 4 commit

```bash
git add packs/base/scenes/jake_morning_after.toml packs/base/scenes/work_marcus_aftermath.toml packs/base/scenes/jake_text_messages.toml packs/base/scenes/landlord_repair.toml packs/base/data/schedule.toml
git commit -m "content: batch 4 — aftermath scenes + landlord_repair + jake_text_messages"
```

---

## Batch 5: Expand Existing Scenes + Stretch Goals

### Task 21: Expand workplace_work_meeting (add actions)

**Files:**
- Modify: `packs/base/scenes/workplace_work_meeting.toml`

Currently has 1 action ("Present the design"). Add 2 more:
- "Stay quiet" — Let Kevin present. Watch him get it wrong. The room's reaction tells you something about the hierarchy.
- "Correct him" (CONFIDENT or ANALYTICAL) — Interrupt. Point out the error. The room recalibrates. Consequences.

Each new action needs its own prose, effects, and transformation texture.

**Confidence:** HIGH — expanding existing structure is safe.

---

### Task 22: Expand workplace_evening (add actions)

**Files:**
- Modify: `packs/base/scenes/workplace_evening.toml`

Currently has 1 action ("Continue"). Add 2 more:
- "Run a bath" — The bathroom as a decompression space. Physical. Tactile.
- "Call someone" — Who does she call? The before-life intrudes. No one to call who knows what happened.

**Confidence:** HIGH

---

### Task 23: laundromat_night (stretch goal)

**Files:**
- Create: `packs/base/scenes/laundromat_night.toml`

**Scene spec:**
- **Setting:** Laundromat. 10 PM. The fluorescent light. The machines. She's alone — then she's not.
- **Inciting situation:** A man. Waiting for his clothes. The enforced proximity of a small space at night. He's not threatening. He's just there. And the body has opinions.
- **Transformation angle:** Mundane space charged by the body. The hum of the machines. The warmth. The way she's hyperaware of him in a way that has nothing to do with danger and everything to do with the body she's in.
- **Actions (3):**
  - "Ignore him" — Earbuds. Phone. The dryer cycle. But she's aware.
  - "Talk to him" — Laundromat small talk. Where do you live. Long have you been in the neighborhood.
  - "Sit closer" (FLIRTY or FEMININITY>35) — The bench has room. She chooses proximity.
- **NOT explicit.** Tension only. The domestic mundane as charged space.
- **Confidence:** MEDIUM — needs a strong hook to not be boring.

---

### Task 24: Batch 5 commit

```bash
git add packs/base/scenes/workplace_work_meeting.toml packs/base/scenes/workplace_evening.toml
# Add laundromat if written
git add packs/base/data/schedule.toml
git commit -m "content: batch 5 — expand single-action scenes + laundromat_night"
```

---

## Final Tasks

### Task 25: Full schedule integration pass

Review `packs/base/data/schedule.toml` holistically:
- Verify all new scenes have entries
- Check condition chains (flag dependencies resolve correctly)
- Check weights balance (Jake/Marcus/stranger scenes don't crowd out universal scenes)
- Ensure once_only is set on all narrative-progression scenes
- Ensure trigger vs weight is correct (mandatory beats = trigger, optional variety = weight)

### Task 26: Run validate-pack

```bash
cargo run --release --bin validate-pack
```

Fix any warnings.

### Task 27: Run cargo test

```bash
cargo test
```

All 262+ tests must pass.

### Task 28: Final commit and HANDOFF update

Update HANDOFF.md with:
- Session log entry
- Updated scene count
- Per-scene confidence ratings
- List of scenes needing user creative review
- Updated remaining priorities

```bash
git add -A
git commit -m "docs: update HANDOFF with prolific writing session results"
```

---

## Confidence Summary

| Scene | Track | Confidence | Notes |
|---|---|---|---|
| jake_first_date | Jake | HIGH | Natural progression |
| jake_second_date | Jake | HIGH | Clear arc step |
| jake_apartment | Jake | MEDIUM | Tone-critical explicit scene |
| jake_morning_after | Jake | HIGH | Aftermath is good writing territory |
| jake_text_messages | Jake | MEDIUM | Unusual format |
| work_marcus_late | Marcus | HIGH | Established territory |
| work_marcus_drinks | Marcus | HIGH | Clear escalation |
| work_marcus_closet | Marcus | MEDIUM | Workplace explicit needs care |
| work_marcus_aftermath | Marcus | HIGH | Consequences are good |
| bar_closing_time | Stranger | HIGH | Setup scene, well-defined |
| bar_stranger_night | Stranger | MEDIUM | Must not be exploitative |
| party_invitation | Stranger | HIGH/MEDIUM | Lots of moving parts |
| weekend_morning | Deepening | HIGH | Contained, clear |
| shopping_mall | Deepening | HIGH | Distinct from existing |
| landlord_repair | Deepening | MEDIUM | Landlord personality undefined |
| laundromat_night | Stretch | MEDIUM | Needs strong hook |
| workplace_work_meeting | Expand | HIGH | Adding to existing |
| workplace_evening | Expand | HIGH | Adding to existing |

**Scenes requiring extra user review:** jake_apartment, bar_stranger_night, work_marcus_closet (all MEDIUM, all explicit)

---

## Execution Command

```
Use `ops:executing-plans` to implement the plan at `docs/plans/2026-03-08-prolific-writing-plan.md`
```
