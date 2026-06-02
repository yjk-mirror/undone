# Marcus Terms-Fork Payoff — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.
> **Director's brief (scene specs + register):** `docs/plans/2026-06-02-marcus-terms-payoff-design.md`.
> Read it before dispatching — it carries the creative vision each scene-writer follows.

**Goal:** Make the `marcus_leverage` three-way fork (`TERMS_HERS` / `TERMS_HIS` /
`AFFAIR_COOLING`) change the back half of the affair: 4 new payoff scenes + re-gated, register-
varied recurring scenes, so the choice keeps mattering and "I ended it" is honored.

**Architecture:** Director (lead) + parallel `scene-writer` subagents. Subagents write one TOML
scene file each (prose only). Lead does all wiring (`schedule.toml`), surgical edits to the two
densely-branched recurring scenes, roadmap/story-map regen, validation, review-fix application,
playtest, and the merge. No subagent runs cargo/git or edits a file it doesn't own.

**Tech Stack:** TOML scenes with embedded Rhai (`effect`/`condition`) + minijinja (`prose`).
Validation: `cargo run --bin validate-pack`, `cargo test --workspace`,
`cargo run --bin story-map -- --check`. Authoring validation: `mcp__minijinja__jinja_validate_prose`,
`mcp__rhai__rhai_validate_effect`.

---

## File Structure

| File | Responsibility | Owner |
|---|---|---|
| `packs/base/scenes/marcus_terms_hers.toml` | NEW — HERS once-scene ("On Her Clock") | scene-writer A |
| `packs/base/scenes/marcus_terms_his.toml` | NEW — HIS once-scene ("Leverage") + line/sink sub-choice | scene-writer B |
| `packs/base/scenes/marcus_cooling.toml` | NEW — COOLING once-scene ("The Monday After") + hold/crack sub-choice | scene-writer C |
| `packs/base/scenes/marcus_cooling_relapse.toml` | NEW — RELAPSE once-scene ("Gravity") | scene-writer D |
| `packs/base/data/schedule.toml` | Bind 4 new once-scenes; re-gate recurrence | lead |
| `packs/base/scenes/marcus_repeat_office.toml` | HERS/HIS register layer + cooling gate | lead |
| `packs/base/scenes/marcus_pushes.toml` | Cooling-suppression gate | lead |
| `packs/base/roadmap.toml` | Claim new scenes under Marcus thread | lead |
| `docs/story-map.{md,json}` | Regenerated; 3 fork flags now consumed | lead (generated) |

**Resolved design decisions baked into this plan:**
- Once-scene schedule idiom (matches `marcus_apartment`): `weight = 0`, `trigger = '…'`,
  `once_only = true`, `npc_role = "ROLE_MARCUS"`, in the work `[slot]`.
- Cooling-suppression idiom (recurrence gate):
  `&& (!gd.hasGameFlag("MARCUS_AFFAIR_COOLING") || gd.hasGameFlag("MARCUS_RELAPSED"))`.
- **HIS "draw a line" converges on the cooling scene** by *also* setting `MARCUS_AFFAIR_COOLING`
  (plus `MARCUS_HIS_LINE_DRAWN` for flavor reads). One cooling entry point, no duplicate scene.
- Flags set by `marcus_leverage` already exist; this is their first consumer.

---

## Phase 0 — Worktree

**Step 1:** Create the isolated worktree (per ops:using-git-worktrees), branch from master HEAD:

```bash
git worktree add .worktrees/marcus-terms -b marcus-terms HEAD
```

**Step 2:** All subsequent file paths are under `.worktrees/marcus-terms/`. Scene-writers receive
absolute paths into this worktree. Verify: `git -C .worktrees/marcus-terms status` is clean.

---

## Phase 1 — Fan out 4 scene-writers (parallel)

Dispatch all four in ONE message (parallel). Each `scene-writer` subagent prompt MUST include:
(a) the path to the director's brief, (b) the brief's spec section for THAT scene quoted inline,
(c) read-refs: `docs/characters/marcus.md`, `docs/writing-guide.md`,
`.worktrees/marcus-terms/packs/base/scenes/marcus_leverage.toml` (voice) +
`marcus_repeat_office.toml` (structure/explicit register), (d) the exact output path, (e) the
exact `effect` Rhai string for each action (below), (f) the boundary: **write only your one TOML
file; do not run cargo/git; do not touch any other file.**

**Required `effect` strings — ALL VALIDATED against the live Rhai gate (`rhai_validate_effect`
→ empty diagnostics). Give verbatim. Activity id is `"vaginal"` (NOT `sex_vaginal`); het-sex
house convention = `TIMES_HAD_SEX` + `TOTAL_ORGASMS` + `TIMES_MADE_HIM_CUM` + `addSexualActivity("vaginal")`.**

