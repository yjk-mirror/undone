# Sprint 3: Robin's Playable Loop

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the minimum viable ongoing content loop for the Robin route — 7 work slot scenes, 5 free_time scenes, and 4 NPC follow-up scenes — so there is a real rotating pool of content after the workplace arc ends.

**Architecture:** All scenes are TOML content files in `packs/base/scenes/` using Minijinja templates. The `work` slot is a new schedule.toml slot that fires when `workplace_opening` arc state is `settled`. Free_time scenes extend the existing 3-scene pool. NPC liking conditions use string comparison (`m.getLiking() == 'Ok'`) — no engine changes needed. All scenes use CisMale→Woman only: `{% if not w.alwaysFemale() %}` blocks, no `{% else %}` branches.

**Tech Stack:** TOML/Minijinja scene files, `scene-writer` custom agent (`subagent_type: scene-writer`), `mcp__minijinja__jinja_validate_template`, `cargo test`, `validate-pack`

---

## Reference: Scene File Conventions

Read `packs/base/scenes/workplace_work_meeting.toml` as the primary style reference — it has the fullest structure: intro, intro_variants, thoughts, NPC-gated actions, trait branches, and effects. Also read `packs/base/scenes/coffee_shop.toml` for free_time scene structure (scene flags, NPC actions, allow_npc_actions).

Every scene file needs:
- `id = "base::<scene_id>"`
- `description = "..."` (one sentence for the scheduler/UI)
- `slot = "<slot_id>"`
- `[[actions]]` with at least 2 choices
- All prose in Minijinja — second-person present tense, no "she" narration
- Transformation content in `{% if not w.alwaysFemale() %}` blocks only, no `{% else %}`

## Reference: schedule.toml Format

Read `packs/base/data/schedule.toml` before editing. The current file has three slots: `free_time`, `workplace_opening`, `campus_opening`. Add the new `work` slot as a fourth slot block.

Event fields: `scene_id`, `weight`, `condition` (optional), `once_only` (optional boolean), `trigger` (optional boolean — if true, fires before weighted picks).

## Reference: Writing Guide

Read `docs/writing-guide.md` before writing any prose. Key rules:
- BG3 narrator voice: dry, plain, trusts the scene
- Anti-patterns: staccato closers, em-dash reveals, over-naming experiences, anaphoric repetition
- Inner voice (thoughts) uses `style = "inner_voice"` and italic prose

---

## Task 1: Work Slot Integration Test + schedule.toml Scaffold

**Files:**
- Modify: `crates/undone-scene/src/lib.rs`
- Modify: `packs/base/data/schedule.toml`

**Step 1: Write the failing test**

Add this test to `crates/undone-scene/src/lib.rs` after the existing `femininity_reaches_25_by_workplace_arc_end` test:

```rust
#[test]
fn work_slot_fires_when_settled() {
    use undone_world::GameData;
    let (pack_registry, mut world) = make_world();
    // Set arc to settled state (post-arc)
    world.game_data.set_game_flag("ROUTE_WORKPLACE");
    world.game_data.arc_states.insert("base::workplace_opening".to_string(), "settled".to_string());
    world.game_data.week = 3;
    // Pick several events — at least one should come from the work slot
    let mut got_work_scene = false;
    for _ in 0..20 {
        if let Some(event) = world.scheduler.pick_next(&world.game_data, &pack_registry) {
            if event.scene_id.contains("work_") {
                got_work_scene = true;
                break;
            }
        }
    }
    assert!(got_work_scene, "No work scene fired after 20 picks in settled state");
}
```

**Step 2: Run test to verify it fails**

```bash
cargo test -p undone-scene work_slot_fires_when_settled -- --nocapture
```

Expected: FAIL with "No work scene fired after 20 picks in settled state" (work slot not yet wired)

**Step 3: Add the work slot to schedule.toml**

Open `packs/base/data/schedule.toml`. After the `workplace_opening` slot block, add a new `work` slot. Use the existing slot format (read the file first to confirm exact TOML structure):

```toml
[[slots]]
id = "work"

  # Work scenes fire when the workplace arc has reached 'settled'
  [[slots.events]]
  scene_id = "base::work_standup"
  weight = 10
  condition = "gd.arcState('base::workplace_opening') == 'settled' && gd.isWeekday()"

  [[slots.events]]
  scene_id = "base::work_lunch"
  weight = 10
  condition = "gd.arcState('base::workplace_opening') == 'settled'"

  [[slots.events]]
  scene_id = "base::work_late"
  weight = 8
  condition = "gd.arcState('base::workplace_opening') == 'settled'"

  [[slots.events]]
  scene_id = "base::work_corridor"
  weight = 12
  condition = "gd.arcState('base::workplace_opening') == 'settled'"

  [[slots.events]]
  scene_id = "base::work_friday"
  weight = 8
  condition = "gd.arcState('base::workplace_opening') == 'settled' && gd.isWeekday()"

  # Marcus follow-up scenes — gated by having had the work meeting
  [[slots.events]]
  scene_id = "base::work_marcus_coffee"
  weight = 9
  condition = "gd.arcState('base::workplace_opening') == 'settled' && gd.hasGameFlag('FIRST_MEETING_DONE')"

  [[slots.events]]
  scene_id = "base::work_marcus_favor"
  weight = 7
  condition = "gd.arcState('base::workplace_opening') == 'settled' && gd.hasGameFlag('FIRST_MEETING_DONE') && m.getLiking() == 'Ok'"
```

