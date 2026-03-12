# Workplace Month-One Writing Design

**Date:** 2026-03-12
**Status:** Approved

## Problem

The game has the right creative bible, but the live player experience still falls short of it in
three linked ways:

1. The opening does not yet prove the premise. The plane scene exists, but the bridge into being
   her is weak, and several early scenes still read below the approved register.
2. The first month is not yet structured as a real life-sim. There are scenes, but not yet a
   durable week-by-week gameplay shape that combines work, city life, adult routes, and
   consequences.
3. The engine still treats authored scenes as effectively one active man plus one active woman.
   That is too narrow for the workplace route we actually want to write.

This plan treats Robin as the calibration build for a reusable workplace archetype, not as a
special-case protagonist whose content only works for one locked preset.

## Reference Sources

The design is derived from the repo's current direction, not invented fresh:

- `C:/Users/YJK/dev/mirror/undone/docs/creative-direction.md`
- `C:/Users/YJK/dev/mirror/undone/docs/writing-guide.md`
- `C:/Users/YJK/dev/mirror/undone/docs/writer-core.md`
- `C:/Users/YJK/dev/mirror/undone/docs/review-core.md`
- `C:/Users/YJK/dev/mirror/undone/docs/presets/robin.md`
- `C:/Users/YJK/dev/mirror/undone/docs/arcs/workplace-opening.md`
- `C:/Users/YJK/dev/mirror/undone/docs/player-experience-map.md`
- `C:/Users/YJK/dev/mirror/undone/docs/plans/2026-03-08-writing-register-calibration.md`
- `C:/Users/YJK/dev/mirror/undone/docs/plans/2026-03-08-prolific-writing-session.md`
- `C:/Users/YJK/dev/mirror/undone/docs/content-schema.md`
- `C:/Users/YJK/dev/mirror/undone/docs/engine-contract.md`

## Core Decisions

### 1. Robin is a calibration build, not a one-off heroine

Robin remains the first workplace route used to prove the game, but the authored material should
generalize to the broader workplace archetype:

- adult professional
- recently displaced into a new body and city
- competent at work
- newly subject to gendered misrecognition, attention, and bodily responses

Robin-specific details should calibrate tone and scene logic, not overconstrain the route into a
single narrow personal story.

### 2. Existing prose is editable, not sacred

The live scene set should not be treated as immutable. Weak prose, weak opening beats, shallow
branching, and scenes that no longer meet the approved writing principles should be rewritten,
restructured, or replaced.

### 3. Month one must deliver both breadth and depth

The first four weeks must support:

- broad life-sim texture
- real workplace play
- several erotic/adult route families
- multiple fully explicit paths
- virginity-loss through more than one route register
- aftermath and divergence, not just first-payoff scenes

### 4. Explicit scenes are core content

This is an adult fictional game for consenting adults. The erotic content is the focus of the
game, not a stretch goal. The first month must therefore include multiple fully explicit paths.

Those scenes must:

- be earned by setup and route logic
- be physically specific
- differ structurally by lane and trait build
- avoid generic AI erotic prose, purple prose, and vague "heat/desire building" filler

### 5. Multi-NPC support is a real platform need

The engine's current one-active-man/one-active-woman focus is a gap for the planned route.
Month-one content needs real authored support for:

- meetings with multiple coworkers
- parties and bar clusters
- female comparison/social-reading beats
- situations where more than one meaningful NPC is present at once

The required upgrade is not crowd simulation. It is role-addressable multi-NPC support for
authored scenes.

## Goals

1. Make week 1 read and play at the approved prose standard.
2. Build a workplace-route first month that feels like a life, not a corridor.
3. Support multiple explicit adult routes by weeks 3-4.
4. Ensure virginity-loss can occur through multiple emotional/situational lanes.
5. Add enough engine support for authored multi-NPC scenes to stop faking group dynamics.
6. Leave the route reusable for future workplace archetypes beyond Robin.

## Non-Goals

- broad UI repolish unrelated to opening/scene readability
- full campus-route parity in the same session
- open-ended sandbox completion beyond the first month
- arbitrary crowd AI or systemic social simulation
- preserving weak existing scenes for continuity's sake

## Engine Reality And Required Upgrade

The existing engine already supports:

- deterministic `trigger` scenes for route spine beats
- weighted repeatables for life-sim texture
- persistent game flags, arc states, skills, stats, virginity state, alcohol, arousal, and NPC
  relationship data
- deep branching inside intro prose, thoughts, action prose, NPC actions, and next branches

That is enough for the first month, except for one critical gap: authored scenes can only rely
directly on a single active male NPC and a single active female NPC.

### Required multi-NPC upgrade

The upgrade should keep backward compatibility with `m` and `f`, but add role-addressable NPC
access across prose, conditions, and effects.

Minimum useful capability:

- bind multiple NPCs into scene context by authored role
- access those NPCs in prose templates without pretending only one person matters
- target bound NPCs in effects using role ids, not only `m` or `f`
- surface multiple active NPCs in runtime/UI/debug views well enough for authoring and testing

