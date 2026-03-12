# Player Agency Audit Results

**Date:** 2026-03-12
**Total findings:** 79 → **0 remaining** (after Phase 2 Tasks 5–7)
**Scenes affected:** 23 → **0 remaining** of 54

## Phase 2 Task 5 — Completed (Tier 1 Critical)

Four scenes rewritten. 21 findings fixed. Writing-reviewed and committed.

| Scene | Findings fixed | Status |
|---|---|---|
| `workplace_arrival` | 5 (2 speech + 3 action) | Clean |
| `workplace_first_day` | 7 (2 speech + 5 action) | Clean |
| `workplace_first_night` | 6 (5 action + 1 thought) | Clean |
| `morning_routine` | 3 (3 action) | Clean |

## Phase 2 Tasks 6–7 — Completed (Tier 1 Remaining + Tier 2 + Tier 3)

19 scenes rewritten. 58 findings fixed. Automated audit passes with 0 findings.

### High-density scenes (3 scenes, 17 findings)

| Scene | Findings fixed | Status |
|---|---|---|
| `campus_arrival` | 6 (speech + extended autopilot) | Clean |
| `campus_call_home` | 5 (speech + autopilot + phone actions) | Clean |
| `jake_text_messages` | 6 (pick up phone + type/delete) | Clean |

### Medium-density scenes (5 scenes, 20 findings)

| Scene | Findings fixed | Status |
|---|---|---|
| `campus_orientation` | 7 (2 speech + 5 action) | Clean |
| `coffee_shop` | 4 (look/catch eye/nod/stand) | Clean |
| `workplace_first_clothes` | 4 (find store/escalator/stand/follow) | Clean |
| `weekend_morning` | 4 (stretch ×4 reframed to "A stretch") | Clean |
| `shopping_mall` | 2 (head for store + speech) | Clean |

### Low-density scenes (10 scenes, 16 findings)

| Scene | Findings fixed | Status |
|---|---|---|
| `work_marcus_aftermath` | 2 (walk to desk + speech) | Clean |
| `bar_closing_time` | 1 (step through) | Clean |
| `campus_dining_hall` | 2 (pick up tray + get harissa) | Clean |
| `campus_dorm` | 1 (know what you want to do) | Clean |
| `campus_library` | 3 (look back at notes ×3) | Clean |
| `campus_study_session` | 2 (put pen down + look back) | Clean |
| `coffee_shop_return` | 1 (get in line) | Clean |
| `laundromat_night` | 1 (you look up → eyes up) | Clean |
| `transformation_intro` | 1 (you sit, wait) | Clean |
| `workplace_evening` | 2 (pull out laptop + stand in kitchen) | Clean |

### Also fixed (not in original audit count)

| Scene | Extra fixes |
|---|---|
| `work_marcus_favor` | 2 (open spec + chest reaction — borderline resolved) |
| `campus_orientation` | 2 extra (line 14 speech + line 18 sip water) |

## Tier 3 — Borderline (all resolved)

All borderline cases were reframed:
- `coffee_shop` "You catch his eye" → "Eye contact — his or yours first"
- `work_marcus_favor` "You open the spec" → "The spec is open"
- `weekend_morning` "You stretch" → "A stretch"
- `laundromat_night` "You look up" → "eyes up, automatic"

## Automated Audit Status

`cargo test player_agency_audit_report` → **0 findings** across all 54 scenes.