**Step 4: Write a minimal stub for `work_standup.toml` so the test can pass**

Create `packs/base/scenes/work_standup.toml` with a minimal stub (just enough to load):

```toml
id = "base::work_standup"
description = "Monday standup. Status updates, scope questions, Marcus."
slot = "work"

prose = """
The standup circle forms at 9:05. Seven people. You give your update.
"""

[[actions]]
id = "stand_back"
label = "Listen to the others"
prose = """
Marcus gives his infrastructure update. You listen.
"""

  [[actions.effects]]
  type = "skill_increase"
  skill = "FEMININITY"
  amount = 2
```

**Step 5: Run validate-pack to verify the scene and slot load**

```bash
cargo run --bin validate-pack
```

Expected: `Validation passed. 20 scenes loaded.` (no errors)

**Step 6: Run the test to verify it passes**

```bash
cargo test -p undone-scene work_slot_fires_when_settled -- --nocapture
```

Expected: PASS

**Step 7: Commit the scaffold**

```bash
git add packs/base/data/schedule.toml packs/base/scenes/work_standup.toml crates/undone-scene/src/lib.rs
git commit -m "feat: work slot scaffold — schedule.toml + integration test"
```

---

## Task 2: Work Scenes — Standup + Lunch

Replace the `work_standup.toml` stub with a full scene, and write `work_lunch.toml`. Use the `scene-writer` agent for each (subagent_type: `scene-writer`).

**Files:**
- Modify: `packs/base/scenes/work_standup.toml` (replace stub with full scene)
- Create: `packs/base/scenes/work_lunch.toml`

### Scene: work_standup.toml

**Scene brief for scene-writer:**

ID: `base::work_standup`. Slot: `work`. Description: "Monday standup. Seven people. Someone explains your work back to you again."

**Intro:** The standup circle, 9:05 AM. Status updates go around. Default intro shows the weight of performing competence in a group. ANALYTICAL trait variant: you clock the room's attention distribution — who looks at their phone, who watches you. Low-FEMININITY (<20) intro_variant: you're still calibrating where to stand in a circle of colleagues.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 35` — the observation that you hold yourself slightly smaller than you used to. You used to take up more space in these circles. You notice this without deciding what it means.

**Actions (3):**
1. `give_update` (default, always available): You give your status. Straight, clean, no hedging. Marcus nods once. Kevin doesn't look up from his laptop. +2 FEMININITY. Trait branches: ANALYTICAL adds a note about tracking who heard you vs. who was waiting for their turn; CONFIDENT gets a beat where Kevin finally looks up.
2. `keep_it_brief` (always available): Minimum viable update. You're done in thirty seconds. The circle moves on. Safer. +1 FEMININITY. `{% if not w.alwaysFemale() %}` thought: you used to take longer. Not because you had more to say.
3. `ask_a_question` (always available): You ask Marcus something about his infrastructure update. He answers directly — a real answer. +2 FEMININITY. Sets game flag: `MARCUS_DIALOGUE_1`. `add_npc_liking(npc='m', delta=1)`.

**No once_only** — repeatable scene.

---

### Scene: work_lunch.toml

**Scene brief for scene-writer:**

ID: `base::work_lunch`. Slot: `work`. Description: "The office lunch calculation. Where you eat, who you eat with, what that means."

**Intro:** 12:15. The office energy shifts. People start moving. The question of what to do with forty-five minutes. Default intro: you wait a beat to see which way the current goes. Low-FEMININITY (<20) intro_variant: you don't know the lunch patterns yet. You watch which clusters form and which don't.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 40` — the observation that you used to pick up whatever was in the conference room or eat at your desk without thinking about it. The question of lunch never had social texture before.

