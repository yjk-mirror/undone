//! Integration tests for character preset loading from TOML files.
//!
//! These tests verify the full pipeline: TOML file on disk -> PresetData ->
//! PackRegistry -> CharCreationConfig -> World (via new_game).
//!
//! Written independently from the implementation to catch wiring gaps,
//! field mapping errors, and data integrity issues.

use std::collections::HashSet;
use std::path::PathBuf;

use rand::SeedableRng;
use undone_domain::{
    Age, Appearance, BeforeSexuality, BeforeVoice, BreastSize, ButtSize, ClitSensitivity,
    Complexion, EyeColour, HairColour, HairLength, Height, InnerLabiaSize, LipShape, MaleFigure,
    NaturalPubicHair, NippleSensitivity, PcOrigin, PenisSize, PlayerFigure, PubicHairStyle,
    SkinTone, WaistSize, WetnessBaseline,
};
use undone_packs::{load_packs, PackRegistry, PresetData};

fn packs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs")
}

fn load_registry() -> PackRegistry {
    let (registry, _) = load_packs(&packs_dir()).expect("base pack should load without error");
    registry
}

// ── Acceptance criterion 1: Preset TOML files load correctly ────────────────

// BREAKS IF: TOML files are missing, malformed, or load_presets is never called
#[test]
fn presets_load_from_pack_directory() {
    let registry = load_registry();
    let presets = registry.presets();
    assert!(
        presets.len() >= 2,
        "expected at least 2 presets (Robin + Camila), got {}",
        presets.len()
    );
}

// ── Acceptance criterion 2: Robin preset identity ───────────────────────────

// BREAKS IF: Robin TOML data is wrong or fields are mapped incorrectly
#[test]
fn robin_preset_has_correct_identity() {
    let registry = load_registry();
    let robin = registry
        .presets()
        .iter()
        .find(|p| p.before_name == "Robin")
        .expect("Robin preset must exist in loaded presets");

    assert_eq!(robin.before_name, "Robin");
    assert_eq!(robin.name_fem, "Robin");
    assert_eq!(robin.name_masc, "Robin");
    assert_eq!(robin.origin, PcOrigin::CisMaleTransformed);
    assert_eq!(robin.before_age, Age::Thirties);
    assert_eq!(robin.before_sexuality, BeforeSexuality::AttractedToWomen);
    assert_eq!(robin.before_race, "White");
}

// BREAKS IF: Robin's before-life physical attributes are not loaded from TOML
#[test]
fn robin_preset_has_correct_before_physical() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);

    assert_eq!(robin.before_figure, MaleFigure::Average);
    assert_eq!(robin.before_height, Height::Average);
    assert_eq!(robin.before_hair_colour, HairColour::Brown);
    assert_eq!(robin.before_eye_colour, EyeColour::Brown);
    assert_eq!(robin.before_skin_tone, SkinTone::Light);
    assert_eq!(robin.before_penis_size, PenisSize::Average);
    assert_eq!(robin.before_voice, BeforeVoice::Average);
}

// BREAKS IF: Robin's after-transformation physical attributes are wrong
#[test]
fn robin_preset_has_correct_after_physical() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);

    assert_eq!(robin.age, Age::LateTeen);
    assert_eq!(robin.race, "East Asian");
    assert_eq!(robin.figure, PlayerFigure::Petite);
    assert_eq!(robin.height, Height::Short);
    assert_eq!(robin.breasts, BreastSize::Huge);
    assert_eq!(robin.butt, ButtSize::Big);
    assert_eq!(robin.waist, WaistSize::Narrow);
    assert_eq!(robin.lips, LipShape::Full);
    assert_eq!(robin.hair_colour, HairColour::Black);
    assert_eq!(robin.hair_length, HairLength::Long);
    assert_eq!(robin.eye_colour, EyeColour::DarkBrown);
    assert_eq!(robin.skin_tone, SkinTone::Light);
    assert_eq!(robin.complexion, Complexion::Glowing);
    assert_eq!(robin.appearance, Appearance::Stunning);
    assert_eq!(robin.pubic_hair, PubicHairStyle::Bare);
    assert_eq!(robin.natural_pubic_hair, NaturalPubicHair::None);
}

// BREAKS IF: Robin's sexual attributes are not loaded from TOML
#[test]
fn robin_preset_has_correct_sexual_attributes() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);

    assert_eq!(robin.nipple_sensitivity, NippleSensitivity::High);
    assert_eq!(robin.clit_sensitivity, ClitSensitivity::High);
    assert_eq!(robin.inner_labia, InnerLabiaSize::Average);
    assert_eq!(robin.wetness_baseline, WetnessBaseline::Wet);
}

