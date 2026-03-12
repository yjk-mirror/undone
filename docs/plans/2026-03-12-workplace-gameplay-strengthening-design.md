# Workplace Gameplay Strengthening Design

**Date:** 2026-03-12
**Status:** Approved

## Problem

The workplace month-one route has improved prose and better week-one memory, but gameplay still
falls short in four connected ways:

1. Explicit scenes do not mutate real sexual state, so virginity-loss and sexual continuity are
   not actually simulated.
2. The month-one payoff mix is skewed toward opportunistic and workplace-transgressive lanes
   because Jake's romantic explicit scene still lands in week 5.
3. The party-social stranger route advertises a gateway but currently dead-ends.
4. Several week-one memory branches are stored but only lightly consumed, and early free-time
   pacing risks feeling queued rather than naturally lived.

This pass exists to make the route feel more truthful to the approved fantasy: varied adult
experiences, strong sense of self, trait continuity, and stateful consequences.

## Goals

1. Make first-time and repeat explicit scenes differ in actual persistent game state.
2. Bring the romantic explicit Jake lane into month one without collapsing its buildup.
3. Turn the party-social stranger lane into a real, distinct explicit payoff path.
4. Spend the highest-value week-one flags in later content so choices feel lived, not merely logged.
5. Smooth week-2 and week-3 free-time flow so callbacks blend into the sim instead of blocking it.
6. Use traits to alter erotic logic, pacing, and aftermath, not just surface prose.

## Non-Goals

- campus-route parity
- broad scheduler redesign
- rewriting every repeatable scene in the pack
- adding non-consensual content
- replacing the existing romantic, bar, or Marcus lanes wholesale

## Core Design Decisions

### 1. Sexual state must be real state

The engine already supports:

- `set_virgin`
- `add_sexual_activity`
- relationship state
- persistent flags and aftermath triggers

Month-one explicit scenes should use those systems directly. Route flags alone are not enough.

Required outcome:

- Jake, Marcus, bar stranger, and party stranger explicit scenes all record real sexual history.
- First-time branches can produce route-specific aftermath and later prose.

### 2. Month one must serve multiple erotic audiences

By the end of week 4, the workplace route should already support:

- romantic/tender explicit payoff
- workplace-transgressive explicit payoff
- bar-stranger impulsive explicit payoff
- party-social stranger-risk explicit payoff

This avoids calibrating the early game too narrowly toward only one erotic taste.

### 3. Traits should change behavior, not just adjectives

Traits should shape:

- whether she advances, waits, tests, guides, or compartmentalizes
- how she reads risk and attention
- how she processes aftermath
- which fantasies feel naturally aligned with her current build

Important route uses:

- `SHY`: slower consent language, hesitation, shelter-seeking, relief after stopping
- `CONFIDENT`: active guidance, deliberate escalation, stronger composure
- `ANALYTICAL`: risk parsing, room-reading, compartmentalized aftermath
- `ROMANTIC`: stronger emotional meaning and memory
- `OBJECTIFYING`: earlier recognition of sexual setup, hotter anticipation in darker-but-consensual lanes
- `AMBITIOUS`: stronger workplace/status collision
- `FLIRTY`: more playful escalation and deliberate signal-sending

### 4. Darker-leaning fantasy coverage stays inside explicit boundaries

The game is fictional and adult-only. All explicit content remains:

- adult
- consensual
- fictional

But scenes can still serve darker-leaning fantasies through:

- pressure
- exposure
- secrecy
- social danger
- transactional coldness
- overwhelming momentum

The route should support those fantasies through prior player opt-in, trait-sensitive logic, and
aftermath that distinguishes thrill, pride, hunger, shame, compartmentalization, or confusion.

## Implementation Strategy

### Slice 1: Explicit-state persistence

Update the live explicit scenes to set:

- virginity state when applicable
- sexual activity memory
- relationship or partner continuity where appropriate
- route-specific aftermath hooks

Primary files:

- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/jake_apartment.toml`
- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/work_marcus_closet.toml`
- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/bar_stranger_night.toml`

### Slice 2: Jake in week 4

Move `jake_apartment` into late week 4 under the right relationship and flag conditions.

Primary files:

- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/data/schedule.toml`
- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/jake_second_date.toml`
- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/jake_apartment.toml`

### Slice 3: Party explicit payoff

Author the missing follow-up to `PARTY_STRANGER_OUTSIDE`.

The scene should feel different from the bar lane by using:

- party residue
- narrow privacy
- being chosen out of a crowd
- secrecy after public heat

Primary files:

- new scene under `packs/base/scenes/`
- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/data/schedule.toml`
- `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/party_invitation.toml`

### Slice 4: Spend week-one flags

Consume these flags in live content:

- arrival posture
- landlord posture
- lunch style
- functional-clothes path

Primary consumers:

- opening callback scenes
- one or two social/work-social scenes
- route aftermath scenes

### Slice 5: Relax free-time pacing

Keep only the highest-priority callback beats as trigger-first. Convert the rest to weighted
conditional content so week 2 does not feel like the player is clearing an internal queue.

## Acceptance Standard

This pass succeeds when:

1. Multiple explicit routes in month one mutate real sexual state.
2. Jake can pay off explicitly by week 4.
3. The party-outside branch has a real explicit follow-up.
4. Week-one flags affect later scene selection or later substantial prose, not only docs.
5. Traits measurably change the erotic and emotional meaning of scenes.
6. The route still validates and still plays like a life, not a menu of isolated erotic set pieces.