**Actions (4):**
1. `eat_at_desk` (always available): You order through the Slack channel. Eat at your desk. Nobody comments. You get an hour of uninterrupted work. +0 FEMININITY, -5 stress. SHY trait: this is actively comfortable, not just efficient.
2. `go_out_alone` (always available): You find a place two blocks away, get a table to yourself, and read your phone. The relief of being anonymous in a restaurant. +1 FEMININITY. `{% if not w.alwaysFemale() %}` note: being a woman alone in a lunch spot is different from being a man alone. Less assumption of defeat.
3. `join_the_group` (always available): Someone from your team asks if you're coming. You go. It's fine. The conversation is about a project you're tangentially connected to. +1 FEMININITY, +5 stress. Default: you nod at the right moments. ANALYTICAL: you're mapping the social geometry.
4. `ask_marcus` (condition: `gd.hasGameFlag('FIRST_MEETING_DONE') && !gd.hasGameFlag('LUNCH_WITH_MARCUS')`): You catch Marcus in the kitchen and ask if he's going out. He says sure, like it wasn't a question. Lunch becomes a real conversation. +2 FEMININITY. Sets `LUNCH_WITH_MARCUS`. `add_npc_liking(npc='m', delta=1)`.

**No once_only** — repeatable scene (except the `ask_marcus` action which is gated by flag).

**Step 1: Dispatch scene-writer agent for work_standup (full rewrite)**

Dispatch a `scene-writer` subagent with this prompt:
```
Write the full scene file for packs/base/scenes/work_standup.toml.
[Paste the scene brief above]
Read docs/writing-guide.md first. Read packs/base/scenes/workplace_work_meeting.toml as a structural reference.
Replace the existing stub entirely with the full scene.
Validate with mcp__minijinja__jinja_validate_template after writing.
Report all prose blocks validated.
```

**Step 2: Dispatch scene-writer agent for work_lunch (new file)**

Same pattern — separate scene-writer subagent for work_lunch.toml.

**Step 3: Validate both templates**

After agents complete, validate each prose block:
```
mcp__minijinja__jinja_validate_template for each {% ... %} block in both files
```

**Step 4: Run validate-pack + tests**

```bash
cargo run --bin validate-pack && cargo test -p undone-scene
```

Expected: `Validation passed. 21 scenes loaded.` Tests: 221 passing.

**Step 5: Commit**

```bash
git add packs/base/scenes/work_standup.toml packs/base/scenes/work_lunch.toml
git commit -m "feat: work scenes — standup and lunch"
```

---

## Task 3: Work Scenes — Late + Corridor + Friday

**Files:**
- Create: `packs/base/scenes/work_late.toml`
- Create: `packs/base/scenes/work_corridor.toml`
- Create: `packs/base/scenes/work_friday.toml`

### Scene: work_late.toml

**Scene brief for scene-writer:**

ID: `base::work_late`. Slot: `work`. Description: "Staying past six. The office empties. You work alone — or nearly alone."

**Intro:** 6:15 PM. The ceiling lights have dimmed to efficiency mode. Most desks dark. The HVAC cycling differently when no one else is there to notice it. Default intro: the particular quiet of a late office, and the work that goes better without people. Low-FEMININITY (<20) intro_variant: the late office is one of the few places where nothing about you reads differently yet. You're just someone staying late.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 35` — you always stayed late. It was a thing about you, a marker. You wonder if the same thing still makes you stay, or if it's different now.

**Actions (3):**
1. `work_through` (always available): Head down. You finish what you started. At 8 PM you close your laptop and feel the quiet satisfaction of output. +2 FEMININITY, -10 stress. ANALYTICAL trait: notes that being a woman in an empty office past dark is a different calibration than being a man.
2. `leave_at_six_thirty` (always available): You hit a natural stopping point and go. The responsible choice. +0 FEMININITY. `{% if not w.alwaysFemale() %}`: you leave before the office empties completely. You notice you made this choice.
3. `someone_is_still_here` (condition: `gd.hasGameFlag('FIRST_MEETING_DONE')`): Marcus is at his desk. You pass on your way to the elevator. He says something about the deadline without looking up. You respond. Something shifts in the frequency between you — not attraction, just recognition. +2 FEMININITY. `add_npc_liking(npc='m', delta=1)`. Sets `LATE_OFFICE_MARCUS`.

---

### Scene: work_corridor.toml

**Scene brief for scene-writer:**

ID: `base::work_corridor`. Slot: `work`. Description: "A hallway exchange that lands differently than it should."

**Intro:** The main corridor between engineering and the kitchen. You're going for water or returning from a meeting. A short scene — not important. But small exchanges accumulate. Default intro: two people navigating a narrow corridor. The micro-choreography of yielding. Low-FEMININITY (<20) intro_variant: you still sometimes default to the wrong pattern in a corridor — the one where you don't yield.

