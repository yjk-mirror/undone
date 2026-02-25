# Writing & Game Design Audit — 2026-02-25

**Scope:** All 19 scene files, 2 arc docs, 2 character docs, all pack data files, schedule/arc structure.
**Baseline:** 15 scheduled scenes + 3 universal + 1 routing stub. Two arcs (Robin 7 scenes, Camila 7 scenes).
**Method:** 3 parallel writing-reviewer agents (Robin arc, Camila arc, universal scenes) + 1 game design auditor.

---

## Summary

| Severity | Writing | Game Design | Total |
|----------|---------|-------------|-------|
| Critical | 10      | 6           | 16    |
| Important | 20     | 14          | 34    |
| Minor    | 14      | 7           | 21    |

The prose foundation is strong — character voices are distinct, transformation content is structurally present, and the best scenes (`robin_work_meeting`, `robin_evening`, `camila_study_session`) are near publication quality. The game design structure (arcs, scheduling, state machines) is well-conceived.

Two systemic issues dominate:

1. **POV violation (arc-wide):** Both arcs are written in third-person close ("she") instead of second-person ("you") as the writing guide specifies. TRANS_WOMAN branches in universal scenes also use third-person. This affects every prose block in all 19 files and requires a policy decision before any other fixes.

2. **TRANS_WOMAN register gap (arc-wide):** TransWomanTransformed PCs receive cis-male disorientation prose across most scenes in both arcs. The writing guide is explicit: these are opposite emotional registers that must never be conflated. 13 of 19 scenes lack proper TRANS_WOMAN inner branches.

Beyond these systemic issues, the game design audit reveals a **post-arc content void** — after both arcs exhaust their scenes, the player loops three repeatable free_time scenes indefinitely. Multiple skills, stats, and traits defined in pack data have zero scene coverage.

---

## Part 1: Writing — Systemic Findings

### W-S1. Third-person narration throughout both arcs (Critical — policy decision required)

The writing guide specifies: "Always second-person: 'You go...', 'You see...' Always present tense."

Every scene in both the Robin and Camila arcs uses third-person close ("She stands," "She takes the stairs," "She opens her phone") instead of second-person ("You stand," "You take the stairs"). The universal scenes (`rain_shelter`, `coffee_shop`, `morning_routine`) use second-person for all branches *except* TRANS_WOMAN, which switches to third-person.

This creates three inconsistencies:
- Robin arc: entirely third-person (all branches)
- Camila arc: entirely third-person (all branches)
- Universal scenes: second-person everywhere *except* TRANS_WOMAN branches

**Decision required:** Either (a) convert all prose to second-person throughout, or (b) ratify third-person close as a valid arc-level choice and convert the TRANS_WOMAN universal branches to match. Option (a) is the guide-compliant path.

### W-S2. TRANS_WOMAN inner branches missing from 13 of 19 scenes (Critical)

The writing guide mandates a three-level pattern: `alwaysFemale` → `TRANS_WOMAN` → default (cis-male-start). Most scenes only implement two levels (`alwaysFemale` → everyone else), giving TransWomanTransformed PCs the cis-male disorientation register.

**Scenes with correct 3-level pattern:** `robin_work_meeting`, `robin_evening`, `rain_shelter`, `coffee_shop`, `morning_routine`, `camila_dining_hall` (partial).

**Scenes missing TRANS_WOMAN branches:** `robin_arrival`, `robin_landlord`, `robin_first_night`, `robin_first_clothes`, `robin_first_day`, `camila_arrival`, `camila_dorm`, `camila_orientation`, `camila_library`, `camila_call_raul`, `camila_study_session`, `transformation_intro` (has it for some beats but not all), `plan_your_day` (no transformation content, exempt).

The most damaging cases:
- `camila_dorm`: entire scene is about desire-as-destabilisation, which is wrong for trans women
- `camila_arrival`: "welcome" reads as disorientation instead of recognition
- `robin_first_night`: low-FEMININITY intro_variant uses male pronouns ("He — she") which contradicts the trans woman's self-knowledge

### W-S3. Low-FEMININITY intro_variants lack TRANS_WOMAN guards (Critical)

`robin_arrival`, `robin_first_night`, and `robin_first_day` have intro_variants gated to `!w.alwaysFemale() && w.getSkill('FEMININITY') < 15`. This condition admits TransWomanTransformed PCs (who return `false` for `alwaysFemale()`). The content uses male pronoun slippage ("He — she — Robin does the math") which is wrong for trans women — they would not experience pronoun confusion.

