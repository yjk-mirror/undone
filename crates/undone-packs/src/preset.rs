use serde::Deserialize;
use std::path::Path;
use undone_domain::{
    Age, Appearance, BeforeSexuality, BeforeVoice, BreastSize, ButtSize, ClitSensitivity,
    Complexion, EyeColour, HairColour, HairLength, Height, InnerLabiaSize, LipShape, MaleFigure,
    NaturalPubicHair, NippleSensitivity, PcOrigin, PenisSize, PlayerFigure, PubicHairStyle,
    SkinTone, WaistSize, WetnessBaseline,
};

use crate::loader::PackLoadError;

// ── Raw TOML sections ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct IdentityRaw {
    before_name: String,
    name_fem: String,
    name_masc: String,
    origin: PcOrigin,
    before_age: Age,
    before_sexuality: BeforeSexuality,
    before_race: String,
    blurb: String,
}

#[derive(Debug, Deserialize)]
struct BeforeRaw {
    figure: MaleFigure,
    height: Height,
    hair_colour: HairColour,
    eye_colour: EyeColour,
    skin_tone: SkinTone,
    penis_size: PenisSize,
    voice: BeforeVoice,
}

#[derive(Debug, Deserialize)]
struct AfterRaw {
    age: Age,
    race: String,
    figure: PlayerFigure,
    height: Height,
    breasts: BreastSize,
    butt: ButtSize,
    waist: WaistSize,
    lips: LipShape,
    hair_colour: HairColour,
    hair_length: HairLength,
    eye_colour: EyeColour,
    skin_tone: SkinTone,
    complexion: Complexion,
    appearance: Appearance,
    pubic_hair: PubicHairStyle,
    natural_pubic_hair: NaturalPubicHair,
}

#[derive(Debug, Deserialize)]
struct SexualRaw {
    nipple_sensitivity: NippleSensitivity,
    clit_sensitivity: ClitSensitivity,
    inner_labia: InnerLabiaSize,
    wetness_baseline: WetnessBaseline,
}

#[derive(Debug, Deserialize)]
struct PersonalityRaw {
    traits: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GameRaw {
    starting_flags: Vec<String>,
}

/// A single discovery beat in the FemCreation flow (TOML `[[discovery]]`).
///
/// Each beat has a minijinja prose template (with trait branches — same syntax
/// as scene prose) and a list of attribute groups to reveal. The template is
/// rendered against a throwaway World built from the preset, so `w.hasTrait()`,
/// `w.getSkill()`, and all other template methods work.
///
/// There are no player choices in discovery beats — the character's traits
/// determine the reaction. Player agency lives in scenes, not in discovery.
#[derive(Debug, Deserialize)]
struct DiscoveryBeatRaw {
    /// Which attribute groups to reveal after this beat's prose.
    /// Valid values: "scale", "body", "face", "name", "sexual", "begin"
    reveals: Vec<String>,
    /// Minijinja prose template. Rendered with the full template context
    /// (w.hasTrait, w.getSkill, etc.) against a throwaway world.
    prose: String,
}

#[derive(Debug, Deserialize)]
struct PresetFile {
    identity: IdentityRaw,
    before: BeforeRaw,
    after: AfterRaw,
    sexual: SexualRaw,
    personality: PersonalityRaw,
    game: GameRaw,
    /// Discovery beats for interactive FemCreation flow (optional).
    /// If absent, FemCreation falls back to the flat form layout.
    #[serde(default)]
    discovery: Vec<DiscoveryBeatRaw>,
}

// ── Public preset type ──────────────────────────────────────────────────────

/// A character preset loaded from a pack's `data/presets/*.toml` files.
///
/// All fields use domain enums directly (they derive `Deserialize`).
/// Trait IDs are stored as strings — they get resolved to `TraitId` at
/// character creation time via the registry.
#[derive(Debug, Clone)]
pub struct PresetData {
    // Identity
    pub before_name: String,
    pub name_fem: String,
    pub name_masc: String,
    pub origin: PcOrigin,
    pub before_age: Age,
    pub before_sexuality: BeforeSexuality,
    pub before_race: String,
    pub blurb: String,

    // Before-life physical
    pub before_figure: MaleFigure,
    pub before_height: Height,
    pub before_hair_colour: HairColour,
    pub before_eye_colour: EyeColour,
    pub before_skin_tone: SkinTone,
    pub before_penis_size: PenisSize,
    pub before_voice: BeforeVoice,

    // After-transformation physical
    pub age: Age,
    pub race: String,
    pub figure: PlayerFigure,
    pub height: Height,
    pub breasts: BreastSize,
    pub butt: ButtSize,
    pub waist: WaistSize,
    pub lips: LipShape,
    pub hair_colour: HairColour,
    pub hair_length: HairLength,
    pub eye_colour: EyeColour,
    pub skin_tone: SkinTone,
    pub complexion: Complexion,
    pub appearance: Appearance,
    pub pubic_hair: PubicHairStyle,
    pub natural_pubic_hair: NaturalPubicHair,

