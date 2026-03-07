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
| Scene TOML parsing + condition validation | implemented/tested | Scene conditions, effect IDs, goto targets, duplicate scene IDs, and duplicate action / npc_action IDs fail during load. |
| `validate-pack` persistent-mutation warning policy | implemented/tested | Warning now keys off any persistent world mutation in player actions or NPC actions; scene-local flags and pure navigation no longer create false-positive "no lasting effects" warnings. |
| Scheduler condition / trigger validation | implemented/tested | Schedule expressions now use the same semantic and ID validation path as scene conditions. |
| Scheduler scene references | implemented/tested | Schedule event scene IDs are validated during startup and `validate-pack`. |
| Opening / transformation scene references | implemented/tested | Manifest entry scenes are validated against the loaded scene set before gameplay starts. |
| Action visibility and stale-click protection | implemented/tested | `choose_action()` rechecks conditions before executing a selected action. |
| Scene transition guard | implemented/tested | Transition count is capped per command to prevent runaway goto cycles. |
| Runtime effect error surfacing | implemented/tested | Effect failures emit `ErrorOccurred`; UI appends them into story output. |
| Runtime unknown-scene visibility | implemented/tested | Unknown scene IDs generate visible error prose and finish the scene. |
| Runtime condition-eval visibility | implemented/tested | Scene condition errors now emit `ErrorOccurred`, remain false for gating, and have direct engine coverage. |
| Template render error visibility | implemented/tested | Template failures emit `ErrorOccurred` and UI coverage verifies they reach story output as visible diagnostics. |
| Save format migration chain | implemented/tested | Save v1→v5 migration and ID-table validation have explicit test coverage. |
| Save/load ID stability invariants | implemented/tested | Saves embed interner strings; loads fail on mismatch or removed-pack shrinkage. |
| Fresh runtime after loading saves | implemented/tested | Resume tests now save real world state, reload through UI helpers, verify runtime reset, prevent opening-scene replay, and confirm scheduler picks from persisted state. |
| Active NPC semantics (`m` / `f`) | implemented/tested | Engine/UI tests now cover fallback binding for post-start action effects; contract explicitly limits fallback binding to post-start context and one active male + one active female. |
| NPC persistence (flags, roles, liking, relationship, contactability) | implemented/tested | Scene effects mutate persistent NPC world state with direct effect tests and integration coverage. |
| Multi-NPC scene selection UX | partial | Engine supports only active `m` / `f` bindings; richer UI selection remains pending. |
| Writing-tool subordinate DeepSeek integration | implemented/weakly tested | Repo-local helper exists and writing-agent docs are wired to it; coverage is command-level rather than mocked API tests. |
| Engine contract documentation | implemented | See `docs/engine-contract.md`. |
