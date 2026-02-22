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
}