**Actions (3):**
1. `pass_kevin` (always available): Kevin Marsh in the hallway. He nods. You nod. Something in the exchange lands correctly — colleague recognition, no more. But you notice it landed correctly, and notice that you noticed. +1 FEMININITY. `{% if not w.alwaysFemale() %}` thought: competence registers differently when people have no prior version of you to compare to.
2. `overhear_something` (always available): Two coworkers around the corner, not aware you're there. They're talking about the hiring process, or a promotion, or someone's performance review. It's not about you. But it's data. +1 FEMININITY. ANALYTICAL trait: you file it systematically. Default: you keep walking.
3. `small_talk_wins` (condition: `gd.hasGameFlag('FIRST_MEETING_DONE')`): Marcus is coming the other way, arms full of printouts. He makes a dry remark about the printer room. You make one back. It takes six seconds. +1 FEMININITY. `add_npc_liking(npc='m', delta=1)`.

---

### Scene: work_friday.toml

**Scene brief for scene-writer:**

ID: `base::work_friday`. Slot: `work`. Description: "Friday 4pm. Someone wants drinks. The decision."

**Intro:** Friday afternoon slack. The channel message is from someone in DevRel: drinks at Corrigan's, 5pm, all welcome. Default intro: you read the message and clock the question underneath it — are you someone who goes to these? Low-FEMININITY (<20) intro_variant: you've been here a month. You don't know who goes to these. You don't know if you're included in "all welcome" in the same way you would have been before.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 40` — you went to things like this without thinking about it. The decision to go somewhere was never a calibration. You're recalibrating.

**Actions (3):**
1. `go_to_drinks` (always available): You go. Corrigan's is a block away. You position yourself at the bar rather than a booth — easier to talk to people on your terms, easier to leave. Someone you don't know very well asks what you're working on. +2 FEMININITY. CONFIDENT trait: you find the right spot immediately and the evening unfolds from there. SHY: you stay thirty minutes and then have a graceful exit.
2. `skip_with_excuse` (always available): You message back: "another time, deadline." +0 FEMININITY, -5 stress. SHY trait: this is the right call. `{% if not w.alwaysFemale() %}` thought: you used to be someone who didn't have to think about whether to go to a thing.
3. `go_with_marcus` (condition: `gd.hasGameFlag('FIRST_MEETING_DONE') && m.getLiking() == 'Ok'`): Marcus is going. He mentions it in passing, not as an invitation. You end up walking over together. The bar conversation is easy — not personal, just peers who've found their shorthand. +2 FEMININITY. `add_npc_liking(npc='m', delta=1)`. Sets `DRINKS_WITH_MARCUS`.

**Step 1: Dispatch 3 parallel scene-writer agents**

Run three scene-writer agents simultaneously — one per scene. Each agent:
- Reads `docs/writing-guide.md`
- Reads `packs/base/scenes/workplace_work_meeting.toml` (structural reference)
- Writes the scene file
- Validates all prose blocks with `mcp__minijinja__jinja_validate_template`

**Step 2: Validate-pack + tests**

```bash
cargo run --bin validate-pack && cargo test -p undone-scene
```

Expected: `Validation passed. 23 scenes loaded.` Tests: 221 passing.

**Step 3: Commit**

```bash
git add packs/base/scenes/work_late.toml packs/base/scenes/work_corridor.toml packs/base/scenes/work_friday.toml
git commit -m "feat: work scenes — late, corridor, friday"
```

---

## Task 4: Marcus Follow-Up Scenes

**Files:**
- Create: `packs/base/scenes/work_marcus_coffee.toml`
- Create: `packs/base/scenes/work_marcus_favor.toml`

### Scene: work_marcus_coffee.toml

**Scene brief for scene-writer:**

ID: `base::work_marcus_coffee`. Slot: `work`. Description: "Break room, Tuesday. Marcus asks what you thought of Kevin's presentation."

**Condition in schedule.toml:** `gd.arcState('base::workplace_opening') == 'settled' && gd.hasGameFlag('FIRST_MEETING_DONE')`

**Intro:** 10:30 AM, the break room. You're making your second coffee. Marcus comes in. He pours himself a cup without ceremony. Then: "What did you think of Kevin's deck?" — not a management question. He actually wants to know. Default intro: the slight recalibration of being asked a real question about work by someone who isn't testing you. Low-FEMININITY (<20) intro_variant: you're still surprised when colleagues ask your opinion as a first move. You file this.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 35` — Marcus asks you things like a person asks another person things. You notice the baseline has changed and you're still adjusting to it.

**Actions (3):**
1. `give_honest_take` (always available): You tell him what you actually think — the deck buried the risk section and Kevin knows it. Marcus's mouth does a thing. "Yeah," he says. A real exchange. +2 FEMININITY. `add_npc_liking(npc='m', delta=1)`. ANALYTICAL trait gets a sharper read of Marcus's expression and what it confirms.
2. `hedge` (always available): You say something measured. He nods, gets his coffee, leaves. Professional. +0 FEMININITY. `{% if not w.alwaysFemale() %}` thought: you used to have opinions faster than this. The hedging is new. You're not sure if it's caution or something else.
3. `turn_it_back` (always available): "What did *you* think?" He considers this for longer than expected. Then gives you an honest answer. Better than the one you would've given. +2 FEMININITY. `add_npc_liking(npc='m', delta=1)`. Sets `MARCUS_REAL_CONVERSATION`.

