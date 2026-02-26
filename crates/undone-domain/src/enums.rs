use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArousalLevel {
    Discomfort,
    Comfort,
    Enjoy,
    Close,
    Orgasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlcoholLevel {
    Sober,
    Tipsy,
    Drunk,
    VeryDrunk,
    MaxDrunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LikingLevel {
    Neutral,
    Ok,
    Like,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoveLevel {
    None,
    Some,
    Confused,
    Crush,
    Love,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AttractionLevel {
    Unattracted,
    Ok,
    Attracted,
    Lust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Behaviour {
    Neutral,
    Romantic,
    Mean,
    Cold,
    Faking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipStatus {
    Stranger,
    Acquaintance,
    Friend,
    Partner { cohabiting: bool },
    Married,
    Ex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerFigure {
    Petite,
    Slim,
    Athletic,
    Hourglass,
    Curvy,
    Thick,
    Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaleFigure {
    Average,
    Skinny,
    Toned,
    Muscular,
    Thickset,
    Paunchy,
    Fat,
}

impl std::fmt::Display for MaleFigure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaleFigure::Average => write!(f, "Average"),
            MaleFigure::Skinny => write!(f, "Skinny"),
            MaleFigure::Toned => write!(f, "Toned"),
            MaleFigure::Muscular => write!(f, "Muscular"),
            MaleFigure::Thickset => write!(f, "Thickset"),
            MaleFigure::Paunchy => write!(f, "Paunchy"),
            MaleFigure::Fat => write!(f, "Fat"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreastSize {
    Flat,
    Perky,
    Handful,
    Average,
    Full,
    Big,
    Huge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Age {
    LateTeen,
    EarlyTwenties,
    MidLateTwenties,
    LateTwenties,
    Thirties,
    Forties,
    Fifties,
    Old,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BeforeSexuality {
    AttractedToWomen,
    AttractedToMen,
    AttractedToBoth,
}

impl std::fmt::Display for BeforeSexuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeforeSexuality::AttractedToWomen => write!(f, "Attracted to Women"),
            BeforeSexuality::AttractedToMen => write!(f, "Attracted to Men"),
            BeforeSexuality::AttractedToBoth => write!(f, "Attracted to Both"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PcOrigin {
    CisMaleTransformed,
    TransWomanTransformed,
    CisFemaleTransformed,
    AlwaysFemale,
}

impl PcOrigin {
    /// Was this PC magically transformed?
    pub fn was_transformed(self) -> bool {
        !matches!(self, PcOrigin::AlwaysFemale)
    }

    /// Did this PC have a male body before transformation?
    pub fn was_male_bodied(self) -> bool {
        matches!(
            self,
            PcOrigin::CisMaleTransformed | PcOrigin::TransWomanTransformed
        )
    }

    /// Should the "before" section show in character creation?
    pub fn has_before_life(self) -> bool {
        self.was_transformed()
    }

    /// For backward compat: equivalent to the old `always_female` bool.
    /// True for CisFemaleTransformed and AlwaysFemale.
    pub fn is_always_female(self) -> bool {
        matches!(
            self,
            PcOrigin::CisFemaleTransformed | PcOrigin::AlwaysFemale
        )
    }
}

impl std::fmt::Display for PcOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcOrigin::CisMaleTransformed => write!(f, "Transformed (was cis man)"),
            PcOrigin::TransWomanTransformed => write!(f, "Transformed (was trans woman)"),
            PcOrigin::CisFemaleTransformed => write!(f, "Transformed (was cis woman)"),
            PcOrigin::AlwaysFemale => write!(f, "Always Female"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeSlot {
    #[default]
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl TimeSlot {
    pub fn next(self) -> Option<TimeSlot> {
        match self {
            TimeSlot::Morning => Some(TimeSlot::Afternoon),
            TimeSlot::Afternoon => Some(TimeSlot::Evening),
            TimeSlot::Evening => Some(TimeSlot::Night),
            TimeSlot::Night => None, // day is over
        }
    }
}

impl std::fmt::Display for TimeSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeSlot::Morning => write!(f, "Morning"),
            TimeSlot::Afternoon => write!(f, "Afternoon"),
            TimeSlot::Evening => write!(f, "Evening"),
            TimeSlot::Night => write!(f, "Night"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Personality {
    Romantic,
    Jerk,
    Friend,
    Intellectual,
    Lad,
}

impl std::fmt::Display for ArousalLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArousalLevel::Discomfort => write!(f, "Discomfort"),
            ArousalLevel::Comfort => write!(f, "Comfort"),
            ArousalLevel::Enjoy => write!(f, "Enjoy"),
            ArousalLevel::Close => write!(f, "Close"),
            ArousalLevel::Orgasm => write!(f, "Orgasm"),
        }
    }
}

impl std::fmt::Display for AlcoholLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlcoholLevel::Sober => write!(f, "Sober"),
            AlcoholLevel::Tipsy => write!(f, "Tipsy"),
            AlcoholLevel::Drunk => write!(f, "Drunk"),
            AlcoholLevel::VeryDrunk => write!(f, "Very Drunk"),
            AlcoholLevel::MaxDrunk => write!(f, "Max Drunk"),
        }
    }
}

impl std::fmt::Display for LikingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LikingLevel::Neutral => write!(f, "Neutral"),
            LikingLevel::Ok => write!(f, "Ok"),
            LikingLevel::Like => write!(f, "Like"),
            LikingLevel::Close => write!(f, "Close"),
        }
    }
}

impl std::fmt::Display for AttractionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttractionLevel::Unattracted => write!(f, "Unattracted"),
            AttractionLevel::Ok => write!(f, "Ok"),
            AttractionLevel::Attracted => write!(f, "Attracted"),
            AttractionLevel::Lust => write!(f, "Lust"),
        }
    }
}

impl std::fmt::Display for RelationshipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipStatus::Stranger => write!(f, "Stranger"),
            RelationshipStatus::Acquaintance => write!(f, "Acquaintance"),
            RelationshipStatus::Friend => write!(f, "Friend"),
            RelationshipStatus::Partner { cohabiting: true } => write!(f, "Partner (cohabiting)"),
            RelationshipStatus::Partner { cohabiting: false } => write!(f, "Partner"),
            RelationshipStatus::Married => write!(f, "Married"),
            RelationshipStatus::Ex => write!(f, "Ex"),
        }
    }
}

