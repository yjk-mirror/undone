# Current Engine Contract

This document describes the engine behavior that is currently expected to be true in code as of 2026-03-10.

## 1. Player Init

Startup succeeds only if pack loading produces:

- required structural IDs used by runtime code
- non-empty male NPC names
- non-empty female NPC names
- non-empty races
- a valid opening scene and transformation scene, if declared
- a scheduler whose event scene IDs point at loaded scenes
- a character-creation registry that contains every trait ID referenced by custom mode, presets, and rough-content preferences
- a scheduler that references every preset starting flag declared by the built-in character-creation presets

`new_game(CharCreationConfig, registry, rng)` must:

- construct a `World` with player, NPC pools, and `GameData`
- seed `FEMININITY` from origin:
  - `CisMaleTransformed` -> `10`
  - `TransWomanTransformed` -> `70`
  - `CisFemaleTransformed` / `AlwaysFemale` -> `75`
- inject origin traits:
  - `TRANS_WOMAN` for `TransWomanTransformed`
  - `ALWAYS_FEMALE` for `CisFemaleTransformed`
  - `ALWAYS_FEMALE` + `NOT_TRANSFORMED` for `AlwaysFemale`
- copy `starting_flags` into `GameData.flags`
- copy `starting_arc_states` into `GameData.arc_states`
- spawn exactly the requested male/female NPC counts

The active player display name is derived from the `FEMININITY` skill:

- `0..=30` -> `name_masc`
- `31..=69` -> `name_androg`
- `70+` -> `name_fem`

## 2. Structural IDs

The current runtime treats these as structural IDs and validates them before play:

- skill: `FEMININITY`
- traits: `TRANS_WOMAN`, `ALWAYS_FEMALE`, `NOT_TRANSFORMED`, `NATURALLY_SMOOTH`, `SMOOTH_LEGS`

In addition, character creation validates every trait ID directly referenced by code:

- selectable custom traits
- preset trait payloads
- `BLOCK_ROUGH`
- `LIKES_ROUGH`

Character creation runtime contracts:

- built-in presets currently remain UI-defined, not pack-defined
- each built-in preset starting flag must be referenced by at least one scheduler condition or trigger
- `PartialCharState.starting_flags` carries preset routing flags explicitly; it is not an arc-id field

Scene content contracts:

- scene conditions are parsed and semantically validated at load time
- scene condition trait/skill/category IDs are validated at load time
- effect stat/skill/trait/arc references are validated at load time
- duplicate scene IDs fail scene load
- duplicate `actions[].id` and `npc_actions[].id` fail scene load
- goto targets are validated after all scenes load
- schedule conditions and triggers go through the same expression validation path during schedule load
- `validate-pack` warns when a scene has no persistent world mutation; scene-local flags and pure navigation do not satisfy that warning, while persistent player/NPC/world/time mutations do

## 3. Scene, Scheduler, and Action Semantics

Scene start contract:

1. Resolve active NPC bindings before intro render. Callers may pass explicit bindings, and the UI runtime controller pre-binds first-male / first-female fallbacks when the scene has not already selected active NPCs.
2. Set `SceneCtx.scene_id`.
3. Pick the first matching `intro_variant`, else use base intro.
4. Render intro prose.
5. Render intro thoughts whose conditions pass.
6. Push the scene frame.
7. Emit `NpcActivated` for any bound active NPCs.
8. Emit the visible action list.

Action contract:

- visible actions are those whose conditions evaluate true
- `choose_action()` rechecks the selected action condition before executing
- action prose renders before effects
- action thoughts render after action prose and before effects
- effect failures do not panic; they emit `ErrorOccurred`
- if `allow_npc_actions = true`, one eligible NPC action is chosen by weight and applied
- next branches are evaluated top to bottom; first passing branch wins
- if no next branch exists, the engine re-emits the current actions

Branch semantics:

- `goto = "scene_id"` starts a new scene
- `slot = "slot_name"` emits `SlotRequested(slot_name)`
- `finish = true` ends the current scene

Cycle protection:

- `MAX_TRANSITIONS_PER_COMMAND = 32`
- exceeding the limit emits visible engine-error prose, clears stack state, and finishes the scene

Scheduler contract:

- `pick(slot)` evaluates only the named slot
- `pick_next()` evaluates all slots
- trigger phase runs first, slot names sorted alphabetically
- weighted phase runs second across every eligible event from every slot
- `once_only` filtering is based on persistent `ONCE_<scene_id>` game flags
- the caller that actually starts the picked scene is responsible for setting the `ONCE_` flag

UI runtime controller contract:

- `RuntimeController` is the single owner of scene-start, choose-action, continue, jump, and resume semantics in `undone-ui`
- new-game launch, save resume, UI button handlers, dev IPC, and dev panel actions all delegate to the controller instead of reimplementing runtime flow
- `start_scene()` always clears transient scene UI state before entering a scene
- `continue_flow()` first checks whether the current runtime is awaiting continue; if not, it may launch the opening scene on first boot or ask the scheduler for the next eligible scene
- when `continue_flow()` starts a scheduler-picked `once_only` scene, it persists `ONCE_<scene_id>` before returning
- `jump_to_scene()` reuses the same scene-start path as normal gameplay
- `resume_from_current_world()` resets runtime-only scene state, does not replay `opening_scene`, and then uses the same continue path as normal runtime progression