Fix: Add `&& !w.hasTrait("TRANS_WOMAN")` to the condition, or add inner guards.

---

## Part 2: Writing — Per-Scene Findings

### Robin Arc

#### `robin_arrival.toml`
| # | Severity | Finding |
|---|----------|---------|
| 1 | Important | Staccato paragraph break: "She stands." as isolated dramatic beat |
| 2 | Important | Over-naming: "indifferent in the specific way airports are" |
| 3 | Important | Staccato triple: "Forget. Stand up. Remember." in low-FEMININITY intro_variant |

#### `robin_landlord.toml`
| # | Severity | Finding |
|---|----------|---------|
| 4 | Important | Emotion announcement: "Robin is still slightly mortified" (SHY branch) |
| 5 | Minor | Sentence fragment as atmospheric filler in default branch |

#### `robin_first_night.toml`
| # | Severity | Finding |
|---|----------|---------|
| 6 | Critical | Staccato trailing closer: "Outside, the city goes on." — canonical prohibited pattern |
| 7 | Important | SHY `call_someone` branch is an adjective-swap (same outcome, slightly shorter conversation) |
| 8 | Important | "The city is outside being the city" — meta-framing in research action |

#### `robin_first_clothes.toml`
| # | Severity | Finding |
|---|----------|---------|
| 9 | Important | Repeated emotion announcement: "Robin is still slightly mortified" (SHY) |
| 10 | Important | No alwaysFemale thoughts block at the dressing room mirror beat |
| 11 | Minor | TRANS_WOMAN distinction missing from `shopper_notices` NPC action |
| 12 | Minor | Redundancy: "She is learning how this works" restates the italicised inner voice |

#### `robin_first_day.toml`
| # | Severity | Finding |
|---|----------|---------|
| 13 | Important | Staccato closer: "The day moves forward at the speed of days." |
| 14 | Minor | SHY intro almost-dropped-bag beat needs follow-through |

#### `robin_work_meeting.toml`
| # | Severity | Finding |
|---|----------|---------|
| 15 | Important | Over-naming: "the other thing, the layer underneath it" in cis-male-start close |
| 16 | Important | Staccato fragments: "Not with malice. Not with awareness." in OBJECTIFYING intro |
| 17 | Minor | "It goes there." staccato pair in TRANS_WOMAN `present` action |
| 18 | Minor | alwaysFemale `after` action thinner than other paths |

#### `robin_evening.toml`
| # | Severity | Finding |
|---|----------|---------|
| 19 | Critical | Copy-paste repetition: "a quiet that's becoming, if not comfortable, at least familiar" duplicated verbatim in same sentence |
| 20 | Important | Staccato pair: "The city is outside. She is in here." + meaning-announcement |
| 21 | Important | "Not comfortable — not yet." — em-dash reveal pattern in TRANS_WOMAN settle path |
| 22 | Minor | ANALYTICAL path over-explains Robin's coping method |

### Camila Arc

#### `camila_arrival.toml`
| # | Severity | Finding |
|---|----------|---------|
| 23 | Critical | TRANS_WOMAN register absent — welcome reads as disorientation instead of recognition |
| 24 | Important | `find_the_room` POSH and DOWN_TO_EARTH branches are adjective-swaps |
| 25 | Important | Emotion announcement: "she feels slightly better about the whole thing" (SHY) |
| 26 | Important | Staccato closer: "She doesn't know when later is." |

#### `camila_dorm.toml`
| # | Severity | Finding |
|---|----------|---------|
| 27 | Critical | TRANS_WOMAN register absent — desire-as-destabilisation is wrong for trans women |
| 28 | Critical | Over-naming: "The thing about shame is it doesn't argue" — narrator meta-frames shame mechanism |
| 29 | Important | `try_to_sleep` alwaysFemale branch too thin (4 words of content) |
| 30 | Important | `text_someone` — Raul-specific content reaches alwaysFemale without a gate |

#### `camila_orientation.toml`
| # | Severity | Finding |
|---|----------|---------|
| 31 | Important | TRANS_WOMAN absent from insider-knowledge moment |
| 32 | Important | `skip_afternoon` — alwaysFemale gets no scene texture from male-glance moment |
| 33 | Important | Over-named closing: "*six months ago I would have done exactly that*" announces theme |
| 34 | Minor | Trailing closer: "Orientation continues." |

