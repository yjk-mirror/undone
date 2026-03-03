# NPC Sidebar Redesign - UX Proposal (2026-03-03)

## Problem

Current sidebar only shows player stats. NPC details were hidden because the old panel
showed the wrong character, leaked information too early, and did not scale when more
than one NPC mattered in a scene.

## Design goals

1. Show only information the player has plausibly learned.
2. Support scenes with 0, 1, or multiple relevant NPCs.
3. Keep the panel readable while the story/action loop stays primary.
4. Preserve pacing: the panel should support choices, not replace prose.

## Proposed model

Treat the right sidebar as two stacked modules:

1. **Player module (always visible)**
   - Name, core stats, arousal/alcohol (current behavior).

2. **People module (contextual)**
   - Header: `People Here`.
   - List of NPC chips/cards for NPCs currently present in the active scene.
   - Selecting a card opens a compact detail pane for that NPC.

## NPC card information hierarchy

Always show:
- Display name
- Relationship label (stranger/coworker/friend/partner/etc.)

Show when known:
- Role tags (e.g. `coworker`, `barista`) only after role is set in scene history
- Liking/Attraction as qualitative bands (not raw numbers):
  - `Cold`, `Neutral`, `Warm`, `Very Warm`
  - `Uninterested`, `Curious`, `Interested`, `Intense`

Never show:
- Hidden internal values
- Traits or metadata the player has not learned

## Interaction details

- If no NPC is present: show muted `No one else is in focus.`
- If one NPC is present: auto-select that NPC.
- If multiple NPCs are present:
  - Horizontal chip row on top of People module
  - Keyboard navigation with Left/Right
  - Enter/Space to select chip
- Hovering an action can temporarily highlight the NPC it affects (future enhancement,
  optional once action metadata supports target hints).

## Information timing rules

NPC appears in People module only after one of:
- NPC is active in engine (`NpcActivated`)
- A scene line explicitly introduces them

Liking/Attraction bands update only after the player has had at least one interaction
with that NPC (prevents omniscient UI on first sight).

## Visual structure

- Keep sidebar width at 280 for now.
- Use section headers with thin separators.
- Use compact cards (name + 1 line meta) to avoid vertical bloat.
- Detail pane sits below chips and can collapse when no selection.

## Implementation phases

### Phase 1 (safe MVP)
- Add `People Here` section with current active NPC only.
- No raw numeric liking/attraction.
- Keep existing player stats unchanged.

### Phase 2
- Track and render multiple present NPCs.
- Add selection chips and detail pane.

### Phase 3
- Add per-action target highlight and richer known-info progression.

## Data/engine needs

UI will need a stable "present NPC set" source. Options:
- Extend scene events with `NpcPresentSet(Vec<NpcKey>)`, or
- Add explicit enter/exit effects and maintain a UI-visible set in world state.

Recommended: event-based set from scene engine (clear ownership, easy to test).

## Test plan

1. **No leakage:** unmet NPCs do not appear.
2. **Single NPC:** card auto-selects and renders.
3. **Multi NPC:** selection switches details correctly.
4. **State transition:** switching scenes updates People module without stale entries.
5. **Accessibility:** keyboard navigation works for chips and selected detail.
