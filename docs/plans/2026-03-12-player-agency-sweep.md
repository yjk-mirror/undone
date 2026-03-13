# Player Agency Sweep — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Systematically find and fix every scene where the intro prose puts words in the player's mouth, decides the player's actions, or narrates the player's internal decisions before any action button is presented.

**Architecture:** Three-phase approach: (1) extend the automated prose audit to detect player-speech-in-intro violations, (2) run it against all 54 scenes to produce a ranked audit report, (3) rewrite the flagged intros scene by scene. The audit tooling catches the mechanical violations; the rewrites require creative judgment and will use the scene-writer agent. The writing-reviewer agent validates each rewrite, and the playtester verifies the result plays correctly.

**Tech Stack:** Rust (validate_pack prose audit), TOML (scene files), minijinja (template validation), scene-writer + writing-reviewer + playtester agents

---

## The Rule (from writing-guide.md §Player Agency)

**The intro describes the world. Actions are what the player decides to do.**

The intro puts the player in a situation. The world acts freely. But the intro **never** decides what the player does — not what she orders, not how she sits, not what she says, not what she thinks.

### Violation taxonomy

| Category | Severity | Example | Fix pattern |
|---|---|---|---|
| **Player speech in intro** | Critical | `"Thanks." You take the bag.` | Remove dialogue or move to action |
| **Player deliberate action in intro** | High | `You sit down` / `You grab the sweater` / `You nod back` | Reframe as world-state ("The seat is open") or move to action |
| **Player internal decision in intro** | High | `You file the pattern instead of reacting to it` | Cut or convert to thought fragment |
| **Player chooses/orders/responds in intro** | Critical | `"Yes. Robin. Software engineering."` | Move dialogue to action button |
| **Extended player autopilot in intro** | Critical | Multiple paragraphs of player acting (getting dressed, commuting, shaking hands) | Major restructure — split into setup + choice points |
| **Player involuntary body response** | Acceptable | `Your hands go cold` / `The stool is higher than expected` | **Not a violation** — the body acts, not the player |

### Known violations (confirmed by reading)

| Scene | Violation | Category |
|---|---|---|
| `workplace_arrival` | `"Thanks." You take the bag` | Player speech + action |
| `workplace_first_day` | `"Yes. Robin. Software engineering."` / entire morning narrated | Player speech + extended autopilot |
| `workplace_first_night` | `You set your carry-on down and do a circuit` / `You open a note` / `You text the shipping company` | Player acts repeatedly |
| `coffee_shop` | `You nod back` / `You smile` / `You look at the menu board` | Player responds/acts |
| `morning_routine` | `You get yourself together. Teeth, face...` / `You grab the yellow sweater` / outfit chosen for player | Extended autopilot |
| `neighborhood_bar` | Player implied to have walked to and sat at the bar | Player acts (implied) |

Likely present in most of the remaining ~48 scenes based on the 100% hit rate in the sample.

---

## Phase 1: Automated Detection (Tasks 1–4)

### Task 1: Add intro-section extraction to the prose audit

**Files:**
- Modify: `src/validate_pack.rs`

The current `audit_scene_text` function scans line-by-line but doesn't know whether it's in an intro section vs an action section. We need a structure-aware pass that:
1. Parses the TOML to extract `[intro].prose`, `[[intro_variants]].prose`, and `[[thoughts]].prose` as intro-zone text
2. Runs player-agency checks ONLY against intro-zone text (player speech in action prose is fine)

**Step 1: Write the failing test**

Add to `tests/prose_audit.rs`:
```rust
#[test]
fn prose_audit_flags_player_speech_in_intro() {
    let scene = r#"[scene]
id = "test::scene"
pack = "test"
description = "test"

[intro]
prose = """
The man hands you the bag.

"Thanks." You take it and keep moving.
""""#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(findings
        .iter()
        .any(|finding| finding.kind == "player_speech_in_intro"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone prose_audit_flags_player_speech_in_intro`
Expected: FAIL

**Step 3: Implement intro-section extraction**

