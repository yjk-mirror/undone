# Restart Plan — 2026-03-19

## Situation

The engine works. 7 crates, 402 tests, pack validator clean, UI responsive, MCP
servers built. The game compiles and runs. Character creation, scene execution,
scheduling, save/load, NPC binding — all functional.

The problem: **the game has no prose worth playing.** All 54 scenes contain
agent-written text that the user rejected. The archived copies are identical to
the active copies — the "reset" removed nothing because no replacement prose was
written. The prose pipeline v2 exists but is blocked on voice samples.

This plan covers what to do now, given that reality.

---

## What exists

| Layer | Status | Notes |
|---|---|---|
| Engine (7 crates) | Done | 402 tests, all passing |
| UI | Done | Responsive, themed, dead-space hardened |
| Pack system | Done | TOML scenes, schedule, arc state machine |
| MCP servers (5) | Built | rhai, minijinja, screenshot, game-input, rust |
| Pipeline v2 tools | Built | scene-pipeline, prose-lint, prose-to-toml, spec-validate, pack-prompt |
| Voice samples | **Empty** | `docs/voice-samples/` has only `.gitkeep` |
| Scene prose | **Rejected** | 54 scenes with agent prose, all below quality bar |
| Adult scenes | **Zero** | Game can't prove its premise |
| Presets | Static Rust | Should be TOML pack data |
| FemCreation | Partial | Has framing, needs interactive discovery |
| Simulator cadence | Broken | Per-slot sim doesn't match real gameplay |

## Bottleneck analysis

The prose pipeline requires voice samples → the user writes those. Everything
downstream (DeepSeek drafts, lint, TOML conversion) is blocked on that input.

But the game also has structural gaps that don't require prose quality to fix.
The infrastructure work is done. What remains is:

1. **Content that needs the user's hand** — voice samples, creative direction
2. **Engine work that's independent of prose** — presets, simulator, FemCreation
3. **Scene structure work** — specs, schedules, flag chains (independent of prose text)

---

## The plan

### Track A: User-driven (prose calibration)

**Owner: User.** No agent work until samples exist.

1. User writes 3–5 voice sample scenes in `docs/voice-samples/`
2. Run pipeline on 1 test scene → user reviews → calibrate
3. If calibration passes, batch-produce scenes through pipeline
4. Writing-reviewer pass on output
5. Playtester pass on integrated scenes

**What "voice sample" means:** A complete scene's prose in the simple labeled
format (`INTRO:`, `ACTION: id`, etc.) — not TOML. Just the words. The pipeline's
`pack-prompt.mjs` includes these as few-shot examples for DeepSeek.

**Suggested first samples:**
- The bar intro from writing-samples.md Sample 0 (already written, just needs
  to be dropped into the voice-samples format)
- A workplace scene (Robin arriving, first day texture)
- An intimate/adult scene (to establish the explicit register)

### Track B: Presets as pack data (engineering, no creative judgment)

**Owner: Agent.** Independent of prose.

Right now Robin and Camila presets are hardcoded Rust structs in `undone-ui`.
They should be TOML files in `packs/base/data/presets/` loaded by `undone-packs`.

Tasks:
1. Design TOML schema for presets (name, before_name, traits, figure, route, etc.)
2. Write `robin.toml` and `camila.toml` from existing Rust struct data
3. Add preset loading to `undone-packs` PackRegistry
4. Replace hardcoded structs in UI with loaded preset data
5. Tests: preset loads, preset applies correctly to new game

**Why now:** Presets as data unblocks community/modding extensibility. It's a
clean engineering task with zero creative judgment needed.

### Track C: Simulator cadence fix (engineering)

**Owner: Agent.** Independent of prose.

The `validate-pack --simulate` tool picks once per slot per tick, which doesn't
match how real gameplay advances time (only `consumes_time = true` slots advance
the clock). The Codex attempt at this conflicted and was deleted.

Tasks:
1. Make simulator execute scenes (not just pick them) with a deterministic
   action policy (always pick first action, always continue)
2. Only advance time when a `consumes_time` slot's scene finishes
3. Test: Robin route reaches week 2 naturally without dev time-travel
4. Update `validate-pack --simulate` output to reflect real cadence

**Why now:** Without this, we can't validate that week-gated content is
actually reachable in real gameplay. It's the only way to trust the schedule.

### Track D: FemCreation interactive discovery (needs creative direction)

**Owner: User decides, agent implements.**

FemCreation currently shows a brief transformation bridge paragraph then jumps
to attribute selection. The creative-direction doc specifies that the player
"wakes up transformed" but the discovery beats — how she encounters the new
body — are undesigned.

**Blocked until user provides creative direction.** Questions:
- Is discovery interactive (player choices) or narrated (prose only)?
- What does she discover first? (Mirror? Clothes don't fit? Physical scale?)
- How much of this is in FemCreation vs. the first in-game scene?

---

## Execution order

```
Now:        Track B (presets as TOML) — clean, independent, unblocked
Parallel:   Track C (simulator cadence) — independent of prose
When ready:  Track A (voice samples → pipeline) — user-initiated
Later:      Track D (FemCreation) — needs creative direction first
```

Tracks B and C can run in parallel as separate worktrees. Track A starts
whenever the user is ready to write. Track D waits for user input.

---

## Success criteria

- [ ] Presets load from TOML, UI uses loaded data, tests pass
- [ ] Simulator executes scenes and advances time correctly
- [ ] Robin route reaches week 2 in simulation without dev time-travel
- [ ] Voice samples exist (user-written)
- [ ] Pipeline produces 1 scene that passes user review
- [ ] FemCreation has creative spec (user-provided)
