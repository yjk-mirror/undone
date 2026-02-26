# Preset: Robin — Workplace Arc

## Quick Reference

| Field | Value |
|---|---|
| Route flag | `ROUTE_WORKPLACE` |
| Arc | `base::workplace_opening` |
| Before name | Robin (gender-neutral, kept) |
| Current name | Robin |
| Age (before) | Early 30s |
| Age (now) | Appears late teens |
| Race (before) | White |
| Race (now) | East Asian |
| Figure | Petite, short |
| Breasts | Huge (H-cup range) |
| Appearance | Stunning — draws immediate attention |
| Job | Software engineer (hired pre-transformation, starts Monday) |
| Arrived | Saturday before first Monday |
| Season | Early spring |
| FEMININITY start | 10 (CisMaleTransformed) |
| Sexuality (before) | Straight (attracted to women) |

## What This Preset Is

Robin is a pre-configured character build that locks all physical and sexual attributes and
routes the player through the workplace arc. Choosing Robin means:

- All physical and sexual attributes are fixed (see tables below)
- All 38 traits are pre-assigned — no selection required
- Game flag `ROUTE_WORKPLACE` is set at game start
- The arc `base::workplace_opening` drives the opening scene sequence

You were thirty-two. A software engineer with ten years of experience. You took a job offer
in a city you didn't know — new company, new start, boxes shipped to an apartment you'd never
seen. When things go sideways, you inventory and solve. You're very good at that.

## Locked Physical Attributes

### Before (pre-transformation)

| Attribute | Value |
|---|---|
| Figure | Average (male) |
| Height | Average |
| Hair colour | Brown |
| Eye colour | Brown |
| Skin tone | Light |
| Penis size | Average |
| Voice | Average |

### After (post-transformation)

| Attribute | Value |
|---|---|
| Race | East Asian |
| Apparent age | Late teens |
| Figure | Petite |
| Height | Short |
| Breasts | Huge |
| Butt | Big |
| Waist | Narrow |
| Lips | Full |
| Hair colour | Black |
| Hair length | Long |
| Eye colour | Dark Brown |
| Skin tone | Light |
| Complexion | Glowing |
| Appearance | Stunning |
| Pubic hair (current) | Bare |
| Natural pubic hair | None |

### Sexual Attributes

| Attribute | Value |
|---|---|
| Nipple sensitivity | High |
| Clit sensitivity | High |
| Inner labia size | Average |
| Wetness baseline | Wet |

## Names

All name variants resolve to "Robin" — the name is gender-neutral and was kept.

| Variant | Value |
|---|---|
| Feminine | Robin |
| Androgynous | Robin |
| Masculine | Robin |

## Traits (38 total)

### Personality

| Trait ID | Name | Function |
|---|---|---|
| `AMBITIOUS` | Ambitious | Goal-focused. Impatient with things that don't advance anything. |
| `ANALYTICAL` | Analytical | Reaches for frameworks before feelings. Thinks in systems. |
| `DOWN_TO_EARTH` | Down to Earth | Practical, unselfconscious. |
| `OBJECTIFYING` | Objectifying | Had the casual objectifying gaze. Didn't think of it as a gaze. Just thought of it as noticing. |

### Physical

| Trait ID | Name | Function |
|---|---|---|
| `STRAIGHT_HAIR` | Straight Hair | Falls flat, sleek, swings when she turns. Low maintenance. |
| `SWEET_VOICE` | Sweet Voice | Warm, inviting. The kind men describe as "cute." The kind that makes "please" dangerous. |
| `ALMOND_EYES` | Almond Eyes | Elegant shape, slightly exotic to some men. |
| `WIDE_HIPS` | Wide Hips | Pronounced hip curve. Sways when she walks even when she's not trying. |
| `NARROW_WAIST` | Narrow Waist | Emphasized by anything fitted. Hands almost encircle it. |
| `SMALL_HANDS` | Small Hands | Delicate, feminine. |
| `PRONOUNCED_COLLARBONES` | Pronounced Collarbones | Visible, elegant. |
| `THIGH_GAP` | Thigh Gap | Space between thighs. Visible in tight jeans. |
| `SOFT_SKIN` | Soft Skin | Noticeably soft. People want to touch. |
| `NATURALLY_SMOOTH` | Naturally Smooth | Little to no body hair anywhere — a naturally hairless body. |
| `INTOXICATING_SCENT` | Intoxicating Scent | Something about how you smell makes men lean in. An effect, not a specific fragrance. |

### Sexual Response

| Trait ID | Name | Function |
|---|---|---|
| `HAIR_TRIGGER` | Hair Trigger | Cums embarrassingly fast. |
| `HEAVY_SQUIRTER` | Heavy Squirter | Squirting isn't subtle — it's dramatic, drenching, and unmistakable. |
| `MULTI_ORGASMIC` | Multi-Orgasmic | Can cum again within seconds. Stacks them. |
| `ORAL_FIXATION` | Oral Fixation | Her mouth is wired directly to between her legs. |
| `SENSITIVE_NECK` | Sensitive Neck | Light touch on her neck short-circuits her brain. |
| `SENSITIVE_EARS` | Sensitive Ears | Whispering in her ear is foreplay. |
| `SENSITIVE_INNER_THIGHS` | Sensitive Inner Thighs | The lightest fingertip trace up her inner thigh makes her gasp. |
| `SUBMISSIVE` | Submissive | Being told what to do makes her drip. |
| `PRAISE_KINK` | Praise Kink | "Good girl" makes her clench. |
| `EASILY_WET` | Easily Wet | Gets aroused fast and her body shows it immediately. |
| `BACK_ARCHER` | Back Archer | Involuntary response when she's close. |
| `TOE_CURLER` | Toe Curler | When she cums, her toes curl. |