impl std::fmt::Display for PlayerFigure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerFigure::Petite => write!(f, "Petite"),
            PlayerFigure::Slim => write!(f, "Slim"),
            PlayerFigure::Athletic => write!(f, "Athletic"),
            PlayerFigure::Hourglass => write!(f, "Hourglass"),
            PlayerFigure::Curvy => write!(f, "Curvy"),
            PlayerFigure::Thick => write!(f, "Thick"),
            PlayerFigure::Plus => write!(f, "Plus"),
        }
    }
}

impl std::fmt::Display for BreastSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreastSize::Flat => write!(f, "Flat"),
            BreastSize::Perky => write!(f, "Perky"),
            BreastSize::Handful => write!(f, "Handful"),
            BreastSize::Average => write!(f, "Average"),
            BreastSize::Full => write!(f, "Full"),
            BreastSize::Big => write!(f, "Big"),
            BreastSize::Huge => write!(f, "Huge"),
        }
    }
}

impl std::fmt::Display for Age {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Age::LateTeen => write!(f, "Late Teen"),
            Age::EarlyTwenties => write!(f, "Early Twenties"),
            Age::MidLateTwenties => write!(f, "Mid to Late 20s"),
            Age::LateTwenties => write!(f, "Late Twenties"),
            Age::Thirties => write!(f, "Thirties"),
            Age::Forties => write!(f, "Forties"),
            Age::Fifties => write!(f, "Fifties"),
            Age::Old => write!(f, "Old"),
        }
    }
}

// ---------------------------------------------------------------------------
// Physical attribute enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Height {
    VeryShort,
    Short,
    Average,
    Tall,
    VeryTall,
}

