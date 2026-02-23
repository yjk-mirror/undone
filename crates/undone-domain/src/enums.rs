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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sexuality {
    StraightMale, // was attracted to women; now attracted to men = new territory
    GayMale,      // was attracted to men; now attracted to men = familiar desire, new position
    BiMale,       // was attracted to both
    AlwaysFemale, // always_female=true; before_sexuality is not applicable
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

impl std::fmt::Display for Sexuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sexuality::StraightMale => write!(f, "Straight Male"),
            Sexuality::GayMale => write!(f, "Gay Male"),
            Sexuality::BiMale => write!(f, "Bi Male"),
            Sexuality::AlwaysFemale => write!(f, "Always Female"),
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
    fn sexuality_serde_roundtrip() {
        let s = Sexuality::StraightMale;
        let json = serde_json::to_string(&s).unwrap();
        let back: Sexuality = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn personality_is_eq() {
        assert_eq!(Personality::Romantic, Personality::Romantic);
        assert_ne!(Personality::Jerk, Personality::Friend);
    }
}