## 4. Runtime Error Visibility

Startup-time failures:

- pack, scene, schedule, entry-scene, and char-creation-contract errors fail before gameplay and populate `init_error`

Scene-time failures:

- unknown scene IDs emit visible prose and `SceneFinished`
- template render failures emit `ErrorOccurred`
- effect failures emit `ErrorOccurred`
- scene condition-evaluation failures emit `ErrorOccurred`, are logged, and are treated as false
- scheduler condition / trigger evaluation failures are logged and treated as false

UI contract:

- `ErrorOccurred` is appended into story output as `[Scene error: ...]`
- story output is capped at 200 paragraphs
- new appended prose scrolls to bottom only after existing story already exists

## 5. Save / Load Invariants

Save format:

- current version is `5`
- save files store the full `World`
- save files also store `id_strings`, the pack interner contents in spur order
- save files written by the current runtime also store `pack_id_prefix_len`, the count of pack-loaded IDs before any runtime-only interning

Load must fail if:

- save version is newer than the loader understands
- saved interner strings differ from the current registry at any matching index
- `pack_id_prefix_len` is malformed or the current registry is shorter than the saved pack-loaded prefix
- an older save omits `pack_id_prefix_len` and has more interned IDs than the current registry

Load may succeed if:

- the current registry has additional IDs appended after the saved prefix
- the save references additional runtime-only interned IDs after the saved pack-loaded prefix and the current registry matches the saved ID table prefix; load replays only the missing saved tail back into the registry before deserializing the world

Migration chain:

- v1 -> v2
- v2 -> v3
- v3 -> v4
- v4 -> v5

Runtime reset invariant:

- loading a save rebuilds a fresh `GameState` and `SceneEngine`
- runtime scene stack / queued events are not part of the persisted world contract
- loaded games do not replay `opening_scene`
- loading a save into an existing in-memory game state must call the same runtime reset before resuming
- resume picks the next eligible scheduled scene from persisted world state, using the current scheduler/registry

## 6. NPC State Semantics

Scene expression/effect receivers:

- `m` means the active male NPC in `SceneCtx.active_male`
- `f` means the active female NPC in `SceneCtx.active_female`

The engine does not pick semantic NPCs on its own. The caller supplies active NPC bindings through:

- `SetActiveMale(MaleNpcKey)`
- `SetActiveFemale(FemaleNpcKey)`

Current UI helper behavior:

- `RuntimeController::start_scene()` uses `SceneEngine::start_scene_with_bindings(...)`
- when the caller has not explicitly chosen active NPCs, the controller binds the first male NPC and first female NPC in the world before intro render
- intro prose / intro thoughts / intro variants may rely on fallback `m` / `f` bindings when those NPCs exist in the world

Persistent NPC mutations include:

- liking / love / attraction
- relationship state
- behavior
- contactability
- relationship flags
- sexual activity memory
- named roles

`NpcActivated` snapshots surface:

- name
- age
- resolved personality string
- relationship
- PC liking of NPC
- PC attraction to NPC

Current limitation:

- scene runtime supports one active male NPC and one active female NPC at a time
- richer multi-NPC selection remains a UI/runtime-layer follow-up, not part of the current contract

## 7. Runtime Snapshot Contract

`RuntimeSnapshot` is the shared player-visible runtime contract used by tests, the dev panel inspector, dev IPC, and `game-input-mcp`.

It currently includes:

- `phase`
- `tab`
- `current_scene_id`
- `awaiting_continue`
- `init_error`
- `story_paragraphs`
- `visible_actions` with stable `id`, `label`, and `detail`
- `active_npc`
- player summary fields matching the sidebar
- world summary fields: `week`, `day`, `time_slot`, sorted `game_flags`, sorted `arc_states`

Snapshot invariants:

- story is exposed as paragraph strings using the same paragraph-splitting model as the visible story panel
- visible actions are only the actions currently available to the player
- snapshot building must not reimplement runtime flow; it only reflects `AppSignals` plus `GameState`

## 8. Dev IPC / MCP Runtime Tooling Contract

Dev IPC runtime commands:

- `get_runtime_state`
- `jump_to_scene`
- `choose_action`
- `continue_scene`
- `set_tab`

Successful runtime commands return the updated `RuntimeSnapshot` in `DevCommandResponse.data`.

Additional tooling invariants:

- `choose_action` must fail if the action id is not currently visible
- `continue_scene` must fail if runtime is not awaiting continue
- `set_tab` validates `game`, `saves`, `settings`, and `dev`, and rejects `dev` when dev mode is disabled
- `game-input-mcp` exposes typed wrappers for the runtime commands above so agents can inspect and drive a running game without screenshot parsing