impl std::fmt::Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Height::VeryShort => write!(f, "Very Short"),
            Height::Short => write!(f, "Short"),
            Height::Average => write!(f, "Average"),
            Height::Tall => write!(f, "Tall"),
            Height::VeryTall => write!(f, "Very Tall"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HairLength {
    Buzzed,
    Short,
    Shoulder,
    Long,
    VeryLong,
}

impl std::fmt::Display for HairLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HairLength::Buzzed => write!(f, "Buzzed"),
            HairLength::Short => write!(f, "Short"),
            HairLength::Shoulder => write!(f, "Shoulder"),
            HairLength::Long => write!(f, "Long"),
            HairLength::VeryLong => write!(f, "Very Long"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinTone {
    VeryFair,
    Fair,
    Light,
    Medium,
    Olive,
    Tan,
    Brown,
    DarkBrown,
    Deep,
}

impl std::fmt::Display for SkinTone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkinTone::VeryFair => write!(f, "Very Fair"),
            SkinTone::Fair => write!(f, "Fair"),
            SkinTone::Light => write!(f, "Light"),
            SkinTone::Medium => write!(f, "Medium"),
            SkinTone::Olive => write!(f, "Olive"),
            SkinTone::Tan => write!(f, "Tan"),
            SkinTone::Brown => write!(f, "Brown"),
            SkinTone::DarkBrown => write!(f, "Dark Brown"),
            SkinTone::Deep => write!(f, "Deep"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexion {
    Clear,
    Glowing,
    Normal,
    Rosy,
    Acne,
}

impl std::fmt::Display for Complexion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexion::Clear => write!(f, "Clear"),
            Complexion::Glowing => write!(f, "Glowing"),
            Complexion::Normal => write!(f, "Normal"),
            Complexion::Rosy => write!(f, "Rosy"),
            Complexion::Acne => write!(f, "Acne"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Appearance {
    Plain,
    Average,
    Attractive,
    Beautiful,
    Stunning,
    Devastating,
}

impl std::fmt::Display for Appearance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Appearance::Plain => write!(f, "Plain"),
            Appearance::Average => write!(f, "Average"),
            Appearance::Attractive => write!(f, "Attractive"),
            Appearance::Beautiful => write!(f, "Beautiful"),
            Appearance::Stunning => write!(f, "Stunning"),
            Appearance::Devastating => write!(f, "Devastating"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EyeColour {
    Brown,
    DarkBrown,
    Hazel,
    Green,
    Blue,
    LightBlue,
    Grey,
    Amber,
    Black,
}

impl std::fmt::Display for EyeColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EyeColour::Brown => write!(f, "Brown"),
            EyeColour::DarkBrown => write!(f, "Dark Brown"),
            EyeColour::Hazel => write!(f, "Hazel"),
            EyeColour::Green => write!(f, "Green"),
            EyeColour::Blue => write!(f, "Blue"),
            EyeColour::LightBlue => write!(f, "Light Blue"),
            EyeColour::Grey => write!(f, "Grey"),
            EyeColour::Amber => write!(f, "Amber"),
            EyeColour::Black => write!(f, "Black"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HairColour {
    Black,
    DarkBrown,
    Brown,
    Chestnut,
    Auburn,
    Copper,
    Red,
    Strawberry,
    Blonde,
    HoneyBlonde,
    PlatinumBlonde,
    Silver,
    White,
}

impl std::fmt::Display for HairColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HairColour::Black => write!(f, "Black"),
            HairColour::DarkBrown => write!(f, "Dark Brown"),
            HairColour::Brown => write!(f, "Brown"),
            HairColour::Chestnut => write!(f, "Chestnut"),
            HairColour::Auburn => write!(f, "Auburn"),
            HairColour::Copper => write!(f, "Copper"),
            HairColour::Red => write!(f, "Red"),
            HairColour::Strawberry => write!(f, "Strawberry"),
            HairColour::Blonde => write!(f, "Blonde"),
            HairColour::HoneyBlonde => write!(f, "Honey Blonde"),
            HairColour::PlatinumBlonde => write!(f, "Platinum Blonde"),
            HairColour::Silver => write!(f, "Silver"),
            HairColour::White => write!(f, "White"),
        }
    }
}

// ---------------------------------------------------------------------------
// Sexual/intimate attribute enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NippleSensitivity {
    Low,
    Normal,
    High,
    Extreme,
}

impl std::fmt::Display for NippleSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NippleSensitivity::Low => write!(f, "Low"),
            NippleSensitivity::Normal => write!(f, "Normal"),
            NippleSensitivity::High => write!(f, "High"),
            NippleSensitivity::Extreme => write!(f, "Extreme"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClitSensitivity {
    Low,
    Normal,
    High,
    Extreme,
}

impl std::fmt::Display for ClitSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClitSensitivity::Low => write!(f, "Low"),
            ClitSensitivity::Normal => write!(f, "Normal"),
            ClitSensitivity::High => write!(f, "High"),
            ClitSensitivity::Extreme => write!(f, "Extreme"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PubicHairStyle {
    Natural,
    Trimmed,
    Landing,
    Brazilian,
    Bare,
}

impl std::fmt::Display for PubicHairStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PubicHairStyle::Natural => write!(f, "Natural"),
            PubicHairStyle::Trimmed => write!(f, "Trimmed"),
            PubicHairStyle::Landing => write!(f, "Landing Strip"),
            PubicHairStyle::Brazilian => write!(f, "Brazilian"),
            PubicHairStyle::Bare => write!(f, "Bare"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NaturalPubicHair {
    Full,
    Sparse,
    Minimal,
    None,
}

impl std::fmt::Display for NaturalPubicHair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NaturalPubicHair::Full => write!(f, "Full"),
            NaturalPubicHair::Sparse => write!(f, "Sparse"),
            NaturalPubicHair::Minimal => write!(f, "Minimal"),
            NaturalPubicHair::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InnerLabiaSize {
    Small,
    Average,
    Prominent,
}

impl std::fmt::Display for InnerLabiaSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InnerLabiaSize::Small => write!(f, "Small"),
            InnerLabiaSize::Average => write!(f, "Average"),
            InnerLabiaSize::Prominent => write!(f, "Prominent"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WetnessBaseline {
    Dry,
    Normal,
    Wet,
    Soaking,
}

impl std::fmt::Display for WetnessBaseline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WetnessBaseline::Dry => write!(f, "Dry"),
            WetnessBaseline::Normal => write!(f, "Normal"),
            WetnessBaseline::Wet => write!(f, "Wet"),
            WetnessBaseline::Soaking => write!(f, "Soaking"),
        }
    }
}

// ---------------------------------------------------------------------------
// Body shape enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtSize {
    Flat,
    Small,
    Pert,
    Round,
    Big,
    Huge,
}

impl std::fmt::Display for ButtSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ButtSize::Flat => write!(f, "Flat"),
            ButtSize::Small => write!(f, "Small"),
            ButtSize::Pert => write!(f, "Pert"),
            ButtSize::Round => write!(f, "Round"),
            ButtSize::Big => write!(f, "Big"),
            ButtSize::Huge => write!(f, "Huge"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaistSize {
    Tiny,
    Narrow,
    Average,
    Thick,
    Wide,
}

impl std::fmt::Display for WaistSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaistSize::Tiny => write!(f, "Tiny"),
            WaistSize::Narrow => write!(f, "Narrow"),
            WaistSize::Average => write!(f, "Average"),
            WaistSize::Thick => write!(f, "Thick"),
            WaistSize::Wide => write!(f, "Wide"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LipShape {
    Thin,
    Average,
    Full,
    Plush,
    BeeStung,
}

impl std::fmt::Display for LipShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LipShape::Thin => write!(f, "Thin"),
            LipShape::Average => write!(f, "Average"),
            LipShape::Full => write!(f, "Full"),
            LipShape::Plush => write!(f, "Plush"),
            LipShape::BeeStung => write!(f, "Bee-stung"),
        }
    }
}

// ---------------------------------------------------------------------------
// Before-life / male attribute enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeforeVoice {
    Deep,
    Average,
    Light,
}