#### `camila_library.toml`
| # | Severity | Finding |
|---|----------|---------|
| 35 | Critical | Over-naming: "the kind of jaw that you notice before you've decided to notice anything" |
| 36 | Critical | Over-naming closer: "her face is doing something she doesn't have a name for yet" |
| 37 | Important | TRANS_WOMAN absent — attraction scene uses wrong register for trans women |
| 38 | Important | `theo_looks_up` alwaysFemale gets no closing beat |
| 39 | Minor | "notes that she noticed it" — slightly circular phrasing |

#### `camila_call_raul.toml`
| # | Severity | Finding |
|---|----------|---------|
| 40 | Critical | SHY / AMBITIOUS / default intro branches are adjective-swaps — same outcome |
| 41 | Critical | `call_him_back` delivers identity-crisis prose to alwaysFemale without a gate |
| 42 | Important | Staccato closer: "She has a problem set due Friday." |
| 43 | Minor | Anaphoric: three "He's right here" constructions — trim to one |

#### `camila_study_session.toml`
| # | Severity | Finding |
|---|----------|---------|
| 44 | Important | Intro body-unfamiliarity prose reaches alwaysFemale without a gate ("the way she used to") |
| 45 | Important | Over-named closer: "she doesn't know what to do with the gap" |
| 46 | Minor | "always slightly too present" — edges toward italicised coinage |
| 47 | Minor | SEXIST branch has two ambient atmospheric sentences after key insight — cut one |

#### `camila_dining_hall.toml`
| # | Severity | Finding |
|---|----------|---------|
| 48 | Critical | alwaysFemale `hierarchy` path skips the scene's central beat entirely |
| 49 | Important | SEXIST trait branch absent despite arc doc specifying it |
| 50 | Important | `week` alwaysFemale path thin relative to transformed paths |
| 51 | Minor | "the kind of person who takes up space without thinking about it" — soft over-naming |

### Universal Scenes

#### `transformation_intro.toml`
| # | Severity | Finding |
|---|----------|---------|
| 52 | Critical | Staccato closer: "It is." — two-word paragraph as dramatic reveal |
| 53 | Important | Over-naming in alwaysFemale: "a quality to the morning you can't immediately locate" |
| 54 | Minor | "Somewhere a door closes." — trailing atmospheric closer |
| 55 | Minor | TRANS_WOMAN `waking`: "You breathe in. You breathe out." — staccato triple |

#### `rain_shelter.toml`
| # | Severity | Finding |
|---|----------|---------|
| 56 | Critical | Over-naming: "There's a specific quality to being looked at by a strange man" — the guide's own example |
| 57 | Important | TRANS_WOMAN branches use third-person ("she") while rest is second-person |
| 58 | Important | "Boundaries aren't a personality flaw." — narrator editorializing |

#### `coffee_shop.toml`
| # | Severity | Finding |
|---|----------|---------|
| 59 | Critical | Over-naming: "There's a geometry to being a woman in a line" — category label before specifics |
| 60 | Critical | Same over-naming repeated in TRANS_WOMAN version of same block |
| 61 | Important | "That might be the milestone." — staccato closer / over-naming hybrid |
| 62 | Minor | "The universal coffee-shop acknowledgment" — meta-commentary on a nod |

#### `morning_routine.toml`
| # | Severity | Finding |
|---|----------|---------|
| 63 | Important | Trailing staccato closer: "Monday." as standalone paragraph |
| 64 | Important | "The day is waiting. Best not to keep it." — aphorism button |
| 65 | Minor | TRANS_WOMAN wardrobe block: POV slip to third-person in one paragraph |
| 66 | Minor | "Coffee in hand. The day starts now." — two-fragment closer |

#### `plan_your_day.toml`
| # | Severity | Finding |
|---|----------|---------|
| 67 | Critical | Entire scene is a placeholder stub — 4-8 words of prose per time-slot variant, no world texture, no trait branching, no transformation content, no NPC. Needs full rewrite. |
| 68 | Important | "The evening is yours." — narrator voiceover, not prose |
| 69 | Important | "Sometimes doing nothing is doing something." — platitude substituted for scene |

---

## Part 3: Game Design Findings

### Critical

#### D-C1. Both arcs terminate into a three-scene content void
When Robin reaches `settled` and Camila reaches `first_week`, all arc-specific scenes are exhausted. The player loops `rain_shelter`, `morning_routine`, and `coffee_shop` indefinitely. Neither terminal state has any associated scenes. This is the core playability gap.

