# Player Experience Map

**Purpose:** Persistent reference documenting exactly what the player sees at every
step. Updated after content changes. Agents must consult this before claiming work
is complete.

**Last updated:** 2026-03-12

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
- Full-width story presentation only. No gameplay sidebar chrome, no empty NPC panel.
- Prose: Gate C31, boarding pass, route-pressure setup, and trait-sensitive internal framing.
- Single action: "Board the flight"
- **Status:** Improved. The opening reads like authored narrative rather than a live sandbox scene.

### 4. TransformationIntro — Board Action Result
- Action prose appends below intro (no visual break)
- Jet bridge, seat, route-pressure callback, trait-sensitive in-flight beat,
  "Somewhere over Ohio, you fall asleep."
- Scene finishes → transitions to FemCreation
- **Issues:**
  - No visual separator between intro prose and action prose
  - The "scene finished" transition is invisible to the player

### 5. FemCreation — "Who Are You Now?"
- Opens with a two-paragraph transformation bridge instead of dropping straight into raw form fields.
- Shows: Name Robin, Figure Petite, Breasts Huge, traits list, Race East Asian,
  Age Late Teen
- Button: "Begin Your Story"
- **Current issues:**
  - Discovery is framed, but still not interactive.
  - Traits and body presentation remain dense for presets.
  - This still wants 4-5 actual discovery beats, not only better setup prose.

### 6. First In-Game Scene
- Workplace route now starts from scheduler-picked opening content instead of falling through to `base::rain_shelter`.

### 6a. Rain Shelter Scene (what the player actually sees)
- No longer the default Robin/workplace opening.
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
4. FemCreation → transformation bridge prose, then body confirmation
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
| Stats during narrative | Resolved | TransformationIntro no longer shows gameplay sidebar chrome |
| "People Here" empty | Resolved | TransformationIntro no longer shows the empty NPC panel |

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
| Transformation sequence | Critical | Better framed now, but still missing 4-5 interactive discovery beats |
| Adult content | Critical | Zero explicit scenes. Game can't prove its premise. |
| Scene transitions in UI | Critical | Engine/presentation problem, not content |
| First-scene bug fix | Resolved | Routed new games now reach scheduler-picked opening scenes first |
| Writing register pass | High | 29 scenes in old register |
| Campus post-arc scenes | Low | No work/activity slot for campus route |

---

## Scheduler Bug Detail

Resolved in runtime flow. Routed new games now try `pick_next()` before falling
back to `opening_scene`, so workplace and campus opening arcs can actually fire.