---

### Scene: work_marcus_favor.toml

**Scene brief for scene-writer:**

ID: `base::work_marcus_favor`. Slot: `work`. Description: "Marcus messages: can you look at this spec? Not a big ask. Just an ask."

**Condition in schedule.toml:** `gd.arcState('base::workplace_opening') == 'settled' && gd.hasGameFlag('FIRST_MEETING_DONE') && m.getLiking() == 'Ok'`

**Intro:** A Slack message from Marcus at 2pm: "Hey — got 10 minutes to look at something?" The spec is a twelve-page infrastructure document. The ask is real — he wants actual feedback. Default intro: the particular satisfaction of being asked for competence directly. Low-FEMININITY (<20) intro_variant: you read the message twice to make sure you're reading it right. An ask, not an assignment.

**Actions (3):**
1. `review_and_comment` (always available): You spend 40 minutes on it. Your comments are specific — two structural concerns, one thing he's right about that nobody else will catch. He responds the same day. +2 FEMININITY. `add_npc_liking(npc='m', delta=1)`. Sets `REVIEWED_MARCUS_SPEC`.
2. `quick_pass` (always available): You skim it, leave two comments, send it back in twenty minutes. Efficient. He says thanks. +1 FEMININITY. `add_npc_liking(npc='m', delta=1)`.
3. `decline` (always available): You're buried. You say so. He says no problem. +0 FEMININITY. Professional and honest. No liking change — it's a reasonable call.

**Step 1: Dispatch 2 parallel scene-writer agents**

One for each Marcus scene.

**Step 2: Validate-pack + tests**

```bash
cargo run --bin validate-pack && cargo test -p undone-scene
```

Expected: `Validation passed. 25 scenes loaded.`

**Step 3: Commit**

```bash
git add packs/base/scenes/work_marcus_coffee.toml packs/base/scenes/work_marcus_favor.toml
git commit -m "feat: Marcus follow-up work scenes"
```

---

## Task 5: Free_time Expansion — Bookstore + Park + Grocery

Add 3 new scenes to the `free_time` slot. Wire each into schedule.toml.

**Files:**
- Create: `packs/base/scenes/bookstore.toml`
- Create: `packs/base/scenes/park_walk.toml`
- Create: `packs/base/scenes/grocery_store.toml`
- Modify: `packs/base/data/schedule.toml` (add 3 events to free_time slot)

**Schedule.toml entries to add** (inside the existing `free_time` slot block):

```toml
  [[slots.events]]
  scene_id = "base::bookstore"
  weight = 8
  condition = "gd.week() >= 1"

  [[slots.events]]
  scene_id = "base::park_walk"
  weight = 10
  condition = "gd.week() >= 1"

  [[slots.events]]
  scene_id = "base::grocery_store"
  weight = 9
  condition = "gd.week() >= 1"
```

### Scene: bookstore.toml

**Scene brief for scene-writer:**

ID: `base::bookstore`. Slot: `free_time`. Description: "A secondhand bookstore. Browsing. Being addressed as 'miss.'"

**Intro:** The bookstore on Garfield Ave. You go because you want to look at books without anyone wanting anything from you. Default intro: the specific pleasure of browsing — no agenda, no queue. Low-FEMININITY (<20) intro_variant: you still sometimes feel like you're performing browsing rather than just doing it. You're not sure what you're supposed to look like doing this.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 40` — the clerk said "miss" when you asked about the biography section. It's not the first time. It still does something small and unresolved inside you.

**Actions (3):**
1. `browse_and_buy` (always available): You find something. The clerk wraps it in tissue paper without being asked. You overpay by two dollars and don't correct it. +1 FEMININITY. `{% if not w.alwaysFemale() %}` beat: the transaction was ordinary and unremarkable. You hold onto this for some reason.
2. `just_browse` (always available): You don't buy anything. You stay an hour. No one bothers you. +1 FEMININITY, -5 stress.
3. `talk_to_the_clerk` (condition: `!gd.hasGameFlag('BOOKSTORE_VISITED')`): The clerk has opinions about the biography section. Strong ones. You have a ten-minute argument about whether a certain kind of memoir counts as journalism. +1 FEMININITY. Sets `BOOKSTORE_VISITED`. `{% if not w.alwaysFemale() %}` note: the conversation happened without either of you navigating anything. Just two people in a bookstore with opinions.

---

### Scene: park_walk.toml

**Scene brief for scene-writer:**