// ── Acceptance criterion 3: Camila preset identity ──────────────────────────

// BREAKS IF: Camila TOML data is wrong or before_name/name_fem mapping is swapped
#[test]
fn camila_preset_has_correct_identity() {
    let registry = load_registry();
    let camila = registry
        .presets()
        .iter()
        .find(|p| p.before_name == "Raul")
        .expect("Camila/Raul preset must exist in loaded presets");

    assert_eq!(camila.before_name, "Raul");
    assert_eq!(camila.name_fem, "Camila");
    assert_eq!(camila.name_masc, "Raul");
    assert_eq!(camila.origin, PcOrigin::CisMaleTransformed);
    assert_eq!(camila.before_age, Age::LateTeen);
    assert_eq!(camila.before_sexuality, BeforeSexuality::AttractedToWomen);
    assert_eq!(camila.before_race, "Latina");
}

// BREAKS IF: Camila's after-transformation physical attributes are wrong
#[test]
fn camila_preset_has_correct_after_physical() {
    let registry = load_registry();
    let camila = camila_from_registry(&registry);

    assert_eq!(camila.age, Age::LateTeen);
    assert_eq!(camila.race, "Latina");
    assert_eq!(camila.figure, PlayerFigure::Hourglass);
    assert_eq!(camila.height, Height::Average);
    assert_eq!(camila.breasts, BreastSize::Full);
    assert_eq!(camila.butt, ButtSize::Round);
    assert_eq!(camila.waist, WaistSize::Average);
    assert_eq!(camila.lips, LipShape::Average);
    assert_eq!(camila.hair_colour, HairColour::DarkBrown);
    assert_eq!(camila.hair_length, HairLength::Shoulder);
    assert_eq!(camila.eye_colour, EyeColour::DarkBrown);
    assert_eq!(camila.skin_tone, SkinTone::Olive);
    assert_eq!(camila.complexion, Complexion::Normal);
    assert_eq!(camila.appearance, Appearance::Attractive);
    assert_eq!(camila.pubic_hair, PubicHairStyle::Trimmed);
    assert_eq!(camila.natural_pubic_hair, NaturalPubicHair::Full);
}

// BREAKS IF: Camila's before-life physical attributes are wrong
#[test]
fn camila_preset_has_correct_before_physical() {
    let registry = load_registry();
    let camila = camila_from_registry(&registry);

    assert_eq!(camila.before_figure, MaleFigure::Toned);
    assert_eq!(camila.before_height, Height::Tall);
    assert_eq!(camila.before_hair_colour, HairColour::Black);
    assert_eq!(camila.before_eye_colour, EyeColour::DarkBrown);
    assert_eq!(camila.before_skin_tone, SkinTone::Olive);
    assert_eq!(camila.before_penis_size, PenisSize::AboveAverage);
    assert_eq!(camila.before_voice, BeforeVoice::Average);
}

// BREAKS IF: Camila's sexual attributes are wrong
#[test]
fn camila_preset_has_correct_sexual_attributes() {
    let registry = load_registry();
    let camila = camila_from_registry(&registry);

    assert_eq!(camila.nipple_sensitivity, NippleSensitivity::Normal);
    assert_eq!(camila.clit_sensitivity, ClitSensitivity::Normal);
    assert_eq!(camila.inner_labia, InnerLabiaSize::Average);
    assert_eq!(camila.wetness_baseline, WetnessBaseline::Normal);
}

// ── Acceptance criterion 4: Presets accessible via PackRegistry::presets() ───

// BREAKS IF: register_presets is never called or presets() returns wrong data
#[test]
fn presets_accessible_via_registry_after_load_packs() {
    let registry = load_registry();
    let presets = registry.presets();

    // Verify we got real data, not empty structs
    let names: Vec<&str> = presets.iter().map(|p| p.before_name.as_str()).collect();
    assert!(
        names.contains(&"Robin"),
        "presets() must include Robin, got: {:?}",
        names
    );
    assert!(
        names.contains(&"Raul"),
        "presets() must include Camila/Raul, got: {:?}",
        names
    );
}

// ── Acceptance criterion 5: Deterministic ordering ──────────────────────────

