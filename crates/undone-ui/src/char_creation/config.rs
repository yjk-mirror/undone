//! Character-creation config building: presets, starting traits, defaults, bridge copy.
use undone_domain::{Age, BeforeIdentity, BreastSize, PcOrigin, PlayerFigure, TraitId};
use undone_packs::{char_creation::CharCreationConfig, PackRegistry, PresetData};

use crate::PartialCharState;

// ── Preset character data ─────────────────────────────────────────────────────
//
// Presets are loaded from TOML files in packs/<pack>/data/presets/ and stored in
// PackRegistry::presets(). The `PresetData` type is defined in undone-packs::preset.

pub(crate) const CUSTOM_STARTING_TRAIT_IDS: &[&str] = &[
    "SHY",
    "CUTE",
    "POSH",
    "SULTRY",
    "DOWN_TO_EARTH",
    "BITCHY",
    "REFINED",
    "ROMANTIC",
    "FLIRTY",
    "AMBITIOUS",
    "OUTGOING",
    "OVERACTIVE_IMAGINATION",
    "ANALYTICAL",
    "CONFIDENT",
    "SEXIST",
    "HOMOPHOBIC",
    "OBJECTIFYING",
];

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FemFormDefaults {
    pub(crate) name_fem: String,
    pub(crate) age: Age,
    pub(crate) figure: PlayerFigure,
    pub(crate) breasts: BreastSize,
    pub(crate) race: String,
}

pub fn resolve_starting_traits(
    registry: &PackRegistry,
    trait_names: &[&str],
    include_rough: bool,
    likes_rough: bool,
) -> Result<Vec<TraitId>, String> {
    let mut errors = Vec::new();
    let mut starting_traits = Vec::new();

    for trait_name in trait_names {
        match registry.resolve_trait(trait_name) {
            Ok(trait_id) => starting_traits.push(trait_id),
            Err(_) => errors.push(format!(
                "character creation requires trait '{trait_name}', but it is not registered"
            )),
        }
    }

    if !include_rough {
        match registry.block_rough_trait() {
            Ok(trait_id) => starting_traits.push(trait_id),
            Err(_) => errors.push(
                "character creation requires trait 'BLOCK_ROUGH', but it is not registered"
                    .to_string(),
            ),
        }
    }

    if likes_rough {
        match registry.likes_rough_trait() {
            Ok(trait_id) => starting_traits.push(trait_id),
            Err(_) => errors.push(
                "character creation requires trait 'LIKES_ROUGH', but it is not registered"
                    .to_string(),
            ),
        }
    }

    errors.sort();
    errors.dedup();
    if errors.is_empty() {
        Ok(starting_traits)
    } else {
        Err(format!(
            "Character creation contract error(s):\n{}",
            errors.join("\n")
        ))
    }
}

pub(crate) fn preset_by_idx(registry: &PackRegistry, idx: Option<u8>) -> Option<&PresetData> {
    idx.and_then(|i| registry.presets().get(i as usize))
}

pub(crate) fn fem_form_defaults(
    registry: &PackRegistry,
    partial: Option<&PartialCharState>,
    fallback_race: Option<&str>,
) -> FemFormDefaults {
    if let Some(preset) = partial.and_then(|partial| preset_by_idx(registry, partial.preset_idx)) {
        return FemFormDefaults {
            name_fem: preset.name_fem.clone(),
            age: preset.age,
            figure: preset.figure,
            breasts: preset.breasts,
            race: preset.race.clone(),
        };
    }

    let default_race = partial
        .map(|state| state.before_race.as_str())
        .filter(|race| !race.is_empty())
        .or(fallback_race)
        .unwrap_or("White");
    let default_age = partial
        .map(|state| state.before_age)
        .unwrap_or(Age::EarlyTwenties);

    FemFormDefaults {
        name_fem: "Eva".to_string(),
        age: default_age,
        figure: PlayerFigure::Slim,
        breasts: BreastSize::Full,
        race: default_race.to_string(),
    }
}