#### D-C2. `robin_first_clothes` is permanently unreachable — dead scene
`robin_first_clothes` is weighted (weight 10) in the `week_one` state. But `robin_first_day` is a trigger in the same state. `pick_next()` evaluates triggers first (alphabetically), so `robin_first_day` always fires first, advancing the arc to `working`. `robin_first_clothes` is gated to `week_one` and can never fire. A complete, well-written scene that no player will ever see.

#### D-C3. `free_time` slot is too thin to sustain play
Only 3 repeatable scenes after arc exhaustion. `plan_your_day` is a once-only trigger. `morning_routine` at weight 15 dominates over `coffee_shop` and `rain_shelter` at weight 10. No decay mechanism means players see the same scenes at the same ratios forever.

#### D-C4. Four personality traits have near-zero scene presence
| Trait | Scenes used |
|-------|-------------|
| SULTRY | `coffee_shop` only (one action) |
| ROMANTIC | `coffee_shop` only (one action) |
| REFINED | `morning_routine` only |
| OVERACTIVE_IMAGINATION | `robin_first_clothes` only (unreachable — see D-C2) |

These traits are available at character creation but barely affect gameplay.

#### D-C5. Eight skills have zero scene usage
FITNESS, CHARM, FASHION, DANCE, COOKING, ADMIN, MANAGEMENT, CHILDCARE — defined in `skills.toml`, tracked by the engine, shown on the sidebar, but never referenced in any scene condition or effect. Only FEMININITY is used.

#### D-C6. All three stats are ghost data
`TIMES_KISSED`, `DATES_ATTENDED`, `WEEKS_WORKED` — defined in `stats.toml`, never set by any scene effect, never referenced in any condition.

### Important

#### D-I1. FEMININITY never increments — progression writing is unreachable
No scene sets `change_skill FEMININITY +N`. FEMININITY starts at 10 (cis-male-start) and stays there forever. `robin_evening` and `camila_dining_hall` have three-tier FEMININITY branching (< 25, < 50, ≥ 50) — the 25-49 and 50+ prose is written but unreachable.

#### D-I2. Camila `orientation` state has only one scene
The arc doc describes orientation as "day 1-3" with "early social scenes," but only `camila_orientation` exists. The state is a single-scene pass-through.

#### D-I3. `morning_routine` weight 15 will dominate free_time
No decay mechanism. Players will see `morning_routine` ~60% of free_time draws, quickly feeling repetitive.

#### D-I4. Robin fetishization thread — agency side never written
The character doc describes a "lean into / cut it off" player choice regarding the male gaze. Only the observation side is implemented. No scene offers a choice to use or capitalize on being looked at.

#### D-I5. Robin's emerging attraction to men has no arc-specific scenes
Camila has three scenes engaging sexual confusion. Robin's character doc notes she's "curious; not ready to admit" her attraction to men, but no Robin-specific scene addresses this.

#### D-I6. CONFIDENT, SEXIST, HOMOPHOBIC attitude traits absent from Robin arc
`robin_work_meeting` branches on OBJECTIFYING and ANALYTICAL. `robin_evening` branches on ANALYTICAL and AMBITIOUS. But CONFIDENT, SEXIST, HOMOPHOBIC have zero presence in Robin's arc.

#### D-I7. Camila social position inversion underbuilt
The character doc specifies privilege inversions as a core thread. Only `camila_dining_hall` directly addresses this. The thread is introduced but not developed.

#### D-I8. `camila_call_raul` implements the Diego call, not the parent call
The arc doc describes this scene slot as "The phone call home... one of the sharpest moments in the arc." What was implemented is Diego calling for Raul. The parent call — arguably more charged — doesn't exist.

#### D-I9. Most choices are cosmetic — same flag set regardless of decision
Nearly every scene sets the same game flag no matter which action the player takes. Choices branch prose and adjust stress/anxiety by ±2-5 points, but converge to identical outcomes. The writing guide mandates "at least one lasting consequence per scene" — most scenes satisfy this technically but without flag branching based on player decisions.

#### D-I10. No NPC relationship infrastructure
David (rain shelter), Jake (coffee shop), Theo (library) — all set meeting flags but have no persistent NPC records and no follow-up scenes. Relationships can't deepen.

#### D-I11. `camila_library` sets no world-persistent memory
The only scene in either arc that leaves no world trace. Meeting Theo — a named NPC with a reaction beat — sets no game flag.

#### D-I12. Arrival scenes redundantly set route flags
`robin_arrival` and `camila_arrival` both set `ROUTE_ROBIN` / `ROUTE_CAMILA` flags that were already set by the character creation preset via `starting_flags`.