// BREAKS IF: files are not sorted alphabetically by filename, or 01/02 prefix is wrong
#[test]
fn preset_ordering_is_robin_first_camila_second() {
    let registry = load_registry();
    let presets = registry.presets();

    assert!(
        presets.len() >= 2,
        "need at least 2 presets to verify ordering"
    );
    assert_eq!(
        presets[0].before_name, "Robin",
        "Robin (01-robin.toml) must be at index 0"
    );
    assert_eq!(
        presets[1].before_name, "Raul",
        "Camila/Raul (02-camila.toml) must be at index 1"
    );
}

// BREAKS IF: ordering is platform-dependent or changes between loads
#[test]
fn preset_ordering_is_stable_across_multiple_loads() {
    let reg1 = load_registry();
    let reg2 = load_registry();

    let names1: Vec<&str> = reg1.presets().iter().map(|p| p.before_name.as_str()).collect();
    let names2: Vec<&str> = reg2.presets().iter().map(|p| p.before_name.as_str()).collect();
    assert_eq!(names1, names2, "preset ordering must be stable across loads");
}

// ── Acceptance criterion 6: Trait IDs are valid SCREAMING_SNAKE_CASE ────────

// BREAKS IF: trait IDs in TOML have lowercase, spaces, or other invalid characters
#[test]
fn all_preset_trait_ids_are_screaming_snake_case() {
    let registry = load_registry();
    for preset in registry.presets() {
        for trait_id in &preset.trait_ids {
            assert!(
                !trait_id.is_empty(),
                "empty trait ID in preset '{}'",
                preset.before_name
            );
            assert!(
                trait_id.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                "trait ID '{}' in preset '{}' is not SCREAMING_SNAKE_CASE",
                trait_id,
                preset.before_name
            );
        }
    }
}

// BREAKS IF: preset TOML references trait IDs that are not defined in the pack's trait data
#[test]
fn all_preset_trait_ids_resolve_to_registered_traits() {
    let registry = load_registry();
    for preset in registry.presets() {
        for trait_id in &preset.trait_ids {
            let result = registry.resolve_trait(trait_id);
            assert!(
                result.is_ok(),
                "trait ID '{}' in preset '{}' is not registered in the pack: {:?}",
                trait_id,
                preset.before_name,
                result.err()
            );
        }
    }
}

// ── Acceptance criterion 7: Starting flags match expected routes ─────────────

// BREAKS IF: Robin's TOML starting_flags are wrong or not loaded
#[test]
fn robin_preset_has_route_workplace_flag() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);
    assert_eq!(
        robin.starting_flags,
        vec!["ROUTE_WORKPLACE".to_string()],
        "Robin must have exactly ROUTE_WORKPLACE"
    );
}

// BREAKS IF: Camila's TOML starting_flags are wrong or not loaded
#[test]
fn camila_preset_has_route_campus_flag() {
    let registry = load_registry();
    let camila = camila_from_registry(&registry);
    assert_eq!(
        camila.starting_flags,
        vec!["ROUTE_CAMPUS".to_string()],
        "Camila must have exactly ROUTE_CAMPUS"
    );
}

// ── Acceptance criterion 8: robin_quick_config still works ──────────────────

// BREAKS IF: robin_quick_config panics or returns wrong data
#[test]
fn robin_quick_config_produces_valid_config() {
    let registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);

    assert_eq!(config.name_fem, "Robin");
    assert_eq!(config.name_masc, "Robin");
    assert_eq!(config.origin, PcOrigin::CisMaleTransformed);
    assert!(
        config.starting_flags.contains("ROUTE_WORKPLACE"),
        "robin_quick_config must set ROUTE_WORKPLACE flag"
    );
}

// BREAKS IF: robin_quick_config maps physical attributes incorrectly
#[test]
fn robin_quick_config_maps_all_physical_attributes_from_toml() {
    let registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);

    // After-transformation fields must match TOML exactly
    assert_eq!(config.age, Age::LateTeen);
    assert_eq!(config.race, "East Asian");
    assert_eq!(config.figure, PlayerFigure::Petite);
    assert_eq!(config.height, Height::Short);
    assert_eq!(config.breasts, BreastSize::Huge);
    assert_eq!(config.butt, ButtSize::Big);
    assert_eq!(config.waist, WaistSize::Narrow);
    assert_eq!(config.lips, LipShape::Full);
    assert_eq!(config.hair_colour, HairColour::Black);
    assert_eq!(config.hair_length, HairLength::Long);
    assert_eq!(config.eye_colour, EyeColour::DarkBrown);
    assert_eq!(config.skin_tone, SkinTone::Light);
    assert_eq!(config.complexion, Complexion::Glowing);
    assert_eq!(config.appearance, Appearance::Stunning);
    assert_eq!(config.pubic_hair, PubicHairStyle::Bare);
    assert_eq!(config.natural_pubic_hair, NaturalPubicHair::None);
}