impl std::fmt::Display for BeforeVoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeforeVoice::Deep => write!(f, "Deep"),
            BeforeVoice::Average => write!(f, "Average"),
            BeforeVoice::Light => write!(f, "Light"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PenisSize {
    None,
    Micro,
    Small,
    Average,
    AboveAverage,
    Big,
    Huge,
    Massive,
}

impl std::fmt::Display for PenisSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PenisSize::None => write!(f, "None"),
            PenisSize::Micro => write!(f, "Micro"),
            PenisSize::Small => write!(f, "Small"),
            PenisSize::Average => write!(f, "Average"),
            PenisSize::AboveAverage => write!(f, "Above Average"),
            PenisSize::Big => write!(f, "Big"),
            PenisSize::Huge => write!(f, "Huge"),
            PenisSize::Massive => write!(f, "Massive"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arousal_ordering() {
        assert!(ArousalLevel::Orgasm > ArousalLevel::Discomfort);
        assert!(ArousalLevel::Enjoy > ArousalLevel::Comfort);
        assert!(ArousalLevel::Close < ArousalLevel::Orgasm);
    }

    #[test]
    fn relationship_partner_carries_data() {
        let r = RelationshipStatus::Partner { cohabiting: true };
        assert_eq!(r, RelationshipStatus::Partner { cohabiting: true });
        assert_ne!(r, RelationshipStatus::Partner { cohabiting: false });
    }

    #[test]
    fn serde_roundtrip() {
        let level = ArousalLevel::Close;
        let json = serde_json::to_string(&level).unwrap();
        let back: ArousalLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, back);
    }

    #[test]
    fn before_sexuality_serde_roundtrip() {
        let s = BeforeSexuality::AttractedToWomen;
        let json = serde_json::to_string(&s).unwrap();
        let back: BeforeSexuality = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn pc_origin_helpers() {
        assert!(PcOrigin::CisMaleTransformed.was_transformed());
        assert!(PcOrigin::TransWomanTransformed.was_male_bodied());
        assert!(!PcOrigin::AlwaysFemale.was_transformed());
        assert!(PcOrigin::AlwaysFemale.is_always_female());
        assert!(PcOrigin::CisFemaleTransformed.is_always_female());
        assert!(!PcOrigin::CisMaleTransformed.is_always_female());
    }

    #[test]
    fn personality_is_eq() {
        assert_eq!(Personality::Romantic, Personality::Romantic);
        assert_ne!(Personality::Jerk, Personality::Friend);
    }
}