### Arousal Response

| Trait ID | Name | Function |
|---|---|---|
| `NIPPLE_GETTER` | Nipple Getter | Her nipples visibly harden when she's turned on. |
| `FLUSHER` | Flusher | Flushes pink across her chest, neck, cheeks when aroused. |
| `THIGH_CLENCHER` | Thigh Clencher | Presses her thighs together when aroused. |
| `BREATH_CHANGER` | Breath Changer | Breathing gets shallow and fast when turned on. |
| `LIP_BITER` | Lip Biter | Unconscious tell. |

### Sexual Preference

| Trait ID | Name | Function |
|---|---|---|
| `LIKES_ORAL_GIVING` | Likes Oral (Giving) | Enjoys sucking cock. |
| `LIKES_DOUBLE_PENETRATION` | Likes Double Penetration | Filled in both holes simultaneously. |

### Dark Content

| Trait ID | Name | Function |
|---|---|---|
| `FREEZE_RESPONSE` | Freeze Response | When threatened, she freezes. |

### Menstruation

| Trait ID | Name | Function |
|---|---|---|
| `REGULAR_PERIODS` | Regular Periods | Predictable cycle, no surprises. |

## Scene Conditions

Workplace arc scenes require: `gd.hasGameFlag('ROUTE_WORKPLACE')`

Arc state gate: `gd.arcState('base::workplace_opening') == 'state_name'`

## Scene List

| Scene ID | Arc state gate | Location | Content | Sets |
|---|---|---|---|---|
| `base::workplace_arrival` | none (once-only, `ROUTE_WORKPLACE`) | Airport → subway | VANILLA | `ROUTE_WORKPLACE`, arc→arrived |
| `base::workplace_landlord` | arrived | Her apartment building | VANILLA | `MET_LANDLORD` |
| `base::workplace_first_night` | arrived | Her apartment | VANILLA | arc→week_one |
| `base::workplace_first_clothes` | week_one | Clothing store | VANILLA | — |
| `base::workplace_first_day` | week_one | Tech office | VANILLA | arc→working, `STARTED_JOB` |
| `base::workplace_work_meeting` | working | Tech office — meeting room | VANILLA | `FIRST_MEETING_DONE` |
| `base::workplace_evening` | working | Her apartment (evening) | VANILLA | arc→settled |

## Arc State Machine

```
arrived → week_one → working → settled
```

**arrived** (Saturday, day 0–1): workplace_arrival, workplace_landlord, workplace_first_night

**week_one** (Sunday before first workday): workplace_first_clothes (plus universal slot scenes)

**working** (Monday onward, has started job): workplace_first_day, universal work-adjacent scenes

**settled** (after ~week 2): Universal scenes fire normally. Preset-specific flavour via `[[intro_variants]]`.

## What This Preset Explores

These notes frame the thematic territory this build is designed to navigate.
They are not character backstory — they are design constraints for scene writers.

### Player register

You are thirty-two years old inside. You process the world like a senior engineer:
systematic, calm under pressure, methodical. You do not panic — you inventory. When
something goes wrong you say *okay* the way you say it on production incidents.

You are not performing competence. You are competent. The gap is between your internal
state and how every stranger reads you: they see a teenager. You are aware of this constantly.

You are not in denial about the transformation. You have accepted it the way you accept
any unexpected constraint: pragmatically, with a certain resignation. You are working the problem.

### The fetishization thread

You were a white man who had the casual gaze that fetishizes Asian women. You didn't think
much about it. You were not malicious — you were unexamined. Now you receive exactly that
gaze daily, from men who might have been your friends, who might have been you.

You can read it perfectly. You know the internal monologue of the man looking at you right
now because it was your internal monologue. This creates a specific kind of cognitive
dissonance: understanding something completely and still feeling it.

**Player choice:** Lean into the fetishization (accept it, perhaps use it, find something
in it), or cut it off (deflect, correct, refuse). Both are valid and fully written.

The `OBJECTIFYING` trait gates interiority on this — scenes use `w.hasTrait("OBJECTIFYING")`
to unlock recognition beats. The `ANALYTICAL` trait is secondary: it shapes how the
recognition is articulated.

### The misrecognition thread

You look late teens. You have a software engineering job, ten years of professional
experience, and you speak like someone who has been in boardrooms. You get carded.
You get asked if you are a student. You get explained things you invented.

This is not primarily a source of comedy. It is a constant low-grade friction that
occasionally becomes acute. Write it straight.

### Adaptation arc

Week 1: Purely practical. Bra shopping. Period supplies. Not knowing how to move through
the world in this body. Learning by doing. Internally still using male reference — "I" is
continuous, but the world is calling you she. The catching-yourself is not dramatized — just noted.

Over time: gradual adaptation. Small things that become familiar. Not a dramatic arc —
you don't "discover yourself" as a woman. You adjust, like a codebase migration.

### What the player doesn't know yet (opening arc)

- How to shop for clothes in this size
- How periods work in practice
- How to style her hair
- Why men hold doors now and what to do with that
- What she actually feels about men (curious; not ready to admit it)
- That her face is going to be a problem in ways she doesn't anticipate