In `src/validate_pack.rs`, add a new function `audit_intro_agency(file_path, scene_text) -> Vec<ProseFinding>` that:
1. Extracts intro prose using TOML parsing (the `[intro]` prose field + `[[intro_variants]]` prose fields)
2. Scans intro prose for quoted speech patterns that are player-attributed:
   - Lines starting with `"` followed by `You` (player speaks then acts)
   - `you say` / `you tell` / `you ask` patterns near quoted speech
   - **Exclude** NPC speech (lines where someone else is clearly speaking — context before the quote mentions a non-player subject)
3. Returns `ProseFinding` with kind `"player_speech_in_intro"`

Call this from the existing `audit_scene_text` function.

**Step 4: Run test to verify it passes**

Run: `cargo test -p undone prose_audit_flags_player_speech_in_intro`
Expected: PASS

**Step 5: Commit**

---

### Task 2: Add player-action-in-intro detection

**Files:**
- Modify: `src/validate_pack.rs`
- Modify: `tests/prose_audit.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn prose_audit_flags_player_deliberate_action_in_intro() {
    let scene = r#"[scene]
id = "test::scene"
pack = "test"
description = "test"

[intro]
prose = """
The coffee shop is warm.

You sit down at the counter and order a drink.
""""#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(findings
        .iter()
        .any(|finding| finding.kind == "player_action_in_intro"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone prose_audit_flags_player_deliberate_action_in_intro`
Expected: FAIL

**Step 3: Implement detection**

Detect sentences in intro prose where:
- `You` + deliberate verb: `sit`, `stand`, `grab`, `take`, `order`, `open`, `pick`, `choose`, `decide`, `walk`, `nod`, `smile`, `shake`, `text`, `type`, `add`
- Exclude involuntary/experiential verbs: `feel`, `notice`, `hear`, `see`, `smell`, `know`, `remember`, `catch yourself`, `realize`
- Exclude conditional body responses behind `{% if %}` FEMININITY guards (these are transformation-specific body experiences)

This will have false positives. That's fine — the audit produces a report for human review, not an auto-fix.

**Step 4: Run test to verify it passes**

Run: `cargo test -p undone prose_audit_flags_player_deliberate_action_in_intro`
Expected: PASS

**Step 5: Commit**

---

### Task 3: Run audit against all 54 scenes and produce the violation report

**Files:**
- Modify: `tests/prose_audit.rs`

**Step 1: Write a report-generating test**

```rust
#[test]
fn player_agency_audit_report() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().unwrap();
    let agency_findings: Vec<_> = report
        .prose_findings
        .iter()
        .filter(|f| {
            f.kind == "player_speech_in_intro" || f.kind == "player_action_in_intro"
        })
        .collect();

    // Print the report for human review
    for f in &agency_findings {
        eprintln!("[{}] {} (line {:?}): {}", f.kind, f.file_path, f.line, f.message);
    }

    // This test doesn't assert zero — it produces the ranked list
    eprintln!("\nTotal player-agency findings: {}", agency_findings.len());
}
```

**Step 2: Run the audit**

Run: `cargo test -p undone player_agency_audit_report -- --nocapture 2>&1 | tee docs/plans/player-agency-audit-results.txt`

**Step 3: Review the output and bucket scenes into tiers:**

- **Tier 1 (Critical):** Player speaks in intro, or extended autopilot (rewrite required)
- **Tier 2 (High):** Player takes deliberate actions in intro (restructure needed)
- **Tier 3 (Low):** Borderline cases — body responses, minor positioning (review case-by-case)

Save the tiered list into the audit results file.

**Step 4: Commit**

---

### Task 4: Update writing-guide.md and writing-reviewer agent

**Files:**
- Modify: `docs/writing-guide.md`
- Modify: `.claude/agents/writing-reviewer.md`

**Step 1: Add the violation taxonomy to the writing guide**

Add a subsection under "Player agency — the intro/action split" with the violation taxonomy table from this plan. Include concrete examples of each category with before/after.

**Step 2: Add detection rules to the writing-reviewer agent**

The writing-reviewer agent prompt should include these patterns as explicit check items:
- Does the intro contain any quoted player speech?
- Does the intro narrate the player taking deliberate actions?
- Does the intro narrate an extended autopilot sequence?
- Can you identify where the player's first CHOICE should appear?

**Step 3: Commit**

---

## Phase 2: Scene Rewrites (Tasks 5–9)