#### D-I13. alwaysFemale players receive significantly thinner content across both arcs
Always-female branches are consistently present but often 1-2 sentences versus multiple paragraphs for transformation paths. `camila_dining_hall` skips the scene's central beat entirely for alwaysFemale. An alwaysFemale player experiences a noticeably diminished game.

#### D-I14. `robin_arrival` / `robin_landlord` trigger sequencing depends on alphabetical scene ID ordering
The correct ordering of arrival → landlord → first_night is achieved by alphabetical trigger precedence, not explicit dependency. Renaming a scene would break the sequence.

### Minor

#### D-M1. `plan_your_day` is too thin to justify existence as a scene
Functionally a pass-through routing stub with 4-8 words per time-slot variant. One extra free_time draw, no texture.

#### D-M2. Terminal arc states (`settled`, `first_week`) undocumented in schedule.toml
A future content author wouldn't know these are intentionally terminal vs. a scheduling gap.

#### D-M3. Desire crossover absent from Robin arc-specific scenes
Robin's character doc notes her attraction to men. None of Robin's 7 arc scenes address this — it exists only in universal scenes.

#### D-M4. Body unfamiliarity texture weaker in Camila arc than Robin arc
Robin has 3 strong body-unfamiliarity scenes (first_night bra, first_clothes mirror, evening light switch). Camila has one (study_session hair/posture).

#### D-M5. `change_anxiety` / `change_stress` not defined in stats.toml
Used as engine-level effects but not declared alongside the pack-defined stats. Undocumented distinction.

#### D-M6. No FEMININITY gating in `morning_routine` or `coffee_shop` universal scenes
Transformation commentary reads as first-time observation appropriate at FEMININITY < 25 but inert at FEMININITY 60. Should be gated.

#### D-M7. `condition` and `trigger` fields identical in all trigger-mode schedule entries
Both fields serve different roles in the scheduler, but being identical for every trigger event creates redundant noise. Document the distinction or simplify.

---

## Part 4: Strengths

**Writing:**
- Character voice is distinct and consistent — Robin's systematic inventory vs. Camila's explosive reactivity come through clearly
- `robin_work_meeting` and `robin_evening` are the best scenes in the codebase — near publication quality, with genuine structural branching on attitude traits and correct three-level transformation gating
- `camila_study_session` SEXIST and HOMOPHOBIC branches are genuinely different events, not adjective swaps
- Transformation content when present is well-calibrated — the four textures (insider knowledge, body unfamiliarity, social reversal, desire crossover) are all represented across the corpus
- NPC dialogue is specific to character and goal — Frank the landlord, Dan the coworker, Marcus, Jake, Diego all have distinct voices
- `transformation_intro.toml` correctly handles all four PC origins with structurally different paths

**Game Design:**
- Arc state machines are clean and logical — the progression makes narrative sense
- Schedule structure is sound — triggers for mandatory beats, weights for optional content, once-only for progression scenes
- The trait system (personality + attitude) creates genuine combinatorial variety when properly used
- The FEMININITY dial is a well-designed mechanic — the three-tier branching in `robin_evening` and `camila_dining_hall` shows what it looks like when fully leveraged
- Cross-reference integrity is perfect — every scene ID, arc state, game flag, and goto target resolves correctly

---

## Priority Recommendations

### Immediate (blocks content quality)
1. **Policy decision on POV** — third-person or second-person? Resolve before any other prose work.
2. **TRANS_WOMAN inner branches** — systematic pass through all 13 affected scenes.
3. **Fix `robin_first_clothes` scheduling** — move it to a different arc state or make it a trigger that fires before `robin_first_day`.
4. **Fix `robin_evening` copy-paste duplication** — verbatim repeated clause in `settle` action.

### Before next content pass
5. **alwaysFemale gating audit** — particularly `camila_call_raul`, `camila_dorm`, `camila_dining_hall`.
6. **FEMININITY increment mechanism** — at least some scenes should set `change_skill FEMININITY +N` to make the dial functional.
7. **`plan_your_day` full rewrite** — currently below quality floor.

### Content expansion priorities
8. **Post-arc content** — `settled` and `first_week` states need scenes, or a second arc layer.
9. **Free_time expansion** — more universal scenes to sustain play.
10. **Trait coverage** — SULTRY, ROMANTIC, REFINED, OVERACTIVE_IMAGINATION need scene presence.
11. **Skill integration** — CHARM, FASHION, FITNESS at minimum should affect social scenes.
