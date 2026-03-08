# Player Experience Map

**Purpose:** Persistent reference documenting exactly what the player sees at every
step. Updated after content changes. Agents must consult this before claiming work
is complete.

**Last updated:** 2026-03-08

---

## Screen-by-Screen Flow (Robin Preset)

### 1. Title Screen
- "Your Story Begins" heading
- Buttons: New Game / Continue / Load / Settings
- **Status:** Clean, functional

### 2. BeforeCreation — "Who Were You?"
- Robin preset selected by default. Shows before-body summary:
  Name Robin, Age Thirties, Race White, Build Average, Height Average,
  Hair Brown, Eyes Brown, Skin tone Light, Voice Average, Penis size Average
- Personality: Ambitious, Analytical, Down to earth, Objectifying
- Button: "Next →"
- **Status:** Clean layout. Penis size shown (correct for adult game).

### 3. TransformationIntro — Plane Scene
- Stats sidebar appears (FEMININITY 10, MONEY $500, STRESS 0, etc.)
- "People Here: No one else is in focus yet."
- Prose: Gate C31, boarding pass says Robin, ROUTE_WORKPLACE motivation,
  AMBITIOUS trait branch, "They call your zone."
- Single action: "Board the flight"
- **Issues:**
  - Stats sidebar showing during what should feel like a narrative pre-game moment
  - "People Here" section is empty and cluttering

### 4. TransformationIntro — Board Action Result
- Action prose appends below intro (no visual break)
- Jet bridge, finding row, overhead bins, runway, falls asleep
- ANALYTICAL trait branch fires: calculating altitude by headlight size
- "Somewhere over Ohio, your eyes close."
- Scene finishes → transitions to FemCreation
- **Issues:**
  - No visual separator between intro prose and action prose
  - The "scene finished" transition is invisible to the player

### 5. FemCreation — "Who Are You Now?"
- Shows: Name Robin, Figure Petite, Breasts Huge, traits list, Race East Asian,
  Age Late Teen
- Button: "Begin Your Story"
- **CRITICAL ISSUES:**
  - **No prose.** The transformation — the game's reason for existing — is a data form.
    No waking up, no discovery, no "your hands are wrong."
  - **No interaction.** One button. Player doesn't explore who they are.
  - **Traits line overflows** off right edge, clipped at "Naturally sm..."
  - **"Traits" label jammed** against list: "TraitsStraight hair" — no separator
  - **This is the game's biggest gap.** Needs 4-5 interactive discovery beats.

### 6. First In-Game Scene — **BUG: Wrong Scene**
- **Shows rain_shelter instead of workplace_arrival**
- Root cause: `lib.rs:299` — `opening_scene.take()` fires before `pick_next()`.
  New games always use pack.toml's `opening_scene = "base::rain_shelter"`,
  bypassing the scheduler. The ROUTE_WORKPLACE trigger never gets a chance.
- The fix: call `pick_next()` first; only fall back to `opening_scene` when the
  scheduler returns `None`.

### 6a. Rain Shelter Scene (what the player actually sees)
- Prose flows directly from transformation_intro with **no visual break**.
  "Somewhere over Ohio, your eyes close." → "The sky opened up three blocks
  from your apartment." — looks like one continuous scene.
- **Writing issues (uncalibrated scene):**
  - *"You used to do this, you think, without knowing you were doing anything."*
    — THE banned phrase, word for word
  - "You know that look. You've made that look" — banned "you know/used to" pattern
  - "He makes a decision about what kind of person he's going to be today" —
    omniscient narrator (knows what he decided)
  - Intro decides player actions: "You slip in and lean against the glass" —
    violates intro/action split
  - Multiple em-dashes
- Round 2: 3 action buttons ("Wait it out" / "Share his umbrella" / "Decline politely")
- **No visual indicator** of what happened after choosing in round 1

