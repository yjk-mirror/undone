# Engine Readiness Matrix — 2026-03-07

Status meanings:

- `implemented/tested`
- `implemented/weakly tested`
- `partial`
- `missing`
- `undocumented`

| Area | Status | Notes |
|---|---|---|
| Pack load bootstrap | implemented/tested | `load_packs()` now fails if required structural IDs, names, or races are missing. |
| Character creation registry contract | implemented/tested | Startup validates all trait IDs referenced by custom mode, presets, and rough-content preferences. |
| Player init from `CharCreationConfig` | implemented/tested | `new_game()` seeds player state, flags, arcs, FEMININITY, and origin traits with unit coverage. |
| Preset FemCreation values shown accurately | implemented/tested | FemCreation now derives preset defaults from the selected preset instead of hardcoded `Eva` / `Ev`. |
| Structural ID contract | implemented/tested | Core structural IDs are validated at pack load; scene/stat/skill/category references are validated before runtime. |
| Scene TOML parsing + condition validation | implemented/tested | Scene conditions, effect IDs, goto targets, and action signatures fail during load. |
| Scheduler condition / trigger validation | implemented/tested | Schedule expressions now use the same semantic and ID validation path as scene conditions. |
| Scheduler scene references | implemented/tested | Schedule event scene IDs are validated during startup and `validate-pack`. |
| Opening / transformation scene references | implemented/tested | Manifest entry scenes are validated against the loaded scene set before gameplay starts. |
| Action visibility and stale-click protection | implemented/tested | `choose_action()` rechecks conditions before executing a selected action. |
| Scene transition guard | implemented/tested | Transition count is capped per command to prevent runaway goto cycles. |
| Runtime effect error surfacing | implemented/tested | Effect failures emit `ErrorOccurred`; UI appends them into story output. |
| Runtime unknown-scene visibility | implemented/tested | Unknown scene IDs generate visible error prose and finish the scene. |
| Runtime condition-eval visibility | implemented/weakly tested | Condition errors are logged and treated as false; visible UI diagnostics remain log-only. |
| Template render error visibility | implemented/weakly tested | Template failures surface as error prose / events, but UI-specific coverage is lighter than engine coverage. |
| Save format migration chain | implemented/tested | Save v1→v5 migration and ID-table validation have explicit test coverage. |
| Save/load ID stability invariants | implemented/tested | Saves embed interner strings; loads fail on mismatch or removed-pack shrinkage. |
| Fresh runtime after loading saves | implemented/weakly tested | Loaded saves rebuild `GameState` and `SceneEngine`; runtime-reset semantics exist and are unit tested, but end-to-end load/resume coverage is still lighter than core save tests. |
| Active NPC semantics (`m` / `f`) | implemented/weakly tested | Engine behavior is defined and tested, but it still supports only one active male and one active female per scene context. |
| NPC persistence (flags, roles, liking, relationship, contactability) | implemented/tested | Scene effects mutate persistent NPC world state with direct effect tests and integration coverage. |
| Multi-NPC scene selection UX | partial | Engine supports only active `m` / `f` bindings; richer UI selection remains pending. |
| Writing-tool subordinate DeepSeek integration | implemented/weakly tested | Repo-local helper exists and writing-agent docs are wired to it; coverage is command-level rather than mocked API tests. |
| Engine contract documentation | implemented | See `docs/engine-contract.md`. |