// BREAKS IF: robin_quick_config maps sexual attributes incorrectly
#[test]
fn robin_quick_config_maps_all_sexual_attributes_from_toml() {
    let registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);

    assert_eq!(config.nipple_sensitivity, NippleSensitivity::High);
    assert_eq!(config.clit_sensitivity, ClitSensitivity::High);
    assert_eq!(config.inner_labia, InnerLabiaSize::Average);
    assert_eq!(config.wetness_baseline, WetnessBaseline::Wet);
}

// BREAKS IF: robin_quick_config builds BeforeIdentity incorrectly
#[test]
fn robin_quick_config_maps_before_identity_from_toml() {
    let registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);

    let before = config.before.expect("Robin must have a before identity");
    assert_eq!(before.name, "Robin");
    assert_eq!(before.age, Age::Thirties);
    assert_eq!(before.race, "White");
    assert_eq!(before.sexuality, BeforeSexuality::AttractedToWomen);
    assert_eq!(before.figure, MaleFigure::Average);
    assert_eq!(before.height, Height::Average);
    assert_eq!(before.hair_colour, HairColour::Brown);
    assert_eq!(before.eye_colour, EyeColour::Brown);
    assert_eq!(before.skin_tone, SkinTone::Light);
    assert_eq!(before.penis_size, PenisSize::Average);
    assert_eq!(before.voice, BeforeVoice::Average);
}

// BREAKS IF: trait resolution silently drops all traits (filter_map returns empty)
#[test]
fn robin_quick_config_resolves_all_traits() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);
    let config = undone_ui::char_creation::robin_quick_config(&registry);

    assert_eq!(
        config.starting_traits.len(),
        robin.trait_ids.len(),
        "config should have the same number of resolved traits as the TOML file defines ({}), \
         got {}. Some traits may not be registered.",
        robin.trait_ids.len(),
        config.starting_traits.len()
    );
}

// ── Acceptance criterion 9: validate-pack passes ────────────────────────────

// BREAKS IF: validate-pack cannot create a simulation world from presets
#[test]
fn validate_pack_simulation_builds_world_from_robin_preset() {
    let mut registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    // The world must have a player with Robin's identity
    assert_eq!(world.player.name_fem, "Robin");
    assert_eq!(world.player.origin, PcOrigin::CisMaleTransformed);
    assert!(
        world.game_data.has_flag("ROUTE_WORKPLACE"),
        "game should have ROUTE_WORKPLACE flag set from Robin preset"
    );
}

// ── Acceptance criterion 10: Full flow preset -> CharCreationConfig -> World ─

// BREAKS IF: new_game drops starting_flags from preset config
#[test]
fn new_game_from_robin_preset_sets_route_flag() {
    let mut registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    assert!(
        world.game_data.has_flag("ROUTE_WORKPLACE"),
        "ROUTE_WORKPLACE flag must be set after new_game with Robin preset"
    );
    assert!(
        !world.game_data.has_flag("ROUTE_CAMPUS"),
        "Robin preset should NOT set ROUTE_CAMPUS"
    );
}

// BREAKS IF: new_game drops player traits from preset config
#[test]
fn new_game_from_robin_preset_applies_all_traits() {
    let mut registry = load_registry();
    let robin = robin_from_registry(&registry);
    let expected_trait_count = robin.trait_ids.len();

    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(100);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    // Player should have all the traits from the preset
    assert_eq!(
        world.player.traits.len(),
        expected_trait_count,
        "player should have exactly {} traits from Robin preset TOML, got {}",
        expected_trait_count,
        world.player.traits.len()
    );

    // Spot-check specific traits by resolving from registry
    let ambitious = registry.resolve_trait("AMBITIOUS").unwrap();
    assert!(
        world.player.has_trait(ambitious),
        "player should have AMBITIOUS trait from Robin preset"
    );

    let hair_trigger = registry.resolve_trait("HAIR_TRIGGER").unwrap();
    assert!(
        world.player.has_trait(hair_trigger),
        "player should have HAIR_TRIGGER trait from Robin preset"
    );

    let regular_periods = registry.resolve_trait("REGULAR_PERIODS").unwrap();
    assert!(
        world.player.has_trait(regular_periods),
        "player should have REGULAR_PERIODS trait from Robin preset"
    );
}

