use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use undone_domain::{
    Age, AlcoholLevel, Appearance, ArousalLevel, BeforeIdentity, BeforeSexuality, BeforeVoice,
    BoundedStat, BreastSize, ButtSize, ClitSensitivity, Complexion, EyeColour, HairColour,
    HairLength, Height, InnerLabiaSize, LipShape, MaleFigure, NaturalPubicHair,
    NippleSensitivity, PcOrigin, PenisSize, Player, PlayerFigure, PubicHairStyle, SkinTone,
    WaistSize, WetnessBaseline,
};

use crate::{GameData, World};

/// Canonical test World: CisMaleTransformed player with sensible defaults.
/// Tests that need specific field values should mutate the returned world.
pub fn make_test_world() -> World {
    World {
        player: Player {
            name_fem: "Eva".into(),
            name_masc: "Evan".into(),
            before: Some(BeforeIdentity {
                name: "Evan".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                sexuality: BeforeSexuality::AttractedToWomen,
                figure: MaleFigure::Average,
                height: Height::Average,
                hair_colour: HairColour::DarkBrown,
                eye_colour: EyeColour::Brown,
                skin_tone: SkinTone::Medium,
                penis_size: PenisSize::Average,
                voice: BeforeVoice::Average,
                traits: HashSet::new(),
            }),
            age: Age::LateTeen,
            race: "east_asian".into(),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::Full,
            eye_colour: EyeColour::Brown,
            hair_colour: HairColour::DarkBrown,
            height: Height::Average,
            hair_length: HairLength::Shoulder,
            skin_tone: SkinTone::Medium,
            complexion: Complexion::Normal,
            appearance: Appearance::Average,
            butt: ButtSize::Round,
            waist: WaistSize::Average,
            lips: LipShape::Average,
            nipple_sensitivity: NippleSensitivity::Normal,
            clit_sensitivity: ClitSensitivity::Normal,
            pubic_hair: PubicHairStyle::Trimmed,
            natural_pubic_hair: NaturalPubicHair::Full,
            inner_labia: InnerLabiaSize::Average,
            wetness_baseline: WetnessBaseline::Normal,
            traits: HashSet::new(),
            skills: HashMap::new(),
            money: 500,
            stress: BoundedStat::new(10),
            anxiety: BoundedStat::new(0),
            arousal: ArousalLevel::Comfort,
            alcohol: AlcoholLevel::Sober,
            partner: None,
            friends: vec![],
            virgin: true,
            anal_virgin: true,
            lesbian_virgin: true,
            on_pill: false,
            pregnancy: None,
            stuff: HashSet::new(),
            custom_flags: HashMap::new(),
            custom_ints: HashMap::new(),
            origin: PcOrigin::CisMaleTransformed,
        },
        male_npcs: SlotMap::with_key(),
        female_npcs: SlotMap::with_key(),
        game_data: GameData::default(),
    }
}