    // Sexual attributes
    pub nipple_sensitivity: NippleSensitivity,
    pub clit_sensitivity: ClitSensitivity,
    pub inner_labia: InnerLabiaSize,
    pub wetness_baseline: WetnessBaseline,

    // Trait IDs (string form, resolved at creation time)
    pub trait_ids: Vec<String>,

    // Game flags
    pub starting_flags: Vec<String>,

    // Discovery beats for interactive FemCreation (preset-only).
    // Empty means the flat form layout is used instead.
    pub discovery_beats: Vec<DiscoveryBeat>,
}

/// A discovery beat in the FemCreation interactive flow.
///
/// Each beat shows prose (a minijinja template rendered with the full
/// template context) and optionally reveals attribute groups. Trait
/// branches in the prose determine what the player sees — there are no
/// player choices during discovery. The character's traits ARE the reaction.
#[derive(Debug, Clone)]
pub struct DiscoveryBeat {
    /// Minijinja prose template (same syntax as scene prose).
    pub prose: String,
    /// Which attribute groups to reveal after this beat.
    pub reveals: Vec<RevealGroup>,
}

/// Attribute groups that can be revealed during a discovery beat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealGroup {
    /// figure, height
    Scale,
    /// breasts, butt, waist, lips
    Body,
    /// hair, eyes, skin, race, appearance, complexion
    Face,
    /// feminine name input
    Name,
    /// nipple_sensitivity, clit_sensitivity, wetness, etc.
    Sexual,
    /// "Begin Your Story" button
    Begin,
}

impl RevealGroup {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "scale" => Some(Self::Scale),
            "body" => Some(Self::Body),
            "face" => Some(Self::Face),
            "name" => Some(Self::Name),
            "sexual" => Some(Self::Sexual),
            "begin" => Some(Self::Begin),
            _ => None,
        }
    }
}

impl From<PresetFile> for PresetData {
    fn from(f: PresetFile) -> Self {
        Self {
            // Identity
            before_name: f.identity.before_name,
            name_fem: f.identity.name_fem,
            name_masc: f.identity.name_masc,
            origin: f.identity.origin,
            before_age: f.identity.before_age,
            before_sexuality: f.identity.before_sexuality,
            before_race: f.identity.before_race,
            blurb: f.identity.blurb,

            // Before
            before_figure: f.before.figure,
            before_height: f.before.height,
            before_hair_colour: f.before.hair_colour,
            before_eye_colour: f.before.eye_colour,
            before_skin_tone: f.before.skin_tone,
            before_penis_size: f.before.penis_size,
            before_voice: f.before.voice,

            // After
            age: f.after.age,
            race: f.after.race,
            figure: f.after.figure,
            height: f.after.height,
            breasts: f.after.breasts,
            butt: f.after.butt,
            waist: f.after.waist,
            lips: f.after.lips,
            hair_colour: f.after.hair_colour,
            hair_length: f.after.hair_length,
            eye_colour: f.after.eye_colour,
            skin_tone: f.after.skin_tone,
            complexion: f.after.complexion,
            appearance: f.after.appearance,
            pubic_hair: f.after.pubic_hair,
            natural_pubic_hair: f.after.natural_pubic_hair,

            // Sexual
            nipple_sensitivity: f.sexual.nipple_sensitivity,
            clit_sensitivity: f.sexual.clit_sensitivity,
            inner_labia: f.sexual.inner_labia,
            wetness_baseline: f.sexual.wetness_baseline,

            // Traits + flags
            trait_ids: f.personality.traits,
            starting_flags: f.game.starting_flags,

            // Discovery beats
            discovery_beats: f
                .discovery
                .into_iter()
                .map(|beat| DiscoveryBeat {
                    prose: beat.prose,
                    reveals: beat
                        .reveals
                        .iter()
                        .filter_map(|s| RevealGroup::from_str(s))
                        .collect(),
                })
                .collect(),
        }
    }
}