// BREAKS IF: new_game maps physical attributes incorrectly from preset config
#[test]
fn new_game_from_robin_preset_sets_correct_physical_state() {
    let mut registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(101);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    assert_eq!(world.player.age, Age::LateTeen);
    assert_eq!(world.player.race, "East Asian");
    assert_eq!(world.player.figure, PlayerFigure::Petite);
    assert_eq!(world.player.height, Height::Short);
    assert_eq!(world.player.breasts, BreastSize::Huge);
    assert_eq!(world.player.butt, ButtSize::Big);
    assert_eq!(world.player.waist, WaistSize::Narrow);
    assert_eq!(world.player.lips, LipShape::Full);
    assert_eq!(world.player.hair_colour, HairColour::Black);
    assert_eq!(world.player.hair_length, HairLength::Long);
    assert_eq!(world.player.eye_colour, EyeColour::DarkBrown);
    assert_eq!(world.player.skin_tone, SkinTone::Light);
    assert_eq!(world.player.complexion, Complexion::Glowing);
    assert_eq!(world.player.appearance, Appearance::Stunning);
    assert_eq!(world.player.pubic_hair, PubicHairStyle::Bare);
    assert_eq!(world.player.natural_pubic_hair, NaturalPubicHair::None);
}

// BREAKS IF: new_game maps sexual attributes incorrectly from preset config
#[test]
fn new_game_from_robin_preset_sets_correct_sexual_state() {
    let mut registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(102);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    assert_eq!(world.player.nipple_sensitivity, NippleSensitivity::High);
    assert_eq!(world.player.clit_sensitivity, ClitSensitivity::High);
    assert_eq!(world.player.inner_labia, InnerLabiaSize::Average);
    assert_eq!(world.player.wetness_baseline, WetnessBaseline::Wet);
}

// BREAKS IF: new_game sets wrong femininity for CisMaleTransformed origin
#[test]
fn new_game_from_robin_preset_sets_femininity_10() {
    let mut registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(103);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    let fem_id = registry
        .resolve_skill("FEMININITY")
        .expect("FEMININITY skill must be registered");
    assert_eq!(
        world.player.skill(fem_id),
        10,
        "CisMaleTransformed origin should start with FEMININITY 10"
    );
}

// BREAKS IF: before identity is dropped during preset -> config -> new_game pipeline
#[test]
fn new_game_from_robin_preset_preserves_before_identity() {
    let mut registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(104);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    let before = world
        .player
        .before
        .as_ref()
        .expect("CisMaleTransformed player must have before identity");
    assert_eq!(before.name, "Robin");
    assert_eq!(before.age, Age::Thirties);
    assert_eq!(before.race, "White");
    assert_eq!(before.sexuality, BeforeSexuality::AttractedToWomen);
    assert_eq!(before.figure, MaleFigure::Average);
    assert_eq!(before.height, Height::Average);
    assert_eq!(before.penis_size, PenisSize::Average);
}

// ── Negative / error-path tests ─────────────────────────────────────────────

// BREAKS IF: load_presets crashes instead of returning empty vec for non-preset packs
#[test]
fn load_presets_returns_empty_for_directory_without_presets() {
    let result = undone_packs::preset::load_presets(std::path::Path::new("/nonexistent/pack"));
    assert!(
        result.is_ok(),
        "missing preset directory should return Ok(empty), got error: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_empty(),
        "missing preset directory should return empty vec"
    );
}

// BREAKS IF: Robin preset blurb is empty or a placeholder
#[test]
fn robin_preset_blurb_is_substantive() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);
    assert!(
        robin.blurb.len() > 50,
        "Robin's blurb should be substantial prose, got only {} chars: '{}'",
        robin.blurb.len(),
        robin.blurb
    );
    assert!(
        robin.blurb.contains("software engineer") || robin.blurb.contains("thirty-two"),
        "Robin's blurb should reference his backstory"
    );
}

// BREAKS IF: Camila preset blurb is empty or a placeholder
#[test]
fn camila_preset_blurb_is_substantive() {
    let registry = load_registry();
    let camila = camila_from_registry(&registry);
    assert!(
        camila.blurb.len() > 50,
        "Camila's blurb should be substantial prose, got only {} chars: '{}'",
        camila.blurb.len(),
        camila.blurb
    );
    assert!(
        camila.blurb.contains("eighteen") || camila.blurb.contains("university"),
        "Camila's blurb should reference his backstory"
    );
}

