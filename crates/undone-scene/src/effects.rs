use thiserror::Error;
use undone_domain::{ArousalLevel, FemaleNpcKey, LikingLevel, LoveLevel, MaleNpcKey, SkillValue};
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
            if let Some(sid) = registry.get_stat(stat) {
                world.game_data.add_stat(sid, *amount);
            }
        }
        EffectDef::SetStat { stat, value } => {
            if let Some(sid) = registry.get_stat(stat) {
                world.game_data.set_stat(sid, *value);
            }
        }
        EffectDef::AddTrait { trait_id } => {
            let tid = registry
                .resolve_trait(trait_id)
                .map_err(|_| EffectError::UnknownTrait(trait_id.clone()))?;
            world.player.traits.insert(tid);
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
                age: Age::Twenties,
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
            },
            char_type: CharTypeId(lasso::Spur::try_from_usize(0).unwrap()),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::MediumSmall,
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
                before_age: 30,
                before_race: "white".into(),
                before_sexuality: Some(BeforeSexuality::AttractedToWomen),
                age: Age::LateTeen,
                race: "east_asian".into(),
                figure: PlayerFigure::Slim,
                breasts: BreastSize::Large,
                eye_colour: "brown".into(),
                hair_colour: "dark".into(),
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
                age: Age::Twenties,
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
}
