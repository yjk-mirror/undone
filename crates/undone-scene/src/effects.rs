use thiserror::Error;
use undone_domain::{
    AlcoholLevel, ArousalLevel, AttractionLevel, Behaviour, FemaleNpcKey, LikingLevel, LoveLevel,
    MaleNpcKey, RelationshipStatus,
};
use undone_expr::{SceneCtx, SceneNpcRef};

#[derive(Debug, Error)]
pub enum EffectError {
    #[error("effect npc ref '{0}' is not 'm', 'f', or a bound scene role")]
    BadNpcRef(String),
    #[error("effect requires active male NPC but none is set")]
    NoActiveMale,
    #[error("effect requires active female NPC but none is set")]
    NoActiveFemale,
    #[error("NPC key not found in world")]
    NpcNotFound,
    #[error("unknown trait '{0}'")]
    UnknownTrait(String),
    #[error("unknown npc trait '{0}'")]
    UnknownNpcTrait(String),
    #[error("unknown skill '{0}'")]
    UnknownSkill(String),
    #[error("unknown stat '{0}'")]
    UnknownStat(String),
    #[error("unknown stuff item '{0}'")]
    UnknownStuff(String),
    #[error("unknown relationship status '{0}'")]
    UnknownRelationshipStatus(String),
    #[error("unknown behaviour '{0}'")]
    UnknownBehaviour(String),
    #[error("unknown virgin_type '{0}'")]
    UnknownVirginType(String),
    #[error("trait conflict: {0}")]
    TraitConflict(String),
}

// ---------------------------------------------------------------------------
// Step helpers
// ---------------------------------------------------------------------------

pub(crate) fn step_liking(current: LikingLevel, delta: i8) -> LikingLevel {
    const LEVELS: [LikingLevel; 4] = [
        LikingLevel::Neutral,
        LikingLevel::Ok,
        LikingLevel::Like,
        LikingLevel::Close,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 3) as usize]
}

pub(crate) fn step_love(current: LoveLevel, delta: i8) -> LoveLevel {
    const LEVELS: [LoveLevel; 5] = [
        LoveLevel::None,
        LoveLevel::Some,
        LoveLevel::Confused,
        LoveLevel::Crush,
        LoveLevel::Love,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 4) as usize]
}

pub(crate) fn step_arousal(current: ArousalLevel, delta: i8) -> ArousalLevel {
    const LEVELS: [ArousalLevel; 5] = [
        ArousalLevel::Discomfort,
        ArousalLevel::Comfort,
        ArousalLevel::Enjoy,
        ArousalLevel::Close,
        ArousalLevel::Orgasm,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 4) as usize]
}

pub(crate) fn step_attraction(current: AttractionLevel, delta: i8) -> AttractionLevel {
    const LEVELS: [AttractionLevel; 4] = [
        AttractionLevel::Unattracted,
        AttractionLevel::Ok,
        AttractionLevel::Attracted,
        AttractionLevel::Lust,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 3) as usize]
}

pub(crate) fn step_alcohol(current: AlcoholLevel, delta: i8) -> AlcoholLevel {
    const LEVELS: [AlcoholLevel; 5] = [
        AlcoholLevel::Sober,
        AlcoholLevel::Tipsy,
        AlcoholLevel::Drunk,
        AlcoholLevel::VeryDrunk,
        AlcoholLevel::MaxDrunk,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 4) as usize]
}

pub(crate) fn parse_relationship_status(s: &str) -> Option<RelationshipStatus> {
    match s {
        "Stranger" => Some(RelationshipStatus::Stranger),
        "Acquaintance" => Some(RelationshipStatus::Acquaintance),
        "Friend" => Some(RelationshipStatus::Friend),
        "Partner" => Some(RelationshipStatus::Partner { cohabiting: false }),
        "PartnerCohabiting" => Some(RelationshipStatus::Partner { cohabiting: true }),
        "Married" => Some(RelationshipStatus::Married),
        "Ex" => Some(RelationshipStatus::Ex),
        _ => None,
    }
}

pub(crate) fn parse_behaviour(s: &str) -> Option<Behaviour> {
    match s {
        "Neutral" => Some(Behaviour::Neutral),
        "Romantic" => Some(Behaviour::Romantic),
        "Mean" => Some(Behaviour::Mean),
        "Cold" => Some(Behaviour::Cold),
        "Faking" => Some(Behaviour::Faking),
        _ => None,
    }
}

pub(crate) enum NpcRef {
    Male(MaleNpcKey),
    Female(FemaleNpcKey),
}

pub(crate) fn resolve_npc_ref(npc: &str, ctx: &SceneCtx) -> Result<NpcRef, EffectError> {
    match npc {
        "m" => ctx
            .active_male
            .map(NpcRef::Male)
            .ok_or(EffectError::NoActiveMale),
        "f" => ctx
            .active_female
            .map(NpcRef::Female)
            .ok_or(EffectError::NoActiveFemale),
        role => match ctx.role_binding(role) {
            Some(SceneNpcRef::Male(key)) => Ok(NpcRef::Male(key)),
            Some(SceneNpcRef::Female(key)) => Ok(NpcRef::Female(key)),
            None => Err(EffectError::BadNpcRef(role.to_string())),
        },
    }
}
