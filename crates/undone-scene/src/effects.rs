use thiserror::Error;
use undone_domain::{
    AlcoholLevel, ArousalLevel, AttractionLevel, Behaviour, FemaleNpcKey, LikingLevel, LoveLevel,
    MaleNpcKey, NpcKey, RelationshipStatus, SkillValue,
};
use undone_expr::SceneCtx;
use undone_packs::PackRegistry;
use undone_world::World;

use crate::types::EffectDef;

#[derive(Debug, Error)]
pub enum EffectError {
    #[error("effect 'add_npc_liking': npc ref '{0}' is not 'm' or 'f'")]
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
}

// ---------------------------------------------------------------------------
// Step helpers
// ---------------------------------------------------------------------------

fn step_liking(current: LikingLevel, delta: i8) -> LikingLevel {
    const LEVELS: [LikingLevel; 4] = [
        LikingLevel::Neutral,
        LikingLevel::Ok,
        LikingLevel::Like,
        LikingLevel::Close,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 3) as usize]
}

fn step_love(current: LoveLevel, delta: i8) -> LoveLevel {
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

fn step_arousal(current: ArousalLevel, delta: i8) -> ArousalLevel {
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

fn step_attraction(current: AttractionLevel, delta: i8) -> AttractionLevel {
    const LEVELS: [AttractionLevel; 4] = [
        AttractionLevel::Unattracted,
        AttractionLevel::Ok,
        AttractionLevel::Attracted,
        AttractionLevel::Lust,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 3) as usize]
}

fn step_alcohol(current: AlcoholLevel, delta: i8) -> AlcoholLevel {
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

fn parse_relationship_status(s: &str) -> Option<RelationshipStatus> {
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

fn parse_behaviour(s: &str) -> Option<Behaviour> {
    match s {
        "Neutral" => Some(Behaviour::Neutral),
        "Romantic" => Some(Behaviour::Romantic),
        "Mean" => Some(Behaviour::Mean),
        "Cold" => Some(Behaviour::Cold),
        "Faking" => Some(Behaviour::Faking),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Main apply_effect
// ---------------------------------------------------------------------------

pub fn apply_effect(
    effect: &EffectDef,
    world: &mut World,
    ctx: &mut SceneCtx,
    registry: &PackRegistry,
) -> Result<(), EffectError> {
    match effect {
        EffectDef::ChangeStress { amount } => {
            world.player.stress += amount;
        }
        EffectDef::ChangeMoney { amount } => {
            world.player.money += amount;
        }
        EffectDef::ChangeAnxiety { amount } => {
            world.player.anxiety += amount;
        }
        EffectDef::AddArousal { delta } => {
            world.player.arousal = step_arousal(world.player.arousal, *delta);
        }
        EffectDef::SetSceneFlag { flag } => {
            ctx.set_flag(flag.clone());
        }
        EffectDef::RemoveSceneFlag { flag } => {
            ctx.scene_flags.remove(flag.as_str());
        }
        EffectDef::SetGameFlag { flag } => {
            world.game_data.set_flag(flag.clone());
        }
        EffectDef::RemoveGameFlag { flag } => {
            world.game_data.remove_flag(flag.as_str());
        }
        EffectDef::AddStat { stat, amount } => {
            let sid = registry
                .get_stat(stat)
                .ok_or_else(|| EffectError::UnknownStat(stat.clone()))?;
            world.game_data.add_stat(sid, *amount);
        }
        EffectDef::SetStat { stat, value } => {
            let sid = registry
                .get_stat(stat)
                .ok_or_else(|| EffectError::UnknownStat(stat.clone()))?;
            world.game_data.set_stat(sid, *value);
        }
        EffectDef::AddTrait { trait_id } => {
            let tid = registry
                .resolve_trait(trait_id)
                .map_err(|_| EffectError::UnknownTrait(trait_id.clone()))?;
            if let Some(conflict_msg) = registry.check_trait_conflict(&world.player.traits, tid) {
                eprintln!("[effect] trait conflict: {}", conflict_msg);
                // Skip the effect — don't add the conflicting trait
            } else {
                world.player.traits.insert(tid);
            }
        }
        EffectDef::RemoveTrait { trait_id } => {
            let tid = registry
                .resolve_trait(trait_id)
                .map_err(|_| EffectError::UnknownTrait(trait_id.clone()))?;
            world.player.traits.remove(&tid);
        }
        EffectDef::SkillIncrease { skill, amount } => {
            let sid = registry
                .resolve_skill(skill)
                .map_err(|_| EffectError::UnknownSkill(skill.clone()))?;
            let entry = world.player.skills.entry(sid).or_insert(SkillValue {
                value: 0,
                modifier: 0,
            });
            entry.value += amount;
        }
        EffectDef::AddNpcLiking { npc, delta } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.pc_liking = step_liking(npc_data.core.pc_liking, *delta);
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.pc_liking = step_liking(npc_data.core.pc_liking, *delta);
            }
        },
        EffectDef::AddNpcLove { npc, delta } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.npc_love = step_love(npc_data.core.npc_love, *delta);
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.npc_love = step_love(npc_data.core.npc_love, *delta);
            }
        },
        EffectDef::AddWLiking { npc, delta } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.npc_liking = step_liking(npc_data.core.npc_liking, *delta);
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.npc_liking = step_liking(npc_data.core.npc_liking, *delta);
            }
        },
        EffectDef::SetNpcFlag { npc, flag } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.relationship_flags.insert(flag.clone());
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.relationship_flags.insert(flag.clone());
            }
        },
        EffectDef::AddNpcTrait { npc, trait_id } => {
            let tid = registry
                .resolve_npc_trait(trait_id)
                .map_err(|_| EffectError::UnknownNpcTrait(trait_id.clone()))?;
            match resolve_npc_ref(npc, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.traits.insert(tid);
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.traits.insert(tid);
                }
            }
        }
        EffectDef::AddStuff { item } => {
            let stuff_id = registry
                .resolve_stuff(item)
                .ok_or_else(|| EffectError::UnknownStuff(item.clone()))?;
            world.player.stuff.insert(stuff_id);
        }
        EffectDef::RemoveStuff { item } => {
            let stuff_id = registry
                .resolve_stuff(item)
                .ok_or_else(|| EffectError::UnknownStuff(item.clone()))?;
            world.player.stuff.remove(&stuff_id);
        }
        EffectDef::SetRelationship { npc, status } => {
            let parsed = parse_relationship_status(status)
                .ok_or_else(|| EffectError::UnknownRelationshipStatus(status.clone()))?;
            match resolve_npc_ref(npc, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.relationship = parsed;
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.relationship = parsed;
                }
            }
        }
        EffectDef::SetNpcAttraction { npc, delta } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.npc_attraction =
                    step_attraction(npc_data.core.npc_attraction, *delta);
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.npc_attraction =
                    step_attraction(npc_data.core.npc_attraction, *delta);
            }
        },
        EffectDef::SetNpcBehaviour { npc, behaviour } => {
            let parsed = parse_behaviour(behaviour)
                .ok_or_else(|| EffectError::UnknownBehaviour(behaviour.clone()))?;
            match resolve_npc_ref(npc, ctx)? {
                NpcRef::Male(key) => {
                    let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.behaviour = parsed;
                }
                NpcRef::Female(key) => {
                    let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                    npc_data.core.behaviour = parsed;
                }
            }
        }
        EffectDef::SetContactable { npc, value } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.contactable = *value;
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.contactable = *value;
            }
        },
        EffectDef::AddSexualActivity { npc, activity } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.sexual_activities.insert(activity.clone());
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.sexual_activities.insert(activity.clone());
            }
        },
        EffectDef::SetPlayerPartner { npc } => {
            let npc_key = match resolve_npc_ref(npc, ctx)? {
                NpcRef::Male(key) => NpcKey::Male(key),
                NpcRef::Female(key) => NpcKey::Female(key),
            };
            world.player.partner = Some(npc_key);
        }
        EffectDef::AddPlayerFriend { npc } => {
            let npc_key = match resolve_npc_ref(npc, ctx)? {
                NpcRef::Male(key) => NpcKey::Male(key),
                NpcRef::Female(key) => NpcKey::Female(key),
            };
            if !world.player.friends.contains(&npc_key) {
                world.player.friends.push(npc_key);
            }
        }
        EffectDef::SetJobTitle { title } => {
            world.game_data.job_title = title.clone();
        }
        EffectDef::ChangeAlcohol { delta } => {
            world.player.alcohol = step_alcohol(world.player.alcohol, *delta);
        }
        EffectDef::SetVirgin { value, virgin_type } => match virgin_type.as_deref() {
            None | Some("vaginal") => {
                world.player.virgin = *value;
            }
            Some("anal") => {
                world.player.anal_virgin = *value;
            }
            Some("lesbian") => {
                world.player.lesbian_virgin = *value;
            }
            Some(other) => {
                return Err(EffectError::UnknownVirginType(format!(
                    "set_virgin: unknown virgin_type '{}'",
                    other
                )));
            }
        },
        EffectDef::AdvanceTime { slots } => {
            for _ in 0..*slots {
                world.game_data.advance_time_slot();
            }
        }
        EffectDef::AdvanceArc { arc, to_state } => {
            world.game_data.advance_arc(arc, to_state);
        }
        EffectDef::SetNpcRole { npc, role } => match resolve_npc_ref(npc, ctx)? {
            NpcRef::Male(key) => {
                let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.roles.insert(role.clone());
            }
            NpcRef::Female(key) => {
                let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
                npc_data.core.roles.insert(role.clone());
            }
        },
        EffectDef::FailRedCheck { skill } => {
            let scene_id = ctx.scene_id.as_deref().unwrap_or("unknown");
            world.game_data.fail_red_check(scene_id, skill);
        }
        EffectDef::Transition { .. } => {
            // Dead code path. EffectDef::Transition exists so that `type =
            // "transition"` in TOML deserialises without error.  Scene
            // transitions are actually driven by NextBranch.goto in the engine
            // (evaluate_next), not by apply_effect.  If a pack author
            // mistakenly places a transition inside [[actions.effects]] it is
            // silently ignored here rather than crashing.
        }
    }
    Ok(())
}