pub(crate) fn fem_creation_bridge_copy(partial: Option<&PartialCharState>) -> String {
    let Some(partial) = partial else {
        return "You take a breath and steady yourself. This body is yours. The story starts from here."
            .to_string();
    };

    if partial.origin == PcOrigin::AlwaysFemale {
        return "You smooth your top, check your reflection once, and let yourself arrive in the moment.\n\nNothing happened to your body on the flight. The choice in front of you is simpler and stranger than that: decide who you are going to be, then step into the city as her."
            .to_string();
    }

    let discovery = format!(
        "You wake before landing with the seatbelt pressed into curves that were not there when you closed your eyes. In the airplane bathroom the mirror gives you {} face, sleep-flushed and unmistakably female. You check once, then again, because disbelief keeps reaching for a mistake and not finding one.",
        if partial.before_name.is_empty() {
            "a stranger's".to_string()
        } else {
            format!("a stranger wearing {}", partial.before_name)
        }
    );

    let route_pressure = if partial
        .starting_flags
        .iter()
        .any(|flag| flag == "ROUTE_WORKPLACE")
    {
        "The job is still waiting. There is still an apartment, a lockbox code, a badge photo, a Monday morning meeting. None of that paused just because somewhere over Ohio the shape of your life stopped matching the shape of your body."
    } else if partial
        .starting_flags
        .iter()
        .any(|flag| flag == "ROUTE_CAMPUS")
    {
        "Orientation is still waiting. There is still a dorm room, a roommate you have not met, a campus full of strangers who are about to read you at a glance and move on."
    } else {
        "The city is still waiting. There is still a key, a bed, a first walk outside, and the immediate problem of learning how to move through public space without looking as shaken as you feel."
    };

    format!(
        "{}\n\n{} Right now you need a name, a body, and enough composure to walk out through arrivals without freezing.",
        discovery, route_pressure
    )
}

/// Build a complete CharCreationConfig for the Robin preset.
/// Used by `--quick` start and later dev tooling entry points.
///
/// Panics if Robin is not found in the loaded presets.
pub fn robin_quick_config(registry: &PackRegistry) -> CharCreationConfig {
    let idx = registry
        .presets()
        .iter()
        .position(|p| p.before_name == "Robin")
        .expect("Robin preset must be loaded from base pack");
    config_from_preset(registry, idx)
}

/// Build a CharCreationConfig from a preset at the given index in `registry.presets()`.
///
/// Panics if the index is out of bounds.
pub(crate) fn config_from_preset(registry: &PackRegistry, idx: usize) -> CharCreationConfig {
    let p = &registry.presets()[idx];

    let starting_traits: Vec<TraitId> = p
        .trait_ids
        .iter()
        .filter_map(|trait_id| registry.resolve_trait(trait_id).ok())
        .collect();

    CharCreationConfig {
        name_fem: p.name_fem.clone(),
        name_masc: p.name_masc.clone(),
        age: p.age,
        race: p.race.clone(),
        figure: p.figure,
        breasts: p.breasts,
        origin: p.origin,
        before: Some(BeforeIdentity {
            name: p.before_name.clone(),
            age: p.before_age,
            race: p.before_race.clone(),
            sexuality: p.before_sexuality,
            figure: p.before_figure,
            height: p.before_height,
            hair_colour: p.before_hair_colour,
            eye_colour: p.before_eye_colour,
            skin_tone: p.before_skin_tone,
            penis_size: p.before_penis_size,
            voice: p.before_voice,
            traits: std::collections::HashSet::new(),
        }),
        starting_traits,
        male_count: 6,
        female_count: 3,
        starting_flags: p.starting_flags.iter().cloned().collect(),
        starting_arc_states: std::collections::HashMap::new(),
        height: p.height,
        butt: p.butt,
        waist: p.waist,
        lips: p.lips,
        hair_colour: p.hair_colour,
        hair_length: p.hair_length,
        eye_colour: p.eye_colour,
        skin_tone: p.skin_tone,
        complexion: p.complexion,
        appearance: p.appearance,
        pubic_hair: p.pubic_hair,
        natural_pubic_hair: p.natural_pubic_hair,
        nipple_sensitivity: p.nipple_sensitivity,
        clit_sensitivity: p.clit_sensitivity,
        inner_labia: p.inner_labia,
        wetness_baseline: p.wetness_baseline,
    }
}

// ── PC origin helpers ─────────────────────────────────────────────────────────

pub(crate) fn origin_from_idx(idx: u8) -> PcOrigin {
    match idx {
        0 => PcOrigin::CisMaleTransformed,
        1 => PcOrigin::TransWomanTransformed,
        2 => PcOrigin::CisFemaleTransformed,
        _ => PcOrigin::AlwaysFemale,
    }
}