- **A · `marcus_terms_hers`** — terminal action(s), HERS established + discharge:
  `gd.addStat("TIMES_HAD_SEX", 1); gd.addStat("TOTAL_ORGASMS", 1); gd.addStat("TIMES_MADE_HIM_CUM", 1); npc("m").addSexualActivity("vaginal"); npc("m").addLiking(1); w.skillIncrease("FEMININITY", 1); w.changeComposure(3); gd.setGameFlag("MARCUS_HERS_ESTABLISHED"); gd.setDesire(12);`
  (A second action may vary the act/tempo — keep the `MARCUS_HERS_ESTABLISHED` set + a discharge;
  adjust `addSexualActivity`/stats to the act written. At least 2 actions. A `RIDING`/`ORAL_SKILL`
  `w.skillIncrease(...)` may be added to match the act.)
- **B · `marcus_terms_his`** — two terminal actions:
  - *Sink in:* `gd.addStat("TIMES_HAD_SEX", 1); gd.addStat("TOTAL_ORGASMS", 1); gd.addStat("TIMES_MADE_HIM_CUM", 1); npc("m").addSexualActivity("vaginal"); npc("m").addLiking(1); w.skillIncrease("FEMININITY", 1); w.changeComposure(-6); w.changeStress(4); gd.setGameFlag("MARCUS_HIS_ESTABLISHED"); gd.setDesire(15);`
  - *Draw a late line:* `gd.setGameFlag("MARCUS_HIS_LINE_DRAWN"); gd.setGameFlag("MARCUS_AFFAIR_COOLING"); w.changeComposure(5); w.changeAnxiety(-2); npc("m").addLiking(-2);`
  - A low-composure reckless variant of *sink in* may gate its `condition` on `w.composure() < 30`.
- **C · `marcus_cooling`** — NO explicit content; two terminal actions:
  - *Hold the line:* `gd.setGameFlag("MARCUS_COOLING_HELD"); w.changeComposure(4); w.changeAnxiety(-3); w.changeStress(-2);`
  - *The door cracks:* `gd.setGameFlag("MARCUS_COOLING_RELAPSE"); gd.addDesire(15); w.changeComposure(-3); w.changeAnxiety(3);`
- **D · `marcus_cooling_relapse`** — terminal action(s):
  `gd.addStat("TIMES_HAD_SEX", 1); gd.addStat("TOTAL_ORGASMS", 1); gd.addStat("TIMES_MADE_HIM_CUM", 1); npc("m").addSexualActivity("vaginal"); npc("m").addLiking(1); w.skillIncrease("FEMININITY", 1); w.changeComposure(-5); w.changeStress(3); gd.setGameFlag("MARCUS_HIS_ESTABLISHED"); gd.setGameFlag("MARCUS_RELAPSED"); gd.setDesire(15);`

**Scene meta each file must have** (subagent fills `description`):
```toml
[scene]
id          = "base::<name>"
pack        = "base"
description = "<one line>"

[intro]
prose = """ … FEMININITY/desire-graded; second person present … """
```
Every action: `id`, `label`, `detail`, optional `condition`, `prose`, `effect`, and
`[[actions.next]] finish = true`.

**After each subagent returns:** lead validates that file with
`mcp__minijinja__jinja_validate_prose` (prose surface) and `mcp__rhai__rhai_validate_effect`
(each `effect`). Do NOT proceed to Phase 2 until all four files validate.

---

## Phase 2 — Lead integration: schedule wiring

**File:** `.worktrees/marcus-terms/packs/base/data/schedule.toml`

**Step 1 — Edit the two recurring gates** (append cooling-suppression idiom).

`marcus_repeat_office` condition (currently line ~404) becomes:
```toml
  condition     = 'gd.arcState("base::workplace_opening") == "settled" && gd.hasGameFlag("MARCUS_INTIMATE") && gd.isWeekday() && (!gd.hasGameFlag("MARCUS_AFFAIR_COOLING") || gd.hasGameFlag("MARCUS_RELAPSED"))'
```

`marcus_pushes` condition (currently line ~411) becomes:
```toml
  condition     = 'gd.arcState("base::workplace_opening") == "settled" && gd.hasGameFlag("MARCUS_INTIMATE") && !gd.hasGameFlag("MARCUS_ACT_ORAL") && (!gd.hasGameFlag("MARCUS_AFFAIR_COOLING") || gd.hasGameFlag("MARCUS_RELAPSED"))'
```

