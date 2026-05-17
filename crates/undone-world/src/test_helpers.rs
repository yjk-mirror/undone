use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use undone_domain::{
    Age, AlcoholLevel, Appearance, ArousalLevel, AttractionLevel, BeforeIdentity, BeforeSexuality,
    BeforeVoice, Behaviour, BoundedStat, BreastSize, ButtSize, ClitSensitivity, Complexion,
    EyeColour, HairColour, HairLength, Height, InnerLabiaSize, LikingLevel, LipShape, LoveLevel,
    MaleClothing, MaleFigure, MaleNpc, NaturalPubicHair, NippleSensitivity, NpcCore, PcOrigin,
    PenisSize, PersonalityId, Player, PlayerFigure, PubicHairStyle, RelationshipStatus, SkinTone,
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

/// Canonical test male NPC ("Jake", stranger to the player).
/// Tests that need different relationship/liking/attraction should mutate the result.
pub fn make_test_male_npc(personality: PersonalityId) -> MaleNpc {
    MaleNpc {
        core: NpcCore {
            name: "Jake".into(),
            age: Age::MidLateTwenties,
            race: "white".into(),
            eye_colour: "blue".into(),
            hair_colour: "brown".into(),
            personality,
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