enum NpcRef {
    Male(MaleNpcKey),
    Female(FemaleNpcKey),
}

fn resolve_npc_ref(npc: &str, ctx: &SceneCtx) -> Result<NpcRef, EffectError> {
    match npc {
        "m" => ctx
            .active_male
            .map(NpcRef::Male)
            .ok_or(EffectError::NoActiveMale),
        "f" => ctx
            .active_female
            .map(NpcRef::Female)
            .ok_or(EffectError::NoActiveFemale),
        _ => Err(EffectError::BadNpcRef(npc.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    use lasso::Key;
    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};

    fn make_female_npc() -> FemaleNpc {
        FemaleNpc {
            core: NpcCore {
                name: "Fiona".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "green".into(),
                hair_colour: "red".into(),
                personality: PersonalityId(lasso::Spur::try_from_usize(0).unwrap()),
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
            char_type: CharTypeId(lasso::Spur::try_from_usize(0).unwrap()),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::Average,
            clothing: FemaleClothing::default(),
            pregnancy: None,
            virgin: true,
        }
    }

    fn make_world() -> World {
        World {
            player: Player {
                name_fem: "Eva".into(),
                name_androg: "Ev".into(),
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
                breasts: BreastSize::Big,
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
                money: 100,
                stress: 10,
                anxiety: 5,
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

    fn make_male_npc() -> MaleNpc {
        MaleNpc {
            core: NpcCore {
                name: "Test".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "blue".into(),
                hair_colour: "brown".into(),
                personality: PersonalityId(lasso::Spur::try_from_usize(0).unwrap()),
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

    #[test]
    fn change_stress_adds_amount() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::ChangeStress { amount: 5 },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.player.stress, 15);
    }

    #[test]
    fn change_money_subtracts() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::ChangeMoney { amount: -30 },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.player.money, 70);
    }

    #[test]
    fn set_scene_flag_adds_flag() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::SetSceneFlag {
                flag: "test_flag".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(ctx.has_flag("test_flag"));
    }

    #[test]
    fn set_game_flag_adds_to_game_data() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::SetGameFlag {
                flag: "GLOBAL".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(world.game_data.has_flag("GLOBAL"));
    }

    #[test]
    fn add_npc_liking_steps_up_clamped() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::AddNpcLiking {
                npc: "m".into(),
                delta: 2,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.male_npcs[key].core.pc_liking, LikingLevel::Like);
    }

    #[test]
    fn add_npc_liking_clamps_at_max() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::AddNpcLiking {
                npc: "m".into(),
                delta: 99,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.male_npcs[key].core.pc_liking, LikingLevel::Close);
    }

    #[test]
    fn add_npc_liking_works_for_female() {
        let mut world = make_world();
        let key = world.female_npcs.insert(make_female_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_female = Some(key);
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::AddNpcLiking {
                npc: "f".into(),
                delta: 1,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.female_npcs[key].core.pc_liking, LikingLevel::Ok);
    }

    #[test]
    fn add_npc_love_works_for_female() {
        let mut world = make_world();
        let key = world.female_npcs.insert(make_female_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_female = Some(key);
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::AddNpcLove {
                npc: "f".into(),
                delta: 2,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.female_npcs[key].core.npc_love, LoveLevel::Confused);
    }

    #[test]
    fn set_npc_flag_works_for_female() {
        let mut world = make_world();
        let key = world.female_npcs.insert(make_female_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_female = Some(key);
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::SetNpcFlag {
                npc: "f".into(),
                flag: "kissed".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(world.female_npcs[key]
            .core
            .relationship_flags
            .contains("kissed"));
    }

    #[test]
    fn add_stat_unknown_returns_error() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new(); // empty registry — no stats registered
        let result = apply_effect(
            &EffectDef::AddStat {
                stat: "NONEXISTENT_STAT".into(),
                amount: 1,
            },
            &mut world,
            &mut ctx,
            &reg,
        );
        assert!(result.is_err(), "expected error for unknown stat");
        assert!(matches!(result, Err(EffectError::UnknownStat(_))));
    }

    #[test]
    fn set_stat_unknown_returns_error() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        let result = apply_effect(
            &EffectDef::SetStat {
                stat: "NONEXISTENT_STAT".into(),
                value: 5,
            },
            &mut world,
            &mut ctx,
            &reg,
        );
        assert!(result.is_err(), "expected error for unknown stat");
        assert!(matches!(result, Err(EffectError::UnknownStat(_))));
    }

    #[test]
    fn add_w_liking_works_for_female() {
        let mut world = make_world();
        let key = world.female_npcs.insert(make_female_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_female = Some(key);
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::AddWLiking {
                npc: "f".into(),
                delta: 1,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.female_npcs[key].core.npc_liking, LikingLevel::Ok);
    }

    #[test]
    fn add_stuff_works() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let mut reg = PackRegistry::new();
        // intern the item so resolve_stuff can find it
        reg.intern_stuff("CONDOMS");
        apply_effect(
            &EffectDef::AddStuff {
                item: "CONDOMS".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        let stuff_id = reg.resolve_stuff("CONDOMS").unwrap();
        assert!(world.player.stuff.contains(&stuff_id));
    }

    #[test]
    fn remove_stuff_works() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let mut reg = PackRegistry::new();
        reg.intern_stuff("CONDOMS");
        // add it first
        apply_effect(
            &EffectDef::AddStuff {
                item: "CONDOMS".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        let stuff_id = reg.resolve_stuff("CONDOMS").unwrap();
        assert!(world.player.stuff.contains(&stuff_id));
        // now remove it
        apply_effect(
            &EffectDef::RemoveStuff {
                item: "CONDOMS".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(!world.player.stuff.contains(&stuff_id));
    }

    #[test]
    fn set_job_title_works() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(
            &EffectDef::SetJobTitle {
                title: "Barista".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.game_data.job_title, "Barista");
    }

    #[test]
    fn change_alcohol_steps_up() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        // world starts Sober, delta 1 → Tipsy
        apply_effect(
            &EffectDef::ChangeAlcohol { delta: 1 },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.player.alcohol, AlcoholLevel::Tipsy);
    }

    #[test]
    fn set_virgin_works() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        assert!(world.player.virgin);
        apply_effect(
            &EffectDef::SetVirgin {
                value: false,
                virgin_type: None,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(!world.player.virgin);
    }

    #[test]
    fn advance_time_works() {
        use undone_domain::TimeSlot;
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        assert_eq!(world.game_data.time_slot, TimeSlot::Morning);
        apply_effect(
            &EffectDef::AdvanceTime { slots: 1 },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.game_data.time_slot, TimeSlot::Afternoon);
    }

    #[test]
    fn set_npc_attraction_works() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        // starts Unattracted, delta 1 → Ok
        apply_effect(
            &EffectDef::SetNpcAttraction {
                npc: "m".into(),
                delta: 1,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(
            world.male_npcs[key].core.npc_attraction,
            AttractionLevel::Ok
        );
    }

    #[test]
    fn set_contactable_works() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        assert!(world.male_npcs[key].core.contactable);
        apply_effect(
            &EffectDef::SetContactable {
                npc: "m".into(),
                value: false,
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(!world.male_npcs[key].core.contactable);
    }

    #[test]
    fn set_relationship_works() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        assert_eq!(
            world.male_npcs[key].core.relationship,
            RelationshipStatus::Stranger
        );
        apply_effect(
            &EffectDef::SetRelationship {
                npc: "m".into(),
                status: "Friend".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(
            world.male_npcs[key].core.relationship,
            RelationshipStatus::Friend
        );
    }

    #[test]
    fn set_npc_behaviour_works() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        assert_eq!(world.male_npcs[key].core.behaviour, Behaviour::Neutral);
        apply_effect(
            &EffectDef::SetNpcBehaviour {
                npc: "m".into(),
                behaviour: "Romantic".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.male_npcs[key].core.behaviour, Behaviour::Romantic);
    }

    #[test]
    fn add_sexual_activity_works() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        assert!(world.male_npcs[key].core.sexual_activities.is_empty());
        apply_effect(
            &EffectDef::AddSexualActivity {
                npc: "m".into(),
                activity: "kissed".into(),
            },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(world.male_npcs[key]
            .core
            .sexual_activities
            .contains("kissed"));
    }

    #[test]
    fn set_player_partner_works() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        assert!(world.player.partner.is_none());
        apply_effect(
            &EffectDef::SetPlayerPartner { npc: "m".into() },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert_eq!(world.player.partner, Some(NpcKey::Male(key)));
    }

    #[test]
    fn add_player_friend_works() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        assert!(world.player.friends.is_empty());
        apply_effect(
            &EffectDef::AddPlayerFriend { npc: "m".into() },
            &mut world,
            &mut ctx,
            &reg,
        )
        .unwrap();
        assert!(world.player.friends.contains(&NpcKey::Male(key)));
    }

    #[test]
    fn advance_arc_effect_changes_state() {
        let mut world = make_world();
        let registry = PackRegistry::new();
        let mut ctx = SceneCtx::new();
        ctx.scene_id = Some("test::scene".into());

        let effect = EffectDef::AdvanceArc {
            arc: "base::workplace_opening".into(),
            to_state: "week_one".into(),
        };
        apply_effect(&effect, &mut world, &mut ctx, &registry).unwrap();
        assert_eq!(
            world.game_data.arc_state("base::workplace_opening"),
            Some("week_one")
        );
    }

    #[test]
    fn set_npc_role_adds_role_to_active_female() {
        let mut world = make_world();
        let npc = make_female_npc();
        let key = world.female_npcs.insert(npc);
        let mut ctx = SceneCtx::new();
        ctx.active_female = Some(key);
        ctx.scene_id = Some("test::scene".into());
        let registry = PackRegistry::new();

        let effect = EffectDef::SetNpcRole {
            npc: "f".into(),
            role: "ROLE_LANDLORD".into(),
        };
        apply_effect(&effect, &mut world, &mut ctx, &registry).unwrap();
        let npc_data = world.female_npcs.get(key).unwrap();
        assert!(npc_data.core.roles.contains("ROLE_LANDLORD"));
    }

    #[test]
    fn fail_red_check_effect_records_failure() {
        let mut world = make_world();
        let registry = PackRegistry::new();
        let mut ctx = SceneCtx::new();
        ctx.scene_id = Some("base::rain_shelter".into());

        let effect = EffectDef::FailRedCheck {
            skill: "CHARM".into(),
        };
        apply_effect(&effect, &mut world, &mut ctx, &registry).unwrap();
        assert!(world
            .game_data
            .has_failed_red_check("base::rain_shelter", "CHARM"));
    }
}