**Step 2 — Add 4 new once-scene events** in the same work `[slot]` (after the `marcus_leverage`
event, before `desire_ambush`):
```toml
  # ── Terms-fork payoff (consequence of marcus_leverage) ─────────────────
  [[slot.events]]
  scene     = "base::marcus_terms_hers"
  weight    = 0
  trigger   = 'gd.hasGameFlag("MARCUS_TERMS_HERS") && !gd.hasGameFlag("MARCUS_HERS_ESTABLISHED")'
  once_only = true
  npc_role  = "ROLE_MARCUS"

  [[slot.events]]
  scene     = "base::marcus_terms_his"
  weight    = 0
  trigger   = 'gd.hasGameFlag("MARCUS_TERMS_HIS") && !gd.hasGameFlag("MARCUS_HIS_ESTABLISHED") && !gd.hasGameFlag("MARCUS_HIS_LINE_DRAWN")'
  once_only = true
  npc_role  = "ROLE_MARCUS"

  [[slot.events]]
  scene     = "base::marcus_cooling"
  weight    = 0
  trigger   = 'gd.hasGameFlag("MARCUS_AFFAIR_COOLING") && !gd.hasGameFlag("MARCUS_COOLING_HELD") && !gd.hasGameFlag("MARCUS_COOLING_RELAPSE")'
  once_only = true
  npc_role  = "ROLE_MARCUS"

  [[slot.events]]
  scene     = "base::marcus_cooling_relapse"
  weight    = 0
  trigger   = 'gd.hasGameFlag("MARCUS_COOLING_RELAPSE") && !gd.hasGameFlag("MARCUS_RELAPSED")'
  once_only = true
  npc_role  = "ROLE_MARCUS"
```

**Step 3 — Commit.**
```bash
git -C .worktrees/marcus-terms add packs/base/data/schedule.toml
git -C .worktrees/marcus-terms commit -m "feat(schedule): bind Marcus terms-fork payoff scenes; cooling suppresses recurrence"
```

---

## Phase 3 — Lead integration: existing-scene register edits

**File:** `marcus_repeat_office.toml` — add a HERS/HIS register block at the TOP of `[intro].prose`
(layered above the existing location/desire branches; do not modify those):
```jinja
{% if gd.hasGameFlag("MARCUS_HERS_ESTABLISHED") %}
<one short paragraph: SHE sets it now — she's the one who closed the door / named the place / set
the clock; his competence serves her tempo. Charged, economical, her control legible in what she
does. Second person present.>
{% elif gd.hasGameFlag("MARCUS_HIS_ESTABLISHED") %}
<one short paragraph: HIS terms — he assumes the access, the asymmetry sits in the framing, the
cost (the floor, being seen) is closer. Not crude; that's what makes it press.>
{% endif %}
```
Keep it to register-setting prose; the body of the scene (the actual encounter) stays shared.
Validate with `mcp__minijinja__jinja_validate_prose`. **Lead writes this prose** (surgical, must
fit existing voice) — do not delegate.

**File:** `marcus_pushes.toml` — no prose edit required (suppression is in schedule). Optional: a
one-line HERS/HIS flavor only if it slots cleanly; skip if not.

**Commit.**
```bash
git -C .worktrees/marcus-terms add packs/base/scenes/marcus_repeat_office.toml
git -C .worktrees/marcus-terms commit -m "feat(content): marcus_repeat_office reflects HERS/HIS power register"
```

---

## Phase 4 — Roadmap + story-map regen

**Step 1:** Add the 4 new scenes to the Marcus thread in
`.worktrees/marcus-terms/packs/base/roadmap.toml` (follow existing thread entry format —
read the Marcus thread block first; add scene ids to its explicit list if it uses one).

**Step 2:** Regenerate (build runs from the worktree; CARGO_TARGET_DIR shared):
```bash
cd .worktrees/marcus-terms && cargo run --bin story-map && cd ../..
```

**Step 3:** Confirm `MARCUS_TERMS_HERS`, `MARCUS_TERMS_HIS`, `MARCUS_AFFAIR_COOLING` are NO
LONGER in the "Write Next / dangling" list:
```bash
grep -E "MARCUS_TERMS_HERS|MARCUS_TERMS_HIS|MARCUS_AFFAIR_COOLING" .worktrees/marcus-terms/docs/story-map.md | grep -i dangling
```
Expected: no output (the 3 flags are consumed).

**Step 4 — Commit.**
```bash
git -C .worktrees/marcus-terms add packs/base/roadmap.toml docs/story-map.md docs/story-map.json
git -C .worktrees/marcus-terms commit -m "docs(story-map): claim terms-fork scenes; 3 fork flags now consumed"
```

---

## Phase 5 — Validation gate (lead, single pass)

Run from the worktree. ALL must pass before review/merge.

**Step 1:** `cd .worktrees/marcus-terms && cargo run --bin validate-pack`
Expected: "All checks passed." Scene count = previous + 4 (78). Prose gate active, no new errors.

**Step 2:** `cargo test --workspace`
Expected: green. Watch the render-regression test
(`all_scene_intros_render_without_missing_methods`) — it catches any `effect`/prose method that
exists in Rhai but not the minijinja ctx. If a new scene's intro references a method the template
ctx lacks, fix the scene prose (intro prose must only use read methods).

