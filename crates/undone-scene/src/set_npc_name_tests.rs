/// Independent acceptance tests for the `set_npc_name` feature.
///
/// Written by an independent test author. These tests exercise the acceptance
/// criteria from the spec without reference to the implementer's reasoning.
///
/// Acceptance criteria being tested:
///   1. TOML `type = "set_npc_name"` deserializes to `EffectDef::SetNpcName`.
///   2. After the effect runs, the NPC's display name is visible in:
///      a. `NpcCore::effective_name()`
///      b. `NpcActivatedData::from_npc()` (sidebar path)
///      c. `m.getName()` / `f.getName()` in prose templates
///      d. `role.getName("ROLE_X")` in prose templates
///   3. The original spawn name (`core.name`) is NOT destroyed.
///   4. `set_npc_name` and `set_npc_role` are independent.
///   5. Save/load round-trip preserves `display_name`; old saves (missing field) load cleanly.
///   6. Invalid NPC references return errors, not panics.
///   7. The three canonical pack scenes contain `set_npc_name` effects.

#[cfg(test)]
mod set_npc_name_acceptance_tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use lasso::Key;
    use undone_domain::{
        Age, AlcoholLevel, ArousalLevel, AttractionLevel, Behaviour, LikingLevel, LoveLevel,
        MaleClothing, MaleFigure, MaleNpc, NpcCore, PersonalityId, RelationshipStatus,
    };
    use undone_expr::SceneCtx;
    use undone_packs::PackRegistry;
    use undone_world::test_helpers::make_test_world;

    use crate::effects::{apply_effect, EffectError};
    use crate::engine::NpcActivatedData;
    use crate::template_ctx::render_prose;
    use crate::types::{EffectDef, SceneToml};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap() // exits crates/undone-scene/
            .parent()
            .unwrap() // exits crates/
            .join("packs")
    }

    fn spawn_name() -> &'static str {
        "Brian" // deliberately NOT Jake/Marcus/Theo — distinguishable from story name
    }

    fn make_male_npc_with_spawn_name(name: &str) -> MaleNpc {
        MaleNpc {
            core: NpcCore {
                name: name.to_string(),
                display_name: None, // starts with no override
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "brown".into(),
                hair_colour: "black".into(),
                personality: PersonalityId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
                traits: HashSet::new(),
                relationship: RelationshipStatus::Stranger,
                pc_liking: LikingLevel::Neutral,
                npc_liking: LikingLevel::Neutral,
                pc_love: LoveLevel::None,
                npc_love: LoveLevel::None,
                pc_attraction: AttractionLevel::Unattracted,
                npc_attraction: AttractionLevel::Unattracted,
                behaviour: Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: ArousalLevel::Comfort,
                alcohol: AlcoholLevel::Sober,
                roles: HashSet::new(),
            },
            figure: MaleFigure::Average,
            clothing: MaleClothing::default(),
            had_orgasm: false,
            has_baby_with_pc: false,
        }
    }

    // -----------------------------------------------------------------------
    // CRITERION 1 — TOML deserialization
    // -----------------------------------------------------------------------

    /// BREAKS IF: `type = "set_npc_name"` in TOML fails to deserialize (typo in variant
    /// name, missing field, wrong serde tag), causing pack loading to crash or silently
    /// skip the effect so NPCs are never renamed.
    #[test]
    fn toml_set_npc_name_deserializes_correctly() {
        let toml = r#"
[scene]
id = "test::rename"
pack = "test"
description = "Name override test."

[intro]
prose = "It begins."

[[actions]]
id = "meet"
label = "Meet"

  [[actions.effects]]
  type = "set_npc_name"
  npc  = "m"
  name = "Jake"
"#;
        let raw: SceneToml = toml::from_str(toml)
            .expect("TOML with type = \"set_npc_name\" should deserialize without error");

        let meet = raw.actions.iter().find(|a| a.id == "meet").unwrap();
        assert_eq!(
            meet.effects.len(),
            1,
            "action should have exactly one effect"
        );

        match &meet.effects[0] {
            EffectDef::SetNpcName { npc, name } => {
                assert_eq!(npc, "m", "npc field should be 'm'");
                assert_eq!(name, "Jake", "name field should be 'Jake'");
            }
            other => panic!("expected SetNpcName, got {:?}", other),
        }
    }

    /// BREAKS IF: the `set_npc_name` TOML variant requires fields in a specific order or
    /// has a case-sensitive mismatch, causing deserialization to fail on real pack content.
    #[test]
    fn toml_set_npc_name_with_role_ref_deserializes() {
        let toml = r#"
[scene]
id = "test::rename_role"
pack = "test"
description = "Name override via role."

[intro]
prose = "It begins."

[[actions]]
id = "meet"
label = "Meet"

  [[actions.effects]]
  type = "set_npc_name"
  npc  = "ROLE_THEO"
  name = "Theo"
"#;
        let raw: SceneToml = toml::from_str(toml).unwrap();
        let meet = raw.actions.iter().find(|a| a.id == "meet").unwrap();
        match &meet.effects[0] {
            EffectDef::SetNpcName { npc, name } => {
                assert_eq!(npc, "ROLE_THEO");
                assert_eq!(name, "Theo");
            }
            other => panic!("expected SetNpcName, got {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // CRITERION 2a — effective_name() returns override
    // -----------------------------------------------------------------------

    /// BREAKS IF: `effective_name()` ignores `display_name` and always returns
    /// `core.name`, so the random spawn name shows everywhere instead of "Jake".
    #[test]
    fn effective_name_returns_display_name_when_set() {
        let mut npc = make_male_npc_with_spawn_name(spawn_name());
        // Before override: should return spawn name
        assert_eq!(npc.core.effective_name(), spawn_name());

        npc.core.display_name = Some("Jake".to_string());
        // After override: should return story name, NOT spawn name
        assert_eq!(
            npc.core.effective_name(),
            "Jake",
            "effective_name() must return display_name when set"
        );
        assert_ne!(
            npc.core.effective_name(),
            spawn_name(),
            "effective_name() must NOT return spawn name when display_name is set"
        );
    }

    /// BREAKS IF: `effective_name()` returns `display_name` even when it's None
    /// (e.g. unwraps blindly), causing a panic instead of falling back to spawn name.
    #[test]
    fn effective_name_falls_back_to_spawn_name_when_no_override() {
        let npc = make_male_npc_with_spawn_name("RandomNpcName");
        assert!(npc.core.display_name.is_none());
        assert_eq!(
            npc.core.effective_name(),
            "RandomNpcName",
            "effective_name() must fall back to core.name when display_name is None"
        );
    }

    // -----------------------------------------------------------------------
    // CRITERION 2b — NpcActivatedData (sidebar) uses effective_name()
    // -----------------------------------------------------------------------

    /// BREAKS IF: `NpcActivatedData::from_npc` reads `core.name` directly instead of
    /// `effective_name()`, causing the People Here sidebar to show the random spawn
    /// name "Brian" instead of the story name "Jake" after the first-meeting scene.
    #[test]
    fn npc_activated_data_shows_display_name_not_spawn_name() {
        let mut registry = PackRegistry::new();
        let personality = registry.intern_personality("ROMANTIC");
        let mut npc = make_male_npc_with_spawn_name(spawn_name());
        npc.core.personality = personality;
        npc.core.display_name = Some("Jake".to_string());

        let data = NpcActivatedData::from_npc(&npc.core, &registry);

        assert_eq!(
            data.name, "Jake",
            "sidebar must show the story name 'Jake', not the spawn name"
        );
        assert_ne!(
            data.name,
            spawn_name(),
            "sidebar must NOT show the random spawn name after set_npc_name runs"
        );
    }

    /// BREAKS IF: NpcActivatedData ignores display_name entirely and always uses name.
    /// Counterpart: sidebar shows spawn name before the effect runs (correct baseline).
    #[test]
    fn npc_activated_data_shows_spawn_name_before_override() {
        let mut registry = PackRegistry::new();
        let personality = registry.intern_personality("CALM");
        let mut npc = make_male_npc_with_spawn_name(spawn_name());
        npc.core.personality = personality;
        // No display_name set — still a stranger

        let data = NpcActivatedData::from_npc(&npc.core, &registry);
        assert_eq!(
            data.name,
            spawn_name(),
            "before set_npc_name runs, sidebar should show the spawn name"
        );
    }

    // -----------------------------------------------------------------------
    // CRITERION 2c — m.getName() / f.getName() in prose templates
    // -----------------------------------------------------------------------

    /// BREAKS IF: the `NpcCtx` built inside `render_prose` for the active male copies
    /// `core.name` rather than `effective_name()`, so prose continues to address the
    /// character by his random spawn name after the rename effect has run.
    #[test]
    fn m_get_name_returns_display_name_in_prose_template() {
        let registry = PackRegistry::new();
        let mut world = make_test_world();
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        world.male_npcs[npc_key].core.display_name = Some("Jake".to_string());

        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(npc_key);

        let template = r#"{{ m.getName() }}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();

        assert_eq!(
            result.trim(),
            "Jake",
            "m.getName() in prose must return 'Jake' after set_npc_name; got: '{}'",
            result.trim()
        );
        assert_ne!(
            result.trim(),
            spawn_name(),
            "m.getName() must NOT return the spawn name after set_npc_name has run"
        );
    }

    /// BREAKS IF: prose renders the spawn name before set_npc_name runs (incorrect baseline),
    /// or if the active_male None-case panics instead of producing UNDEFINED.
    #[test]
    fn m_get_name_returns_spawn_name_before_override() {
        let registry = PackRegistry::new();
        let mut world = make_test_world();
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name("OriginalName"));
        // No display_name set

        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(npc_key);

        let template = r#"{{ m.getName() }}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();

        assert_eq!(
            result.trim(),
            "OriginalName",
            "before set_npc_name, m.getName() should return the spawn name; got: '{}'",
            result.trim()
        );
    }

    // -----------------------------------------------------------------------
    // CRITERION 2d — role.getName("ROLE_X") in prose templates
    // -----------------------------------------------------------------------

    /// BREAKS IF: role-bound NpcCtx is built with `core.name` instead of `effective_name()`,
    /// so `role.getName("ROLE_THEO")` in campus_library prose returns "Kevin" not "Theo".
    #[test]
    fn role_get_name_returns_display_name_for_role_bound_npc() {
        use undone_expr::SceneNpcRef;

        let registry = PackRegistry::new();
        let mut world = make_test_world();
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        world.male_npcs[npc_key].core.display_name = Some("Theo".to_string());

        let mut ctx = SceneCtx::new();
        ctx.bind_role("ROLE_THEO", SceneNpcRef::Male(npc_key));

        let template = r#"{{ role.getName("ROLE_THEO") }}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();

        assert_eq!(
            result.trim(),
            "Theo",
            "role.getName(\"ROLE_THEO\") must return 'Theo' after set_npc_name; got: '{}'",
            result.trim()
        );
        assert_ne!(
            result.trim(),
            spawn_name(),
            "role.getName() must NOT return the spawn name after set_npc_name"
        );
    }

    // -----------------------------------------------------------------------
    // CRITERION 3 — spawn name preserved
    // -----------------------------------------------------------------------

    /// BREAKS IF: `apply_effect` for SetNpcName overwrites `core.name` instead of
    /// `display_name`, destroying the spawn name permanently.
    #[test]
    fn set_npc_name_effect_preserves_spawn_name() {
        let mut world = make_test_world();
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(npc_key);
        let registry = PackRegistry::new();

        let effect = EffectDef::SetNpcName {
            npc: "m".into(),
            name: "Jake".into(),
        };
        apply_effect(&effect, &mut world, &mut ctx, &registry).unwrap();

        let npc = &world.male_npcs[npc_key];
        assert_eq!(
            npc.core.name,
            spawn_name(),
            "core.name (spawn name) must be unchanged after set_npc_name"
        );
        assert_eq!(
            npc.core.display_name.as_deref(),
            Some("Jake"),
            "display_name must be set to the story name"
        );
    }

    // -----------------------------------------------------------------------
    // CRITERION 4 — independence from set_npc_role
    // -----------------------------------------------------------------------

    /// BREAKS IF: `set_npc_role` implicitly sets `display_name`, violating independence.
    /// The role assignment should not trigger any rename side-effect.
    #[test]
    fn set_npc_role_does_not_set_display_name() {
        let mut world = make_test_world();
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(npc_key);
        let registry = PackRegistry::new();

        let role_effect = EffectDef::SetNpcRole {
            npc: "m".into(),
            role: "ROLE_JAKE".into(),
        };
        apply_effect(&role_effect, &mut world, &mut ctx, &registry).unwrap();

        let npc = &world.male_npcs[npc_key];
        assert!(
            npc.core.roles.contains("ROLE_JAKE"),
            "role should be set after set_npc_role"
        );
        assert_eq!(
            npc.core.display_name, None,
            "set_npc_role must NOT set display_name — it stays None until set_npc_name runs"
        );
        assert_eq!(
            npc.core.effective_name(),
            spawn_name(),
            "effective_name should still be spawn name after only set_npc_role"
        );
    }

    /// BREAKS IF: `set_npc_name` implicitly sets a role on the NPC, violating independence.
    #[test]
    fn set_npc_name_does_not_set_npc_role() {
        let mut world = make_test_world();
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(npc_key);
        let registry = PackRegistry::new();

        let name_effect = EffectDef::SetNpcName {
            npc: "m".into(),
            name: "Marcus".into(),
        };
        apply_effect(&name_effect, &mut world, &mut ctx, &registry).unwrap();

        let npc = &world.male_npcs[npc_key];
        assert!(
            npc.core.roles.is_empty(),
            "set_npc_name must NOT assign any role — roles remain empty"
        );
        assert_eq!(npc.core.display_name.as_deref(), Some("Marcus"));
    }

    /// BREAKS IF: combining both effects in sequence causes them to interfere
    /// (one clobbers the other's field).
    #[test]
    fn set_npc_role_and_set_npc_name_combined_are_independent() {
        let mut world = make_test_world();
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(npc_key);
        let registry = PackRegistry::new();

        apply_effect(
            &EffectDef::SetNpcRole {
                npc: "m".into(),
                role: "ROLE_JAKE".into(),
            },
            &mut world,
            &mut ctx,
            &registry,
        )
        .unwrap();
        apply_effect(
            &EffectDef::SetNpcName {
                npc: "m".into(),
                name: "Jake".into(),
            },
            &mut world,
            &mut ctx,
            &registry,
        )
        .unwrap();

        let npc = &world.male_npcs[npc_key];
        assert!(npc.core.roles.contains("ROLE_JAKE"), "role must be set");
        assert_eq!(
            npc.core.display_name.as_deref(),
            Some("Jake"),
            "display_name must be set"
        );
        assert_eq!(npc.core.name, spawn_name(), "spawn name must be preserved");
    }

    // -----------------------------------------------------------------------
    // CRITERION 5 — save/load round-trip
    // -----------------------------------------------------------------------

    /// BREAKS IF: `display_name` is missing `#[serde(default)]`, causing old saves
    /// (written before this feature) to fail to load with a JSON deserialization error.
    ///
    /// Strategy: serialize a real NpcCore (which has display_name = None), strip the
    /// display_name key from the JSON to simulate a pre-feature save, then deserialize.
    /// If deserialization fails, `#[serde(default)]` is missing.
    #[test]
    fn display_name_field_deserializes_as_none_when_absent_in_json() {
        // Start from a real NpcCore to avoid constructing invalid lasso Spur values.
        let npc = make_male_npc_with_spawn_name("Brian");
        let original_json = serde_json::to_string(&npc.core).unwrap();

        // Parse and remove display_name to simulate a pre-feature save.
        let mut val: serde_json::Value = serde_json::from_str(&original_json).unwrap();
        val.as_object_mut()
            .expect("NpcCore JSON must be an object")
            .remove("display_name");

        assert!(
            !val.as_object().unwrap().contains_key("display_name"),
            "test setup: display_name must be absent from the patched JSON"
        );

        let core: undone_domain::NpcCore = serde_json::from_value(val)
            .expect("NpcCore without display_name field must deserialize (old save compatibility)");

        assert!(
            core.display_name.is_none(),
            "display_name should default to None when absent in JSON"
        );
        assert_eq!(
            core.name, "Brian",
            "spawn name must be deserialized correctly"
        );
        assert_eq!(core.effective_name(), "Brian");
    }

    /// BREAKS IF: `display_name` is not serialized, causing it to be lost on save/reload.
    /// The round-trip must preserve the story name override.
    #[test]
    fn display_name_round_trips_through_json_serialization() {
        let mut npc = make_male_npc_with_spawn_name(spawn_name());
        npc.core.display_name = Some("Jake".to_string());

        let json =
            serde_json::to_string(&npc.core).expect("NpcCore with display_name must serialize");

        let restored: undone_domain::NpcCore =
            serde_json::from_str(&json).expect("NpcCore with display_name must deserialize");

        assert_eq!(
            restored.display_name.as_deref(),
            Some("Jake"),
            "display_name must survive a JSON round-trip"
        );
        assert_eq!(
            restored.name,
            spawn_name(),
            "spawn name must also survive the round-trip"
        );
        assert_eq!(restored.effective_name(), "Jake");
    }

    /// BREAKS IF: the full save/load pipeline (including the undone-save crate) drops
    /// display_name somewhere in the serialization chain.
    #[test]
    fn display_name_survives_full_save_load_cycle() {
        use undone_packs::load_packs;
        use undone_save::{load_game, save_game};

        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let mut world = make_test_world();

        // Give the male NPC a display name override
        let npc_key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        world.male_npcs[npc_key].core.display_name = Some("Jake".to_string());

        let dir = std::env::temp_dir().join("undone_set_npc_name_acceptance");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("display_name_round_trip.json");

        save_game(&world, &registry, &path).expect("save must succeed");
        let loaded = load_game(&path, &mut registry).expect("load must succeed");

        // Find the NPC we inserted (by position — there's only one male NPC)
        let loaded_npc = loaded
            .male_npcs
            .values()
            .next()
            .expect("loaded world must contain the male NPC");

        assert_eq!(
            loaded_npc.core.display_name.as_deref(),
            Some("Jake"),
            "display_name must survive a full save/load cycle"
        );
        assert_eq!(
            loaded_npc.core.name,
            spawn_name(),
            "spawn name must also survive a full save/load cycle"
        );
        assert_eq!(loaded_npc.core.effective_name(), "Jake");
    }

    // -----------------------------------------------------------------------
    // CRITERION 6 — error cases (no panics, proper error variants)
    // -----------------------------------------------------------------------

    /// BREAKS IF: `apply_effect` panics (e.g. unwraps on None) when no active male
    /// NPC is set, instead of returning `EffectError::NoActiveMale`.
    #[test]
    fn set_npc_name_returns_error_when_no_active_male() {
        let mut world = make_test_world();
        let mut ctx = SceneCtx::new();
        // active_male intentionally left None
        let registry = PackRegistry::new();

        let effect = EffectDef::SetNpcName {
            npc: "m".into(),
            name: "Jake".into(),
        };
        let result = apply_effect(&effect, &mut world, &mut ctx, &registry);

        assert!(
            result.is_err(),
            "must return Err when no active male NPC is set"
        );
        assert!(
            matches!(result, Err(EffectError::NoActiveMale)),
            "error must be NoActiveMale, got: {:?}",
            result
        );
    }

    /// BREAKS IF: `apply_effect` panics when no active female NPC is set.
    #[test]
    fn set_npc_name_returns_error_when_no_active_female() {
        let mut world = make_test_world();
        let mut ctx = SceneCtx::new();
        // active_female intentionally left None
        let registry = PackRegistry::new();

        let effect = EffectDef::SetNpcName {
            npc: "f".into(),
            name: "Priya".into(),
        };
        let result = apply_effect(&effect, &mut world, &mut ctx, &registry);

        assert!(
            result.is_err(),
            "must return Err when no active female NPC is set"
        );
        assert!(
            matches!(result, Err(EffectError::NoActiveFemale)),
            "error must be NoActiveFemale, got: {:?}",
            result
        );
    }

    /// BREAKS IF: `apply_effect` panics or returns a generic error when a role reference
    /// cannot be resolved (e.g. role was never bound in this scene context).
    #[test]
    fn set_npc_name_returns_error_for_unbound_role() {
        let mut world = make_test_world();
        let mut ctx = SceneCtx::new();
        // No role binding for ROLE_GHOST
        let registry = PackRegistry::new();

        let effect = EffectDef::SetNpcName {
            npc: "ROLE_GHOST".into(),
            name: "Ghost".into(),
        };
        let result = apply_effect(&effect, &mut world, &mut ctx, &registry);

        assert!(
            result.is_err(),
            "must return Err for an unbound role reference"
        );
        assert!(
            matches!(result, Err(EffectError::BadNpcRef(_))),
            "error must be BadNpcRef for unknown role, got: {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // CRITERION 7 — pack content: canonical first-meeting scenes
    // -----------------------------------------------------------------------

    /// BREAKS IF: coffee_shop.toml does not contain a `set_npc_name` effect for "Jake",
    /// meaning Jake's name is never bound to his NPC record in the sidebar/prose.
    #[test]
    fn coffee_shop_scene_contains_set_npc_name_for_jake() {
        use crate::loader::load_scenes;
        use undone_packs::load_packs;

        let (registry, _) = load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).expect("scenes must load without error");

        let scene = scenes
            .get("base::coffee_shop")
            .expect("base::coffee_shop must be present in the loaded scenes");

        let has_jake_rename = scene
            .actions
            .iter()
            .flat_map(|a| a.effects.iter())
            .chain(scene.npc_actions.iter().flat_map(|a| a.effects.iter()))
            .any(|e| matches!(e, EffectDef::SetNpcName { name, .. } if name == "Jake"));

        assert!(
            has_jake_rename,
            "coffee_shop scene must contain a set_npc_name effect with name = 'Jake'; \
             without it the sidebar shows the spawn name after first meeting"
        );
    }

    /// BREAKS IF: workplace_work_meeting.toml does not contain a `set_npc_name` effect
    /// for "Marcus", meaning Marcus's name is never bound to his NPC record.
    #[test]
    fn workplace_work_meeting_scene_contains_set_npc_name_for_marcus() {
        use crate::loader::load_scenes;
        use undone_packs::load_packs;

        let (registry, _) = load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).expect("scenes must load without error");

        let scene = scenes
            .get("base::workplace_work_meeting")
            .expect("base::workplace_work_meeting must be present");

        let has_marcus_rename = scene
            .actions
            .iter()
            .flat_map(|a| a.effects.iter())
            .chain(scene.npc_actions.iter().flat_map(|a| a.effects.iter()))
            .any(|e| matches!(e, EffectDef::SetNpcName { name, .. } if name == "Marcus"));

        assert!(
            has_marcus_rename,
            "workplace_work_meeting scene must contain a set_npc_name effect with name = 'Marcus'"
        );
    }

    /// BREAKS IF: campus_library.toml does not contain a `set_npc_name` effect for "Theo".
    #[test]
    fn campus_library_scene_contains_set_npc_name_for_theo() {
        use crate::loader::load_scenes;
        use undone_packs::load_packs;

        let (registry, _) = load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).expect("scenes must load without error");

        let scene = scenes
            .get("base::campus_library")
            .expect("base::campus_library must be present");

        let has_theo_rename = scene
            .actions
            .iter()
            .flat_map(|a| a.effects.iter())
            .chain(scene.npc_actions.iter().flat_map(|a| a.effects.iter()))
            .any(|e| matches!(e, EffectDef::SetNpcName { name, .. } if name == "Theo"));

        assert!(
            has_theo_rename,
            "campus_library scene must contain a set_npc_name effect with name = 'Theo'"
        );
    }

    // -----------------------------------------------------------------------
    // Additional wiring check: effect mutates_persistent_world()
    // -----------------------------------------------------------------------

    /// BREAKS IF: `SetNpcName` is omitted from `mutates_persistent_world()`, causing
    /// the scheduler/simulator to treat name overrides as non-persistent and not
    /// considering them in world mutation tracking.
    #[test]
    fn set_npc_name_is_flagged_as_persistent_world_mutation() {
        let effect = EffectDef::SetNpcName {
            npc: "m".into(),
            name: "Jake".into(),
        };
        assert!(
            effect.mutates_persistent_world(),
            "SetNpcName must be flagged as a persistent world mutation"
        );
    }

    // -----------------------------------------------------------------------
    // Additional wiring: apply_effect actually sets display_name on world state
    // (not just reading from ctx or some transient buffer)
    // -----------------------------------------------------------------------

    /// BREAKS IF: apply_effect sets display_name on a copy rather than the live world,
    /// so the NPC in world.male_npcs still has display_name = None after the effect.
    #[test]
    fn apply_effect_set_npc_name_mutates_world_not_a_copy() {
        let mut world = make_test_world();
        let key = world
            .male_npcs
            .insert(make_male_npc_with_spawn_name(spawn_name()));
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let registry = PackRegistry::new();

        apply_effect(
            &EffectDef::SetNpcName {
                npc: "m".into(),
                name: "Jake".into(),
            },
            &mut world,
            &mut ctx,
            &registry,
        )
        .unwrap();

        // Re-read from world to confirm the live world was mutated, not a temporary
        assert_eq!(
            world.male_npcs[key].core.display_name.as_deref(),
            Some("Jake"),
            "world.male_npcs[key].core.display_name must be 'Jake' after apply_effect"
        );
    }

    /// BREAKS IF: set_npc_name via the "f" reference doesn't work, meaning female
    /// NPCs cannot be renamed even though the spec covers them.
    #[test]
    fn apply_effect_set_npc_name_works_for_female_via_f_ref() {
        use undone_domain::{BreastSize, CharTypeId, FemaleClothing, FemaleNpc, PlayerFigure};

        let mut world = make_test_world();
        let female_npc = FemaleNpc {
            core: NpcCore {
                name: "SomeSpawnName".into(),
                display_name: None,
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "Brown".into(),
                hair_colour: "Brown".into(),
                personality: PersonalityId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
                traits: HashSet::new(),
                relationship: RelationshipStatus::Stranger,
                pc_liking: LikingLevel::Neutral,
                npc_liking: LikingLevel::Neutral,
                pc_love: LoveLevel::None,
                npc_love: LoveLevel::None,
                pc_attraction: AttractionLevel::Unattracted,
                npc_attraction: AttractionLevel::Unattracted,
                behaviour: Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: ArousalLevel::Comfort,
                alcohol: AlcoholLevel::Sober,
                roles: HashSet::new(),
            },
            char_type: CharTypeId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::Average,
            clothing: FemaleClothing::default(),
            pregnancy: None,
            virgin: true,
        };
        let key = world.female_npcs.insert(female_npc);
        let mut ctx = SceneCtx::new();
        ctx.active_female = Some(key);
        let registry = PackRegistry::new();

        apply_effect(
            &EffectDef::SetNpcName {
                npc: "f".into(),
                name: "Priya".into(),
            },
            &mut world,
            &mut ctx,
            &registry,
        )
        .unwrap();

        assert_eq!(
            world.female_npcs[key].core.display_name.as_deref(),
            Some("Priya"),
            "set_npc_name via 'f' reference must set display_name on the female NPC"
        );
        assert_eq!(
            world.female_npcs[key].core.name, "SomeSpawnName",
            "spawn name must be preserved on the female NPC"
        );
    }
}