// BREAKS IF: Robin has zero traits (entire trait list was lost during loading)
#[test]
fn robin_preset_has_expected_trait_count() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);
    // Robin's TOML defines exactly 36 traits across personality (4), physical (11),
    // sexual response (12), arousal response (5), sexual preference (2), dark (1), body (1).
    assert_eq!(
        robin.trait_ids.len(),
        36,
        "Robin should have exactly 36 traits as defined in 01-robin.toml"
    );
}

// BREAKS IF: Camila has zero traits or wrong count
#[test]
fn camila_preset_has_expected_trait_count() {
    let registry = load_registry();
    let camila = camila_from_registry(&registry);
    // Camila's TOML defines exactly 5 traits
    assert_eq!(
        camila.trait_ids.len(),
        5,
        "Camila should have exactly 5 traits as defined in 02-camila.toml"
    );
}

// BREAKS IF: starting_flags contain empty strings or whitespace
#[test]
fn preset_starting_flags_are_non_empty_screaming_snake_case() {
    let registry = load_registry();
    for preset in registry.presets() {
        assert!(
            !preset.starting_flags.is_empty(),
            "preset '{}' must have at least one starting flag",
            preset.before_name
        );
        for flag in &preset.starting_flags {
            assert!(
                !flag.is_empty(),
                "empty starting flag in preset '{}'",
                preset.before_name
            );
            assert!(
                flag.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                "starting flag '{}' in preset '{}' is not SCREAMING_SNAKE_CASE",
                flag,
                preset.before_name
            );
        }
    }
}

// BREAKS IF: Robin's specific expected traits are missing from the TOML
#[test]
fn robin_preset_contains_expected_specific_traits() {
    let registry = load_registry();
    let robin = robin_from_registry(&registry);

    let expected = [
        "AMBITIOUS",
        "ANALYTICAL",
        "DOWN_TO_EARTH",
        "OBJECTIFYING",
        "STRAIGHT_HAIR",
        "SWEET_VOICE",
        "HAIR_TRIGGER",
        "HEAVY_SQUIRTER",
        "MULTI_ORGASMIC",
        "SUBMISSIVE",
        "NIPPLE_GETTER",
        "FLUSHER",
        "FREEZE_RESPONSE",
        "REGULAR_PERIODS",
    ];

    let trait_set: HashSet<&str> = robin.trait_ids.iter().map(|s| s.as_str()).collect();
    for expected_trait in &expected {
        assert!(
            trait_set.contains(expected_trait),
            "Robin preset is missing expected trait '{}'",
            expected_trait
        );
    }
}

// BREAKS IF: Camila's specific expected traits are missing
#[test]
fn camila_preset_contains_expected_specific_traits() {
    let registry = load_registry();
    let camila = camila_from_registry(&registry);

    let expected = ["AMBITIOUS", "CONFIDENT", "OUTGOING", "SEXIST", "HOMOPHOBIC"];

    let trait_set: HashSet<&str> = camila.trait_ids.iter().map(|s| s.as_str()).collect();
    for expected_trait in &expected {
        assert!(
            trait_set.contains(expected_trait),
            "Camila preset is missing expected trait '{}'",
            expected_trait
        );
    }
}

// BREAKS IF: NPC spawning fails when using preset-derived config
#[test]
fn new_game_from_robin_preset_spawns_npcs() {
    let mut registry = load_registry();
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(105);
    let world = undone_packs::new_game(config, &mut registry, &mut rng);

    assert_eq!(
        world.male_npcs.len(),
        6,
        "robin_quick_config should spawn 6 male NPCs"
    );
    assert_eq!(
        world.female_npcs.len(),
        3,
        "robin_quick_config should spawn 3 female NPCs"
    );
}

// BREAKS IF: validate_registry_contract reports errors for preset trait IDs
#[test]
fn validate_registry_contract_passes_with_loaded_presets() {
    let registry = load_registry();
    let errors = undone_ui::char_creation::validate_registry_contract(&registry);
    assert!(
        errors.is_empty(),
        "registry contract validation should pass with loaded presets, got errors: {:?}",
        errors
    );
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn robin_from_registry(registry: &PackRegistry) -> &PresetData {
    registry
        .presets()
        .iter()
        .find(|p| p.before_name == "Robin")
        .expect("Robin preset must be loaded")
}

fn camila_from_registry(registry: &PackRegistry) -> &PresetData {
    registry
        .presets()
        .iter()
        .find(|p| p.before_name == "Raul")
        .expect("Camila/Raul preset must be loaded")
}