**Step 3:** `cargo run --bin story-map -- --check`
Expected: exit 0 (committed map not stale).

**Step 4:** `cargo fmt --all` (no-op for TOML, but keeps any touched .rs clean) — none expected.

Fix-and-rerun until clean. Commit any fixes with descriptive messages.

---

## Phase 6 — Writing review (parallel) + apply Critical fixes

**Step 1:** Dispatch 4 `writing-reviewer` subagents (parallel), one per new scene. Each gets the
scene file + `docs/writing-guide.md` + `docs/characters/marcus.md`. They are read-only; they
return Critical/Important/Minor findings. Watch specifically for: narrated interiority,
body-as-witness, "come" vs "cum" (climax sense), over-naming, staccato-closer tic, and HERS-branch
power being *narrated* rather than *shown*.

**Step 2:** Lead applies all **Critical** findings (and clear Important ones) directly. Re-validate
edited files (`jinja_validate_prose`). Commit:
```bash
git -C .worktrees/marcus-terms add packs/base/scenes/
git -C .worktrees/marcus-terms commit -m "fix(content): apply writing-review Critical findings (terms-fork scenes)"
```

---

## Phase 7 — Playtest (acceptance)

**Acceptance criteria (the playtester verifies each in-game via dev-IPC):**
- From `marcus_leverage`, take **push_back** → `marcus_terms_hers` fires; SHE controls; afterward
  `marcus_repeat_office` shows the HERS register.
- Take **say_nothing** → `marcus_terms_his` fires; both sub-choices reachable. *Sink in* →
  `MARCUS_HIS_ESTABLISHED`, recurrence continues in HIS register. *Draw a line* →
  `marcus_cooling` fires next (converged via `AFFAIR_COOLING`).
- Take **end_it** → `marcus_cooling` fires; recurrence (`marcus_repeat_office`/`marcus_pushes`)
  **no longer offered**. *Hold* → stays stopped. *Crack* → `marcus_cooling_relapse` fires →
  `MARCUS_RELAPSED` set → recurrence offered again (HIS register).
- All new scenes render with zero prose errors at low-FEM and high-FEM.

**Step 1:** Launch playtester with dev-IPC. Use `set_game_flag`/`jump_to_scene` to seed each fork
state directly (set `MARCUS_INTIMATE`, `MARCUS_REPEAT_USED_CONF`, then the relevant `TERMS_*`
flag) rather than grinding the whole affair. Verify each criterion; screenshot the three registers.

**Step 2:** Playtester returns a player-experience report. Treat as feedback. Lead fixes any
flow/prose breakage found, re-validates, commits.

---

## Phase 8 — Finish the branch (merge)

Per ops:finishing-a-development-branch (project override: always merge, never discard).

**Step 1:** Final gate re-run from worktree: `cargo run --bin validate-pack` + `cargo test --workspace` + `story-map --check` — all green.

**Step 2:** Update `HANDOFF.md` Current State + Session Log (new session entry: what shipped, the
3 flags now consumed, the cooling continuity fix, verification results).

**Step 3:** Merge to master and clean up:
```bash
git -C .worktrees/marcus-terms add HANDOFF.md && git -C .worktrees/marcus-terms commit -m "docs: HANDOFF — Marcus terms-fork payoff shipped"
git checkout master
git merge marcus-terms
git worktree remove .worktrees/marcus-terms
git branch -d marcus-terms
```

**Step 4:** Verify `git status` clean on master; `git log --oneline -8` shows the slice.

---

## Self-Review (against the design brief)

- **Spec coverage:** 4 new scenes (Tasks/Phase 1) ✓; HERS/HIS register in recurring (Phase 3) ✓;
  cooling suppression + relapse re-open (Phase 2 gates) ✓; roadmap/story-map (Phase 4) ✓;
  review (Phase 6) ✓; playtest of all three doors (Phase 7) ✓.
- **Placeholder scan:** all `effect` strings concrete; schedule TOML literal; only prose bodies are
  delegated (by design — that's the scene-writers' job, fully specified by register + flags).
- **Flag/name consistency:** `MARCUS_HERS_ESTABLISHED`, `MARCUS_HIS_ESTABLISHED`,
  `MARCUS_HIS_LINE_DRAWN`, `MARCUS_COOLING_HELD`, `MARCUS_COOLING_RELAPSE`, `MARCUS_RELAPSED` used
  identically in effects, schedule triggers, recurrence gates, and story-map check. Cooling
  suppression idiom identical in both recurring gates. HIS-line convergence sets `AFFAIR_COOLING`
  (matches the `marcus_cooling` trigger). Consistent.

---

## Execution Handoff

```
Use `ops:executing-plans` to implement the plan at `docs/plans/2026-06-02-marcus-terms-payoff.md`
```