This is sufficient for meetings, parties, work clusters, and social scenes without trying to
model a whole crowd.

## Month-One Gameplay Architecture

Month one should be a layered workplace life-sim made of four interacting lanes:

- route spine
- daily-life lane
- workplace lane
- adult-route lane

### Week 1: Shock, setup, forced adaptation

Purpose:
- prove the premise
- fix the opening
- establish body/city/workplace baseline

Required content:
- plane scene rewrite
- transformation bridge/discovery sequence
- arrival rewrite
- landlord / first night / first clothes / first day rewrites
- first work-meeting rewrite
- first evening rewrite

Erotic register:
- charged and destabilizing
- not a payoff-free intro
- opens later adult paths without collapsing immediately into generic first-sex content

### Week 2: Operational life begins

Purpose:
- move from disorientation to lived routine
- make the game feel like a life

Required content:
- repeatable home/city/body-management scenes
- repeatable work scenes with real variation
- first clear romantic escalation
- first clear situational escalation
- early possible route toward virginity-loss

### Week 3: Divergence and major payoffs

Purpose:
- prove the game is an adult game

Required content:
- at least three distinct adult path families available:
  - romantic/tender
  - situational/impulsive
  - workplace/transgressive
- fully explicit scenes
- virginity-loss available through more than one route type

### Week 4: Consequences and continuity

Purpose:
- stop the route from ending at first payoff

Required content:
- aftermath scenes
- changed work/social texture
- repeat-contact scenes
- continuing sexual/personal states that affect later play

Possible end-of-week-4 states should include:
- seriously involved with Jake
- entangled with Marcus
- had an impulsive stranger experience
- sexually active but uncommitted
- still inexperienced because of choices/build, but with strong openings

## Content Lanes And Scene Families

### 1. Opening and adaptation family

Files currently in scope:

- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/transformation_intro.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_arrival.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_landlord.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_first_night.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_first_clothes.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_first_day.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_work_meeting.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_evening.toml`

This family sets the prose standard for everything else.

### 2. Daily-life baseline family

Primary live files to rewrite or replace:

- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/morning_routine.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/plan_your_day.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/bookstore.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/park_walk.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/grocery_store.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/evening_home.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/weekend_morning.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/shopping_mall.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/landlord_repair.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/laundromat_night.toml`

These scenes make being her feel continuous.

### 3. Workplace baseline family

Primary live files:

- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_standup.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_lunch.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_late.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_corridor.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_friday.toml`

This lane must carry competence, hierarchy, misrecognition, and office-adjacent erotic tension.

### 4. Romantic adult lane

Primary live files:

- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/coffee_shop.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/coffee_shop_return.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_outside.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_first_date.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_second_date.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_apartment.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_morning_after.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_text_messages.toml`

Erotic register:
- tenderness
- anticipation
- specific desire
- explicit content that differs from every other lane

### 5. Situational/impulsive adult lane

Primary live files:

- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/neighborhood_bar.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/bar_closing_time.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/bar_stranger_night.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/party_invitation.toml`

Erotic register:
- unpredictability
- momentum
- the world exceeding her plan
- clearly consensual adult fiction without flattening into romance

### 6. Workplace transgression lane

Primary live files:

- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_coffee.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_favor.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_late.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_drinks.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_closet.toml`
- `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_aftermath.toml`

Erotic register:
- competence recognition
- compressed urgency
- risk
- distinct from both Jake and stranger content

### 7. Virginity-loss layer

This is not a single scene family. It is a route layer that must exist across:

- romantic lane
- situational lane
- workplace-transgression lane

Each version must have different pacing, meaning, and aftermath.

## Hard Writing Rules

All month-one content must obey the current writing standard:

- DM narrator, not novelist
- second-person present tense
- intro describes the world; actions are what the player chooses
- transformation shown through physical fact, not narrator analysis
- no filler actions
- no adjective-swap branching
- no generic AI erotic prose
- no purple prose
- no route payoffs that have not been earned by structure

For explicit scenes specifically:

- they must be fully authored, not placeholders
- they must be physically concrete
- they must differ by lane and by trait
- they must be written as adult fictional content for consenting adults

## Writing Session Structure

The implementation should run in this order:

1. Engine gap closure for authored multi-NPC scenes
2. Opening rewrite and transformation bridge
3. Week-1 workplace spine rewrite
4. Daily-life baseline rewrite/replacement cluster
5. Jake lane
6. Marcus lane
7. Stranger/social lane
8. Aftermath and continuation pass
9. Schedule/progression balancing pass

## Success Criteria

The work succeeds when:

- week 1 is strong enough to define the game's prose standard
- the first month feels like a workplace life, not a scene checklist
- multiple explicit routes are reachable by weeks 3-4
- virginity-loss can happen through multiple route families
- the route continues meaningfully after first payoff scenes
- group scenes no longer require pretending only one NPC matters
- the content remains reusable for future workplace-archetype builds beyond Robin
