# Player Agency Audit Results

**Date:** 2026-03-12
**Total findings:** 79 (8 speech, 71 action)
**Scenes affected:** 23 of 54

## Tier 1 — Critical: Player Speech in Intro (8 findings)

Player dialogue that should be moved to action buttons or removed.

| Scene | Line | Violation |
|---|---|---|
| `campus_arrival` | 27 | `You say thank you.` |
| `campus_orientation` | 25 | `Adam asks what you're studying. You tell him.` |
| `campus_orientation` | 34 | `You say time management.` |
| `shopping_mall` | 27 | `You say "just looking"` |
| `workplace_arrival` | 21 | `"Thanks." You take the bag` (ANALYTICAL branch) |
| `workplace_arrival` | 25 | `"Thanks." You take the bag and keep moving` (default branch) |
| `workplace_first_day` | 36 | `You say your name before he can decide what expression to wear.` |
| `work_marcus_aftermath` | 26 | `He says good morning. You say it back.` |

## Tier 2 — High: Player Deliberate Actions in Intro (71 findings)

Player taking deliberate actions before any choice button is presented.

### Highest density scenes (5+ findings — major restructure needed)

| Scene | Count | Key violations |
|---|---|---|
| `campus_arrival` | 6 | Extended autopilot: walk, take lanyard, smile, keep walking |
| `campus_call_home` | 5 | Extended autopilot: hang up, sit, open mouth, pick up phone |
| `jake_text_messages` | 6 | Pick up phone, type response, delete it — all player decisions |
| `workplace_first_day` | 5 | Get off bus, take badge, stand up, grab bag, get set up |
| `workplace_first_night` | 5 | Set carry-on down, text shipping company, add to list, sit |

### Medium density (3–4 findings)

| Scene | Count | Key violations |
|---|---|---|
| `coffee_shop` | 4 | Look at menu, catch his eye, nod back, stand with bag |
| `morning_routine` | 3 | Get yourself together, grab sweater, get dressed |
| `campus_orientation` | 3 | Sit at table, sip water, add to list |
| `workplace_arrival` | 3 | Stand, reach for carry-on, take bag |
| `workplace_first_clothes` | 4 | Stand still, follow, put on, fingers find clasp |
| `weekend_morning` | 4 | Stretch (4 variants) |

### Low density (1–2 findings — minor fixes)

| Scene | Count | Key violations |
|---|---|---|
| `bar_closing_time` | 1 | You step through |
| `campus_dining_hall` | 2 | Pick up tray, get the harissa |
| `campus_dorm` | 1 | You know what you want to do |
| `campus_library` | 2 | You look back at notes |
| `campus_study_session` | 2 | Put pen down, look back |
| `coffee_shop_return` | 1 | Get in line |
| `laundromat_night` | 1 | You look up |
| `shopping_mall` | 1 | Head for anchor store |
| `transformation_intro` | 1 | You sit, wait |
| `workplace_evening` | 2 | Pull out laptop, stand in kitchen |
| `work_marcus_aftermath` | 1 | Walk to desk |
| `work_marcus_favor` | 2 | Open spec, chest does thing (borderline) |

## Tier 3 — Borderline (needs case-by-case review)

The following findings may be acceptable involuntary/experiential responses
flagged by the deliberate verb heuristic:

- `coffee_shop.toml:21` — `You catch his eye` — could be involuntary
- `work_marcus_favor.toml:20` — `Your chest does a small thing` then `You open the spec` — mixed
- `weekend_morning.toml` — `You stretch` — borderline between deliberate and involuntary body action
- `laundromat_night.toml:22` — `You look up` — reflexive response to door opening

## Scenes with zero findings (31 scenes)

These scenes either have no intro prose or their intros correctly describe the world
without player speech or deliberate actions. They are not listed here.
