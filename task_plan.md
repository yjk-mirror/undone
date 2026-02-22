# Task Plan: Backend Completion — Scheduler + Save/Load

## Goal
Implement the Scheduler (weekly slot-based scene selection) and Save/Load system (JSON world persistence with ID validation), leaving the game backend complete for the upcoming UI session.

## Design Notes

### Scheduler
- Lives in `undone-scene/src/scheduler.rs` (all needed deps already present)
- TOML format: `packs/base/data/schedule.toml` with `[[slot]]` sections
- `PackContent` gets an optional `schedule_file: Option<String>` field
- `Scheduler::pick(slot, world, registry, rng) -> Option<String>` does weighted random selection
- Conditions use the existing expr evaluator (with an empty SceneCtx)

### Save/Load
- Uses existing `#[derive(Serialize, Deserialize)]` on `World` and all domain types
- Key challenge: interned IDs (`TraitId`, `SkillId` etc.) serialize as `u32` (Spur index), which is NOT stable across different pack load orderings
- Solution: save file includes `id_strings: Vec<String>` (all interned strings in Spur index order)
- On load: validate that saved id_strings match the current PackRegistry's Rodeo
- Add `PackRegistry::all_interned_strings()` and `PackRegistry::resolve_by_index()` helpers
- `undone-save/Cargo.toml` needs `undone-packs` added as a dep

## Phases
- [x] Phase 1: Research and design (done)
- [ ] Phase 2: Scheduler implementation
  - [ ] 2a. Add `schedule_file: Option<String>` to `PackContent`
  - [ ] 2b. Create `packs/base/data/schedule.toml`
  - [ ] 2c. Create `crates/undone-scene/src/scheduler.rs`
  - [ ] 2d. Update `undone-scene/src/lib.rs` exports
  - [ ] 2e. Add `PackRegistry::slot_names()` or similar if needed
  - [ ] 2f. Verify tests pass
- [ ] Phase 3: Save/Load implementation
  - [ ] 3a. Add `all_interned_strings()` to `PackRegistry` in `undone-packs/src/registry.rs`
  - [ ] 3b. Add `undone-packs` dep to `undone-save/Cargo.toml`
  - [ ] 3c. Implement `undone-save/src/lib.rs` with `SaveFile`, `save_game`, `load_game`
  - [ ] 3d. Verify tests pass
- [ ] Phase 4: Integration and cleanup
  - [ ] 4a. Run full test suite
  - [ ] 4b. Update HANDOFF.md
  - [ ] 4c. Commit

## Decisions Made
- Scheduler in `undone-scene` (not a new crate) — all deps already there
- Save format: `{ version, id_strings, world }` — validates ID stability on load
- No separate "SavedWorld" parallel types — use existing derives + id validation
- `w.hasStuff()` stub stays false — StuffId registry not needed yet

## Errors Encountered
(none yet)

## Status
**Currently in Phase 2** — about to create worktree and start implementation