ID: `base::park_walk`. Slot: `free_time`. Description: "Late afternoon. The park. Moving through outdoor space differently."

**Intro:** 4pm, the park near your apartment. You walk because the apartment gets small sometimes. Default intro: the park in late afternoon — the dogs, the joggers, the specific quality of urban outdoor light. Low-FEMININITY (<20) intro_variant: you're still recalibrating how you move in public space. The speed, the path choices, the eye contact rules.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 35` — you used to walk like you were going somewhere. You still are. But the walk itself is different. Something in the periphery has changed.

**Actions (3):**
1. `walk_the_loop` (always available): You do the full loop. A man jogs past, nods. You nod back. The exchange is nothing. +1 FEMININITY. `{% if not w.alwaysFemale() %}` thought: the nod between joggers used to be unremarkable. Now there's a register to it you didn't used to have access to.
2. `sit_for_a_while` (always available): You find a bench. You sit. Nobody asks why. +1 FEMININITY, -10 stress. SHY trait: this is actively restorative.
3. `run_into_someone` (condition: `gd.hasGameFlag('MET_JAKE')`): Jake, of all people. Walking a dog you didn't know he had. The coincidence is awkward for exactly one second and then it's fine. +1 FEMININITY. `add_npc_liking(npc='m', delta=1)`.

---

### Scene: grocery_store.toml

**Scene brief for scene-writer:**

ID: `base::grocery_store`. Slot: `free_time`. Description: "The weekly grocery run. Small navigations in domestic space."

**Intro:** The grocery store on Thursday evening. You have a list. You've been shopping here for two months. Default intro: the domestic routine of it — the specific pleasure of knowing where things are. Low-FEMININITY (<20) intro_variant: you're still making small adjustments. The weight distribution of the basket. The way you reach past someone at the dairy case without thinking about the contact zone first.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 35` — you used to shop fast. You've slowed down without deciding to. You take the long way around the produce section sometimes. You don't know why.

**Actions (3):**
1. `shop_the_list` (always available): You get what's on the list. A man reaches across you for the pasta. The contact zone calibration happens automatically now. +1 FEMININITY. `{% if not w.alwaysFemale() %}` beat: you moved slightly without being asked and it was fine.
2. `take_your_time` (always available): You're not in a hurry. You read labels you don't need to read. Someone asks if you know if the store stocks a specific thing. You do. +1 FEMININITY, -5 stress.
3. `checkout_exchange` (always available): The checkout clerk is efficient and kind. "Have a good night" lands differently than it used to — or the same. Hard to tell. +1 FEMININITY.

**Step 1: Dispatch 3 parallel scene-writer agents**

One per scene. Each reads `docs/writing-guide.md` and `packs/base/scenes/coffee_shop.toml` (free_time structure reference).

**Step 2: Wire into schedule.toml**

Add the 3 event entries to the `free_time` slot block.

**Step 3: Validate-pack + tests**

```bash
cargo run --bin validate-pack && cargo test -p undone-scene
```

Expected: `Validation passed. 28 scenes loaded.`

**Step 4: Commit**

```bash
git add packs/base/scenes/bookstore.toml packs/base/scenes/park_walk.toml packs/base/scenes/grocery_store.toml packs/base/data/schedule.toml
git commit -m "feat: free_time expansion — bookstore, park walk, grocery"
```

---

## Task 6: Free_time Expansion — Evening Home + Neighborhood Bar

**Files:**
- Create: `packs/base/scenes/evening_home.toml`
- Create: `packs/base/scenes/neighborhood_bar.toml`
- Modify: `packs/base/data/schedule.toml` (add 2 more events to free_time slot)

**Schedule.toml entries to add:**

```toml
  [[slots.events]]
  scene_id = "base::evening_home"
  weight = 12
  condition = "gd.week() >= 1"

  [[slots.events]]
  scene_id = "base::neighborhood_bar"
  weight = 7
  condition = "gd.week() >= 2"
```

### Scene: evening_home.toml

**Scene brief for scene-writer:**

ID: `base::evening_home`. Slot: `free_time`. Description: "A free evening with no agenda. What you do with it."

**Intro:** 7pm. You have the evening. Nobody expects anything. Default intro: the apartment in the evening — the question of what to do when nothing is required. Low-FEMININITY (<20) intro_variant: the free evening is still slightly strange. You're not sure what your defaults are yet. You used to know exactly what you did with unstructured time.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 40` — your evenings have changed shape. Not worse — different. There are things you do now that you didn't used to do and things you don't do anymore, and the ratio is still shifting.

