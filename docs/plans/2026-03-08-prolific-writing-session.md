# Prolific Writing Session — "The Game Proves Its Premise"

**Date:** 2026-03-08
**Goal:** ~15-20 new scenes across 4 tracks. Adult content, NPC deepening, post-arc variety.
**Method:** Autonomous session using scene-writer + writing-reviewer agents in parallel batches.

## Design Principles

- Every intimate scene shows transformation texture — body surprising her, responses she didn't expect
- Jake = tenderness + discovery. Marcus = transgression + urgency. Strangers = unpredictability + loss of control.
- FEMININITY gates: low (<30) = disorientation. Higher = fluency but new complications.
- Trait branches change WHAT HAPPENS, not adjectives.
- All scenes must have 2-4 meaningful player choices.
- CisMale→Woman only. No AlwaysFemale branches.

## Track 1 — Jake Romance Arc (5 scenes)

Jake is the player-driven romantic thread. Coffee shop acquaintance → dates → first intimacy.

| Scene | Prereqs | Register | Confidence |
|---|---|---|---|
| `jake_first_date` | MET_JAKE | Nervous energy, body awareness | HIGH — natural next step |
| `jake_second_date` | JAKE_FIRST_DATE | Comfort growing, physical contact | HIGH |
| `jake_apartment` | JAKE_SECOND_DATE | First time in this body. Explicit. | MEDIUM — tone is critical |
| `jake_morning_after` | JAKE_INTIMATE | Waking up next to someone | HIGH |
| `jake_text_messages` | JAKE_FIRST_DATE | Phone intimacy, lower stakes | MEDIUM — format unusual |

### Notes
- Jake's personality needs documenting (no character doc exists). Infer from coffee_shop + coffee_shop_return scenes.
- The `jake_apartment` scene is the most creatively sensitive. Must balance explicit content with transformation lens.
- Progressive flags: MET_JAKE → JAKE_FIRST_DATE → JAKE_SECOND_DATE → JAKE_INTIMATE.

## Track 2 — Marcus Workplace Tension (4 scenes)

Marcus is the "this wasn't supposed to happen" thread. Professional → blurred → explicit.

| Scene | Prereqs | Register | Confidence |
|---|---|---|---|
| `work_marcus_late` | MET_MARCUS, arc≥working | Working late, proximity | HIGH |
| `work_marcus_drinks` | MET_MARCUS, arc≥settled | After-work drinks | HIGH |
| `work_marcus_closet` | MARCUS_DRINKS | Line crosses. Explicit. | MEDIUM — workplace setting tricky |
| `work_marcus_aftermath` | MARCUS_INTIMATE | Next day at work | HIGH — consequences are good writing |

### Notes
- Marcus has some character info in robin.md. Review before writing.
- The workplace setting adds stakes — professional reputation, being seen, power dynamics.
- Marcus thread should feel fundamentally different from Jake — less romantic, more situational/charged.

## Track 3 — Stranger Encounters (4 scenes)

Pure unpredictability. The world acting on her. Core erotic logic.

| Scene | Prereqs | Register | Confidence |
|---|---|---|---|
| `bar_closing_time` | week≥2 | Late bar, walk home, door moment | HIGH |
| `party_invitation` | week≥3 | House party. Alcohol + strangers | HIGH |
| `laundromat_night` | week≥2 | Mundane → charged. Stranger attention. | MEDIUM — needs strong hook |
| `subway_late` | week≥2 | Late transit. Unwanted attention. Shame/desire. | LOW — hardest to write without being exploitative |

### Notes
- These are the hardest scenes to write well. The line between "unpredictability as erotic" and "assault" must be clear.
- Player agency matters even in "world exceeds her choices" scenes — she always has meaningful options.
- `subway_late` is the riskiest. May want user review of spec before writing.

## Track 4 — Content Deepening (4-5 scenes)

Expand thin spots and add post-arc variety.

| Scene | Type | Register | Confidence |
|---|---|---|---|
| `weekend_morning` | free_time | Different morning, slower pace | HIGH |
| `landlord_repair` | free_time | Power dynamic + domestic space | MEDIUM — landlord personality undefined |
| `shopping_mall` | free_time | Body awareness in public | HIGH |
| `workplace_work_meeting` | EXPAND existing | Add 2-3 action branches | HIGH |
| `workplace_evening` | EXPAND existing | Add choices | HIGH |

### Notes
- Expansions of existing scenes are safest — structure exists, just needs more branches.
- New free_time scenes should feel distinct from the 8 that exist. Different locations, different tensions.

## Batch Execution Plan

### Batch 1 — Foundation (parallel: 3-4 agents)
- Write Jake character doc (from existing scene analysis)
- `jake_first_date` (scene-writer)
- `work_marcus_late` (scene-writer)
- `bar_closing_time` (scene-writer)
- Review all three (writing-reviewer)

### Batch 2 — Escalation (parallel: 3-4 agents)
- `jake_second_date` (scene-writer)
- `work_marcus_drinks` (scene-writer)
- `party_invitation` (scene-writer)
- `weekend_morning` (scene-writer)
- Review all four

### Batch 3 — Explicit (parallel: 3-4 agents)
- `jake_apartment` (scene-writer)
- `work_marcus_closet` (scene-writer)
- `laundromat_night` (scene-writer)
- `shopping_mall` (scene-writer)
- Review all four

### Batch 4 — Resolution + Expansion (parallel: 3-4 agents)
- `jake_morning_after` (scene-writer)
- `jake_text_messages` (scene-writer)
- `work_marcus_aftermath` (scene-writer)
- Expand `workplace_work_meeting` + `workplace_evening`
- Review all

### Batch 5 — Stretch goals if time permits
- `subway_late` (needs careful spec)
- `landlord_repair`
- Additional trait-specific branches in existing scenes
- Campus arc calibration (7 scenes)

## Schedule Integration

All new scenes need entries in `packs/base/scenes/schedule.toml`:
- Jake scenes: free_time slot, gated on progressive flags
- Marcus scenes: work slot, gated on arc state + flags
- Stranger scenes: free_time slot, gated on week count
- New universal scenes: free_time slot, week≥1

## Confidence Summary

- **HIGH confidence:** Jake arc progression, Marcus workplace tension, post-arc variety scenes, existing scene expansion
- **MEDIUM confidence:** Explicit scene tone (jake_apartment, work_marcus_closet), landlord characterization, laundromat hook
- **LOW confidence:** subway_late (exploitation risk), jake_text_messages format

## Post-Session Deliverables

1. All new scene TOML files in `packs/base/scenes/`
2. Updated `schedule.toml` with all new entries
3. Jake character doc in `docs/characters/jake.md`
4. Session log in HANDOFF.md with per-scene confidence ratings
5. Writing-reviewer audit results for every scene
6. List of scenes that need user creative review before shipping