### 7+ Subsequent Scenes
- Continue to append to the same scrolling prose panel
- No scene transitions, no headers, no clearing
- **Monolithic scroll** — all scenes flow together indistinguishably
- Button overflow: when 3+ action buttons, the last one can clip

---

## Expected Flow (What Should Happen with Robin)

1. Title → New Game
2. BeforeCreation → pick Robin → Next
3. TransformationIntro → plane scene → "Board the flight"
4. **[MISSING] Transformation Discovery** — 4-5 beats: wake up wrong, bathroom/mirror,
   body inventory, going public. Character creation woven into discovery.
5. FemCreation → confirm body (should be aftermath of discovery, not cold form)
6. **workplace_arrival** — airport, ID checkpoint, subway/cab → apartment
7. workplace_landlord → meet landlord, get keys
8. workplace_first_night → first night alone, body in private
9. workplace_first_clothes → clothing store
10. workplace_first_day → Dan, lunch choices
11. workplace_work_meeting → design review
12. workplace_evening → home after work
13. Settled state → rotating work + free_time scenes

---

## UI/Presentation Issues (Across All Screens)

| Issue | Severity | Description |
|-------|----------|-------------|
| No scene transitions | Critical | Scenes flow into each other with no visual break |
| Monolithic scroll | Critical | All prose appends to one scroll — no page-based pacing |
| No choice echo | Major | After choosing, no indicator of what you chose |
| No round separator | Major | Multi-round scenes have no beat transitions |
| Button overflow | Major | 3+ buttons can clip below visible area |
| Stats during narrative | Minor | Sidebar showing during TransformationIntro |
| "People Here" empty | Minor | Shows "No one else is in focus yet" — noise |

---

## Writing Issues by Scene (Uncalibrated Scenes)

Only 4 of 33 scenes have been rewritten to the DM narrator register.
The remaining 29 contain some or all of:

- **Banned "you used to" patterns** — narrator moralizing about transformation
- **Omniscient narrator** — knowing what NPCs think/decide/plan
- **Narrator thinking for player** — putting full thoughts in player's head
- **Em-dash overuse** — used as a dramatic crutch
- **Staccato closers** — isolated short sentences for dramatic effect
- **Intro/action split violations** — intro deciding player actions
- **Over-naming** — labeling experiences instead of showing them

### Scenes needing full rewrite (workplace arc):
- workplace_landlord
- workplace_first_night
- workplace_first_clothes
- workplace_work_meeting
- workplace_evening

### Scenes needing full rewrite (free_time):
- rain_shelter (contains literal banned phrase)
- morning_routine
- coffee_shop
- bookstore
- park_walk
- grocery_store
- evening_home

### Scenes needing full rewrite (work slot):
- work_standup, work_lunch, work_late, work_corridor, work_friday
- work_marcus_coffee, work_marcus_favor

### Scenes needing full rewrite (campus arc):
- All 7 campus scenes

---

## Missing Content

| Gap | Priority | Description |
|-----|----------|-------------|
| Transformation sequence | Critical | 4-5 interactive beats: waking, mirror, body, going public |
| Adult content | Critical | Zero explicit scenes. Game can't prove its premise. |
| Scene transitions in UI | Critical | Engine/presentation problem, not content |
| First-scene bug fix | Critical | `opening_scene` bypassing scheduler |
| Writing register pass | High | 29 scenes in old register |
| Campus post-arc scenes | Low | No work/activity slot for campus route |

---

## Scheduler Bug Detail

**File:** `crates/undone-ui/src/lib.rs:279-310`

**Current logic (broken):**
```
if opening_scene is Some → use it (rain_shelter)
else → call pick_next() (would return workplace_arrival)
```

**Correct logic:**
```
try pick_next() first
if pick_next() returns None AND opening_scene is Some → use opening_scene
else if neither → show "no scene available"
```

This ensures arc triggers fire before the generic opening_scene fallback.
The opening_scene is for custom-route players with no arc flags set.
