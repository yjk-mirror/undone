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
    Slim,
    Toned,
    Womanly,
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
    Small,
    MediumSmall,
    MediumLarge,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Age {
    LateTeen,
    EarlyTwenties,
    Twenties,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeSlot {
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl Default for TimeSlot {
    fn default() -> Self {
        TimeSlot::Morning
    }
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
            PlayerFigure::Slim => write!(f, "Slim"),
            PlayerFigure::Toned => write!(f, "Toned"),
            PlayerFigure::Womanly => write!(f, "Womanly"),
        }
    }
}

impl std::fmt::Display for BreastSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreastSize::Small => write!(f, "Small"),
            BreastSize::MediumSmall => write!(f, "Medium Small"),
            BreastSize::MediumLarge => write!(f, "Medium Large"),
            BreastSize::Large => write!(f, "Large"),
        }
    }
}

impl std::fmt::Display for Age {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Age::LateTeen => write!(f, "Late Teen"),
            Age::EarlyTwenties => write!(f, "Early Twenties"),
            Age::Twenties => write!(f, "Twenties"),
            Age::LateTwenties => write!(f, "Late Twenties"),
            Age::Thirties => write!(f, "Thirties"),
            Age::Forties => write!(f, "Forties"),
            Age::Fifties => write!(f, "Fifties"),
            Age::Old => write!(f, "Old"),
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