**Actions (3):**
1. `watch_something` (always available): You find something to watch. An hour becomes two. Fine. +1 FEMININITY, -10 stress.
2. `read_or_work` (always available): You open a book or a side project. Quiet focus. The good kind of evening. +0 FEMININITY, -15 stress. ANALYTICAL: you get absorbed. AMBITIOUS: you finish something.
3. `spend_time_on_yourself` (always available): You do something with your appearance — try a different way with your hair, spend time with makeup without a specific occasion, try on the thing you bought and haven't worn yet. You look at yourself in the mirror for longer than usual. It's not vanity. It's something closer to rehearsal. +2 FEMININITY. `{% if not w.alwaysFemale() %}` block (all of this content is transformation-specific — wrap the whole action prose in `{% if not w.alwaysFemale() %}...{% endif %}`). For `alwaysFemale` players, replace with: "You spend the evening doing something for yourself."

Wait — don't add an alwaysFemale else branch. Per content rules: if the action prose is transformation-specific, the whole prose block goes in `{% if not w.alwaysFemale() %}`. If alwaysFemale players reach this action, they get blank prose and the effect — that's fine for deprioritized content. Or: write a non-transformation version as the default, and `{% if not w.alwaysFemale() %}` adds the transformation layer. Prefer the latter pattern.

Revised action prose pattern for `spend_time_on_yourself`:
- Default prose (works for any origin): You spend time on your appearance. No occasion. Just because.
- `{% if not w.alwaysFemale() %}` block: adds the specific beat of looking in the mirror and the "rehearsal" thought.

---

### Scene: neighborhood_bar.toml

**Scene brief for scene-writer:**

ID: `base::neighborhood_bar`. Slot: `free_time`. Description: "Going for a drink alone. The social read of a woman at a bar by herself."

**Intro:** A Tuesday evening, the bar two blocks from your apartment. You go because you want a drink and the apartment is quiet in the wrong way. Default intro: the positioning decision at a bar — stool at the bar vs. a small table. The read you do before you sit down. Low-FEMININITY (<20) intro_variant: you've done this before, from the other side of the dynamic. You ordered drinks in bars and didn't register the women sitting alone. You register it now from the other side.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 40` — you know exactly what a man in a bar thinks when he sees a woman sitting alone. You know it with uncomfortable precision. You drink your drink and decide what to do with that knowledge.

**Actions (3):**
1. `drink_and_decompress` (always available): You nurse your drink. The bartender makes conversation when the bar is slow. Nobody bothers you. +1 FEMININITY, -15 stress. `{% if not w.alwaysFemale() %}` note: being left alone in public is a thing you can engineer now. You've learned the body language.
2. `someone_buys_a_drink` (always available): A man at the bar asks if you want another. You assess. Default: you accept because the drink is free and the conversation is brief. CONFIDENT: you handle it easily and leave when you want. SHY: you decline politely. OBJECTIFYING trait: you see the transaction clearly from both sides. +2 FEMININITY. `{% if not w.alwaysFemale() %}` block for the interiority of receiving this kind of attention.
3. `talk_to_the_bartender` (always available): The bartender is efficient and dry and has no agenda. You spend an hour at the bar having a conversation about nothing in particular. +2 FEMININITY, -10 stress.

**Step 1: Dispatch 2 parallel scene-writer agents**

**Step 2: Wire into schedule.toml**

**Step 3: Validate-pack + tests**

```bash
cargo run --bin validate-pack && cargo test -p undone-scene
```

Expected: `Validation passed. 30 scenes loaded.`

**Step 4: Commit**

```bash
git add packs/base/scenes/evening_home.toml packs/base/scenes/neighborhood_bar.toml packs/base/data/schedule.toml
git commit -m "feat: free_time expansion — evening home, neighborhood bar"
```

---

## Task 7: Jake Follow-Up Scenes

**Files:**
- Create: `packs/base/scenes/coffee_shop_return.toml`
- Create: `packs/base/scenes/jake_outside.toml`
- Modify: `packs/base/data/schedule.toml` (add 2 events to free_time slot)

**Schedule.toml entries to add:**

```toml
  [[slots.events]]
  scene_id = "base::coffee_shop_return"
  weight = 9
  condition = "gd.week() >= 2 && gd.hasGameFlag('MET_JAKE')"

  [[slots.events]]
  scene_id = "base::jake_outside"
  weight = 7
  condition = "gd.week() >= 3 && gd.hasGameFlag('MET_JAKE') && (m.getLiking() == 'Ok' || m.getLiking() == 'Like' || m.getLiking() == 'Close')"