Each task covers a batch of scenes grouped by violation severity. Use the `scene-writer` agent for rewrites and `writing-reviewer` agent to validate.

### Rewrite principles

1. **Don't just delete — restructure.** Removing `"Thanks."` from the airport scene isn't enough if the narrative depends on it. The fix is to restructure: the man lifts the bag, hands it over — the player's first choice could be how to respond (or whether to respond).

2. **The intro ends with an invitation to act.** "The bartender is waiting." / "He's looking at you." / "The seat is open." The intro sets up the world and then hands control to the player.

3. **Extended autopilot → split into beats.** A scene like `workplace_first_day` that narrates an entire morning needs to become a scene with choice points: how to handle the lobby, how to respond to the manager, how to handle Dan.

4. **Involuntary body responses stay.** `Your hands go cold` / `The stool is higher than expected` — these are the body's experience, not the player's decision. They're core to the transformation premise.

5. **Player speech in actions is fine.** When the player clicks "Accept the drink," writing `"Sure, thanks"` in the action prose is correct — the player chose to engage.

### Task 5: Rewrite Tier 1 Critical scenes (player speech + extended autopilot)

**Scenes:** `workplace_arrival`, `workplace_first_day`, `workplace_first_night`, `morning_routine` (and others identified by Task 3 audit)

**For each scene:**

**Step 1:** Read the current scene, identify all intro violations, note which narrative beats the violations serve.

**Step 2:** Design the restructured intro:
- What situation does the world present?
- Where does the intro hand control to the player?
- What's the first choice point?

**Step 3:** Rewrite using `scene-writer` agent with explicit instructions:
- Preserve all trait branching
- Preserve all game effects and flags
- Move player speech/actions into action buttons or cut them
- The intro must end with an invitation to act, not an action already taken

**Step 4:** Validate with `minijinja` template validation

**Step 5:** Run `writing-reviewer` on the rewrite

**Step 6:** Commit the batch

---

### Task 6: Rewrite Tier 2 High scenes (deliberate player actions in intro)

**Scenes:** `coffee_shop`, `neighborhood_bar`, and others from Task 3 audit

Same process as Task 5 but these are lighter fixes — often just removing a nod, a smile, or repositioning the "you're already at the bar" framing.

**Step 1–6:** Same as Task 5.

---

### Task 7: Review Tier 3 borderline cases

**Scenes:** Identified by Task 3 audit

These need case-by-case judgment. Some will be fine (body responses), some will need minor tweaks.

**Step 1:** For each flagged line, decide: is this the player deciding, or the body experiencing?

**Step 2:** Fix only genuine violations. Leave body-response prose alone.

**Step 3:** Commit.

---

### Task 8: Re-run automated audit to verify zero Critical/High findings

**Step 1:** Run `cargo test -p undone player_agency_audit_report -- --nocapture`

**Step 2:** Verify no Tier 1 or Tier 2 findings remain.

**Step 3:** If findings remain, fix and repeat.

---

## Phase 3: Verification (Task 9)

### Task 9: Playtest the rewritten scenes

**Step 1:** Launch the game in dev mode using the playtester agent.

**Step 2:** Play through each rewritten scene using `jump_to_scene` + `get_runtime_state` + `choose_action`.

**Step 3:** Verify:
- Intro prose describes the world without player speech/actions
- First action button is the player's first decision
- Narrative flow still makes sense after restructuring
- Trait branching still works
- Game flags and effects still fire correctly

**Step 4:** Screenshot key scenes for visual verification.

**Step 5:** Commit final state.

---

## Estimated scope

- **Phase 1 (tooling):** ~4 tasks, mostly Rust code in the prose audit
- **Phase 2 (rewrites):** ~20-40 scenes depending on audit results. The 6 confirmed violations suggest high density. Each scene rewrite is a focused writing task.
- **Phase 3 (verification):** 1 playtest session covering all rewritten scenes

## Dependencies

- `docs/writing-guide.md` — the authoritative rule source
- `.claude/agents/scene-writer.md` — the rewrite agent
- `.claude/agents/writing-reviewer.md` — the validation agent
- `.claude/agents/playtester.md` — the verification agent
- `src/validate_pack.rs` — the automated audit infrastructure
- `tests/prose_audit.rs` — the audit test suite