/// Load all preset TOML files from `<pack_dir>/data/presets/`.
///
/// Returns an empty vec if the directory does not exist (presets are optional
/// per pack). Files are sorted alphabetically by filename so the ordering is
/// deterministic across platforms.
pub fn load_presets(pack_dir: &Path) -> Result<Vec<PresetData>, PackLoadError> {
    let presets_dir = pack_dir.join("data").join("presets");
    if !presets_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&presets_dir)
        .map_err(|e| PackLoadError::Io {
            path: presets_dir.clone(),
            source: e,
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "toml")
                .unwrap_or(false)
        })
        .collect();

    // Sort by filename for deterministic ordering (robin.toml before camila.toml
    // is not guaranteed by fs iteration — sort ensures consistent preset indices).
    entries.sort_by_key(|e| e.file_name());

    let mut presets = Vec::new();
    for entry in entries {
        let path = entry.path();
        let src = std::fs::read_to_string(&path).map_err(|e| PackLoadError::Io {
            path: path.clone(),
            source: e,
        })?;
        let preset_file: PresetFile = toml::from_str(&src).map_err(|e| PackLoadError::Toml {
            path: path.clone(),
            message: e.to_string(),
        })?;
        presets.push(PresetData::from(preset_file));
    }

    Ok(presets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn base_pack_dir() -> PathBuf {
        packs_dir().join("base")
    }

    #[test]
    fn test_load_robin_preset() {
        let presets = load_presets(&base_pack_dir()).unwrap();
        // Files are sorted alphabetically: camila.toml comes before robin.toml
        let robin = presets
            .iter()
            .find(|p| p.before_name == "Robin")
            .expect("Robin preset should be loaded");

        assert_eq!(robin.name_fem, "Robin");
        assert_eq!(robin.name_masc, "Robin");
        assert_eq!(robin.origin, PcOrigin::CisMaleTransformed);
        assert_eq!(robin.before_age, Age::Thirties);
        assert_eq!(robin.before_sexuality, BeforeSexuality::AttractedToWomen);
        assert_eq!(robin.before_race, "White");

        // Before physical
        assert_eq!(robin.before_figure, MaleFigure::Average);
        assert_eq!(robin.before_height, Height::Average);
        assert_eq!(robin.before_hair_colour, HairColour::Brown);
        assert_eq!(robin.before_eye_colour, EyeColour::Brown);
        assert_eq!(robin.before_skin_tone, SkinTone::Light);
        assert_eq!(robin.before_penis_size, PenisSize::Average);
        assert_eq!(robin.before_voice, BeforeVoice::Average);

        // After physical
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

        // Sexual
        assert_eq!(robin.nipple_sensitivity, NippleSensitivity::High);
        assert_eq!(robin.clit_sensitivity, ClitSensitivity::High);
        assert_eq!(robin.inner_labia, InnerLabiaSize::Average);
        assert_eq!(robin.wetness_baseline, WetnessBaseline::Wet);

        // Traits
        assert!(robin.trait_ids.contains(&"AMBITIOUS".to_string()));
        assert!(robin.trait_ids.contains(&"HAIR_TRIGGER".to_string()));
        assert!(robin.trait_ids.contains(&"REGULAR_PERIODS".to_string()));

        // Flags
        assert_eq!(robin.starting_flags, vec!["ROUTE_WORKPLACE".to_string()]);
    }

    #[test]
    fn test_load_camila_preset() {
        let presets = load_presets(&base_pack_dir()).unwrap();
        let camila = presets
            .iter()
            .find(|p| p.before_name == "Raul")
            .expect("Camila/Raul preset should be loaded");

        assert_eq!(camila.name_fem, "Camila");
        assert_eq!(camila.name_masc, "Raul");
        assert_eq!(camila.origin, PcOrigin::CisMaleTransformed);
        assert_eq!(camila.before_age, Age::LateTeen);
        assert_eq!(camila.before_sexuality, BeforeSexuality::AttractedToWomen);
        assert_eq!(camila.before_race, "Latina");

        // After physical
        assert_eq!(camila.figure, PlayerFigure::Hourglass);
        assert_eq!(camila.breasts, BreastSize::Full);
        assert_eq!(camila.appearance, Appearance::Attractive);

        // Traits
        assert!(camila.trait_ids.contains(&"AMBITIOUS".to_string()));
        assert!(camila.trait_ids.contains(&"CONFIDENT".to_string()));
        assert!(camila.trait_ids.contains(&"SEXIST".to_string()));

        // Flags
        assert_eq!(camila.starting_flags, vec!["ROUTE_CAMPUS".to_string()]);
    }

    #[test]
    fn test_preset_trait_ids_are_strings() {
        let presets = load_presets(&base_pack_dir()).unwrap();
        assert!(
            !presets.is_empty(),
            "should have loaded at least one preset"
        );
        for preset in &presets {
            for trait_id in &preset.trait_ids {
                assert!(
                    !trait_id.is_empty(),
                    "trait ID should not be empty in preset '{}'",
                    preset.before_name
                );
                // Trait IDs should be uppercase SCREAMING_SNAKE_CASE
                assert!(
                    trait_id.chars().all(|c| c.is_ascii_uppercase() || c == '_'),
                    "trait ID '{}' in preset '{}' should be SCREAMING_SNAKE_CASE",
                    trait_id,
                    preset.before_name
                );
            }
        }
    }

    #[test]
    fn test_load_presets_empty_dir() {
        // A nonexistent directory should return an empty vec, not an error.
        let presets = load_presets(Path::new("/nonexistent/pack")).unwrap();
        assert!(presets.is_empty());
    }

    #[test]
    fn test_preset_ordering_is_deterministic() {
        let presets1 = load_presets(&base_pack_dir()).unwrap();
        let presets2 = load_presets(&base_pack_dir()).unwrap();
        assert_eq!(presets1.len(), presets2.len());
        for (a, b) in presets1.iter().zip(presets2.iter()) {
            assert_eq!(a.before_name, b.before_name);
        }
    }
}