```

### Scene: coffee_shop_return.toml

**Scene brief for scene-writer:**

ID: `base::coffee_shop_return`. Slot: `free_time`. Description: "Back at the coffee shop. Jake is there. The question of whether you're going to be someone who acknowledges him."

**Condition:** Gated in schedule.toml by `MET_JAKE` flag.

**Intro:** The same coffee shop. You didn't come here for Jake — you came for coffee. But he's there, second table from the door, not looking up from his laptop. Default intro: the split-second calculation of whether to acknowledge someone you've met once, briefly, in a public space. Low-FEMININITY (<20) intro_variant: you know what you'd have done before. You'd have nodded once and gotten your coffee. You're not sure if that's still the right call.

**Actions (3):**
1. `acknowledge_him` (always available): You catch his eye on the way to the counter. He nods. You nod. On your way out he says "hey, you come here too?" — not a line, just a small recognition. +1 FEMININITY. `add_npc_liking(npc='m', delta=1)`.
2. `sit_near_him` (condition: `m.getLiking() == 'Ok'`): You've already met. You take the table next to his. The conversation happens organically. He's working on something he explains badly when you ask. You understand it better than he expects. +1 FEMININITY. `add_npc_liking(npc='m', delta=1)`. Sets `COFFEE_SHOP_SECOND_VISIT`.
3. `pretend_not_to_see_him` (always available): You get your coffee and find a seat at the back. It's not avoidance — it's a quiet morning. +0 FEMININITY. `{% if not w.alwaysFemale() %}` thought: you know he clocked you. This choice is also information.

---

### Scene: jake_outside.toml

**Scene brief for scene-writer:**

ID: `base::jake_outside`. Slot: `free_time`. Description: "Running into Jake on the street. A different context than the counter."

**Condition:** Gated by `MET_JAKE` and liking >= Ok in schedule.toml.

**Intro:** Lexington Ave, a Tuesday afternoon. Jake, going the other way, grocery bag in one hand, phone in the other. He sees you before you've decided whether you've seen him. Default intro: the specific pressure of running into someone whose name you know in an unstructured context. Low-FEMININITY (<20) intro_variant: coffee shop Jake was on a stool. Street Jake is taller than you expected, and you didn't used to have a frame for noticing that first.

**Thoughts:** `!w.alwaysFemale() && w.getSkill('FEMININITY') < 45` — you know what the dynamic is in these encounters when a man and a woman stop on the street. You know it from one side and are learning it from the other. They're the same dynamic. You see both edges of it.

**Actions (3):**
1. `stop_and_talk` (always available): You stop. He stops. The conversation lasts five minutes — where he's going, something about the neighborhood, a dry observation about the weather that lands. +1 FEMININITY. `add_npc_liking(npc='m', delta=1)`. CONFIDENT: you initiate and control the shape of it. Default: it finds its own shape. Sets `MET_JAKE_OUTSIDE`.
2. `quick_hi` (always available): A wave, a brief word, keep moving. Clean. +0 FEMININITY. Professional-social.
3. `he_suggests_coffee` (condition: `m.getLiking() == 'Like' || m.getLiking() == 'Close'`): He mentions the coffee shop, not as an invitation, just as a thing. You either pick up the thread or you don't. If you do: he says "yeah, okay" like it was already decided. +2 FEMININITY. `add_npc_liking(npc='m', delta=1)`. Sets `JAKE_COFFEE_PLANNED`. HOMOPHOBIC trait branch: desire registers before resistance does. Show the desire concretely first. OBJECTIFYING: you see the social mechanics from both sides simultaneously.

**Step 1: Dispatch 2 parallel scene-writer agents**

One per Jake scene. Each reads: `docs/writing-guide.md`, `packs/base/scenes/coffee_shop.toml` (existing Jake scene, for voice/tone continuity).

**Step 2: Wire into schedule.toml**

**Step 3: Validate-pack + full test suite**

```bash
cargo run --bin validate-pack && cargo test
```

Expected: `Validation passed. 32 scenes loaded.` All tests passing.

**Step 4: Final commit**

```bash
git add packs/base/scenes/coffee_shop_return.toml packs/base/scenes/jake_outside.toml packs/base/data/schedule.toml
git commit -m "feat: Jake follow-up scenes — coffee shop return, outside"
```

---

## Summary

After all 7 tasks complete:

| Pool | New scenes | Total scenes | FEMININITY gain |
|---|---|---|---|
| Work slot (new) | 7 (standup, lunch, late, corridor, friday, marcus_coffee, marcus_favor) | 7 | +8–14 per pass |
| Free_time | 5 (bookstore, park_walk, grocery, evening_home, neighborhood_bar) | 8 total | +5–8 per pass |
| Free_time / Jake | 2 (coffee_shop_return, jake_outside) | — | +2–4 progression |
| **Total new scenes** | **14** | **33 scenes loaded** | — |

Post-arc FEMININITY trajectory: 30 (arc end) → ~45–50 after playing through the new pool.

**Session log entry to add to HANDOFF.md after completion:**
> Sprint 3 "Robin's Playable Loop" complete. 14 new scenes: 7 work slot (new slot), 5 free_time, 2 Jake follow-ups. FEMININITY gains wired throughout. schedule.toml expanded. All validate-pack + tests passing.
