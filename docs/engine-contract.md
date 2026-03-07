# Current Engine Contract

This document describes the engine behavior that is currently expected to be true in code as of 2026-03-07.

## 1. Player Init

Startup succeeds only if pack loading produces:

- required structural IDs used by runtime code
- non-empty male NPC names
- non-empty female NPC names
- non-empty races
- a valid opening scene and transformation scene, if declared
- a scheduler whose event scene IDs point at loaded scenes
- a character-creation registry that contains every trait ID referenced by custom mode, presets, and rough-content preferences

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

Scene content contracts:

- scene conditions are parsed and semantically validated at load time
- scene condition trait/skill/category IDs are validated at load time
- effect stat/skill/trait/arc references are validated at load time
- duplicate scene IDs fail scene load
- duplicate `actions[].id` and `npc_actions[].id` fail scene load
- goto targets are validated after all scenes load
- schedule conditions and triggers go through the same expression validation path during schedule load

## 3. Scene, Scheduler, and Action Semantics

Scene start contract:

1. Set `SceneCtx.scene_id`.
2. Pick the first matching `intro_variant`, else use base intro.
3. Render intro prose.
4. Render intro thoughts whose conditions pass.
5. Push the scene frame.
6. Emit the visible action list.

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

Load must fail if:

- save version is newer than the loader understands
- saved interner strings differ from the current registry at any matching index
- the save references more interned IDs than the current registry has

Load may succeed if:

- the current registry has additional IDs appended after the saved prefix

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

- `start_scene()` starts the scene, then binds the first male NPC and first female NPC in the world as fallback active receivers
- because the fallback binding happens after scene start, intro prose / intro thoughts / intro variants must not rely on fallback `m` / `f` bindings

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
