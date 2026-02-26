use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use rand::Rng;
use thiserror::Error;
use undone_domain::{FemaleNpcKey, MaleNpcKey, PcOrigin};
use undone_packs::{CategoryType, PackRegistry};
use undone_world::World;

use crate::parser::{Call, Expr, Receiver, Value};

/// Per-scene mutable state passed to the evaluator.
/// Lives only for the duration of a scene run.
pub struct SceneCtx {
    pub active_male: Option<MaleNpcKey>,
    pub active_female: Option<FemaleNpcKey>,
    pub scene_flags: HashSet<String>,
    pub weighted_map: HashMap<String, i32>,
    /// Cached percentile rolls (1–100) keyed by skill_id string.
    /// Interior mutability so eval() can cache without needing &mut SceneCtx.
    pub skill_rolls: RefCell<HashMap<String, i32>>,
    /// Scene ID set by the engine before evaluating conditions.
    /// Required for red-check failure tracking.
    pub scene_id: Option<String>,
}

impl SceneCtx {
    pub fn new() -> Self {
        Self {
            active_male: None,
            active_female: None,
            scene_flags: HashSet::new(),
            weighted_map: HashMap::new(),
            skill_rolls: RefCell::new(HashMap::new()),
            scene_id: None,
        }
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.scene_flags.contains(flag)
    }

    pub fn set_flag(&mut self, flag: impl Into<String>) {
        self.scene_flags.insert(flag.into());
    }

    /// Force a specific roll value for testing. Call before evaluating checkSkill.
    pub fn set_skill_roll(&self, skill_id: &str, roll: i32) {
        self.skill_rolls
            .borrow_mut()
            .insert(skill_id.to_string(), roll);
    }

    /// Return a cached roll for this skill, or generate and cache a new one (1–100).
    pub fn get_or_roll_skill(&self, skill_id: &str) -> i32 {
        let mut rolls = self.skill_rolls.borrow_mut();
        *rolls
            .entry(skill_id.to_string())
            .or_insert_with(|| rand::thread_rng().gen_range(1_i32..=100))
    }
}

impl Default for SceneCtx {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("no active male NPC in scene context")]
    NoActiveMaleNpc,
    #[error("no active female NPC in scene context")]
    NoActiveFemaleNpc,
    #[error("unknown method '{receiver}.{method}'")]
    UnknownMethod { receiver: String, method: String },
    #[error("wrong argument type for '{0}'")]
    BadArg(String),
    #[error("NPC key not found in world")]
    NpcNotFound,
    #[error("unknown trait '{0}'")]
    UnknownTrait(String),
    #[error("unknown npc trait '{0}'")]
    UnknownNpcTrait(String),
    #[error("unknown skill '{0}'")]
    UnknownSkill(String),
}

/// Evaluate a parsed expression to bool.
pub fn eval(
    expr: &Expr,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<bool, EvalError> {
    match expr {
        Expr::Lit(Value::Bool(b)) => Ok(*b),
        Expr::Lit(_) => Ok(true), // non-bool literals as conditions are truthy

        Expr::Not(inner) => Ok(!eval(inner, world, ctx, registry)?),

        Expr::And(l, r) => Ok(eval(l, world, ctx, registry)? && eval(r, world, ctx, registry)?),
        Expr::Or(l, r) => Ok(eval(l, world, ctx, registry)? || eval(r, world, ctx, registry)?),

        Expr::Eq(l, r) => {
            let lv = eval_to_value(l, world, ctx, registry)?;
            let rv = eval_to_value(r, world, ctx, registry)?;
            Ok(lv == rv)
        }
        Expr::Ne(l, r) => {
            let lv = eval_to_value(l, world, ctx, registry)?;
            let rv = eval_to_value(r, world, ctx, registry)?;
            Ok(lv != rv)
        }
        Expr::Lt(l, r) => int_compare(l, r, world, ctx, registry, |a, b| a < b),
        Expr::Gt(l, r) => int_compare(l, r, world, ctx, registry, |a, b| a > b),
        Expr::Le(l, r) => int_compare(l, r, world, ctx, registry, |a, b| a <= b),
        Expr::Ge(l, r) => int_compare(l, r, world, ctx, registry, |a, b| a >= b),

        Expr::Call(call) => eval_call_bool(call, world, ctx, registry),
    }
}

fn int_compare(
    l: &Expr,
    r: &Expr,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
    cmp: impl Fn(i64, i64) -> bool,
) -> Result<bool, EvalError> {
    let lv = eval_to_int(l, world, ctx, registry)?;
    let rv = eval_to_int(r, world, ctx, registry)?;
    Ok(cmp(lv, rv))
}

/// Evaluate an expression to a generic Value for comparison.
fn eval_to_value(
    expr: &Expr,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<EvalValue, EvalError> {
    match expr {
        Expr::Lit(v) => Ok(match v {
            Value::Str(s) => EvalValue::Str(s.clone()),
            Value::Int(n) => EvalValue::Int(*n),
            Value::Bool(b) => EvalValue::Bool(*b),
        }),
        Expr::Call(call) => {
            // Try to eval as int, then string, then bool
            if let Ok(n) = eval_call_int(call, world, ctx, registry) {
                return Ok(EvalValue::Int(n));
            }
            if let Ok(s) = eval_call_string(call, world, ctx, registry) {
                return Ok(EvalValue::Str(s));
            }
            let b = eval_call_bool(call, world, ctx, registry)?;
            Ok(EvalValue::Bool(b))
        }
        other => {
            let b = eval(other, world, ctx, registry)?;
            Ok(EvalValue::Bool(b))
        }
    }
}

fn eval_to_int(
    expr: &Expr,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<i64, EvalError> {
    match expr {
        Expr::Lit(Value::Int(n)) => Ok(*n),
        Expr::Call(call) => eval_call_int(call, world, ctx, registry),
        _ => Err(EvalError::BadArg("expected integer".into())),
    }
}

#[derive(Debug, PartialEq)]
enum EvalValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

/// Evaluate a method call that returns bool.
pub fn eval_call_bool(
    call: &Call,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<bool, EvalError> {
    let str_arg = |i: usize| -> Result<&str, EvalError> {
        match call.args.get(i) {
            Some(Value::Str(s)) => Ok(s.as_str()),
            _ => Err(EvalError::BadArg(call.method.clone())),
        }
    };

    match call.receiver {
        Receiver::Player => match call.method.as_str() {
            "hasTrait" => {
                let id = str_arg(0)?;
                let trait_id = registry
                    .resolve_trait(id)
                    .map_err(|_| EvalError::UnknownTrait(id.to_string()))?;
                Ok(world.player.has_trait(trait_id))
            }
            "isVirgin" => Ok(world.player.virgin),
            "isAnalVirgin" => Ok(world.player.anal_virgin),
            "isDrunk" => Ok(world.player.is_drunk()),
            "isVeryDrunk" => Ok(world.player.is_very_drunk()),
            "isMaxDrunk" => Ok(world.player.is_max_drunk()),
            "isSingle" => Ok(world.player.partner.is_none()),
            "isOnPill" => Ok(world.player.on_pill),
            "isPregnant" => Ok(world.player.pregnancy.is_some()),
            "alwaysFemale" => Ok(world.player.origin.is_always_female()),
            "hasStuff" => {
                let id = str_arg(0)?;
                match registry.resolve_stuff(id) {
                    Some(stuff_id) => Ok(world.player.stuff.contains(&stuff_id)),
                    None => Ok(false), // never interned = player can't have it
                }
            }
            "wasMale" => Ok(world.player.origin.was_male_bodied()),
            "wasTransformed" => Ok(world.player.origin.was_transformed()),
            "hasSmoothLegs" => {
                let has_naturally_smooth = registry
                    .resolve_trait("NATURALLY_SMOOTH")
                    .ok()
                    .map(|id| world.player.has_trait(id))
                    .unwrap_or(false);
                let has_smooth_legs = registry
                    .resolve_trait("SMOOTH_LEGS")
                    .ok()
                    .map(|id| world.player.has_trait(id))
                    .unwrap_or(false);
                Ok(has_naturally_smooth || has_smooth_legs)
            }
            "checkSkill" => {
                let skill_id_str = str_arg(0)?;
                let dc = match call.args.get(1) {
                    Some(Value::Int(n)) => *n as i32,
                    _ => return Err(EvalError::BadArg("checkSkill".into())),
                };
                let skill_id = registry
                    .resolve_skill(skill_id_str)
                    .map_err(|_| EvalError::UnknownSkill(skill_id_str.to_string()))?;
                let skill_value = world.player.skill(skill_id);
                let roll = ctx.get_or_roll_skill(skill_id_str);
                let target = (skill_value + (50 - dc)).clamp(5, 95);
                Ok(roll <= target)
            }
            "checkSkillRed" => {
                let skill_id_str = str_arg(0)?;
                let dc = match call.args.get(1) {
                    Some(Value::Int(n)) => *n as i32,
                    _ => return Err(EvalError::BadArg("checkSkillRed".into())),
                };
                // If already permanently failed, block immediately.
                let scene_id = ctx.scene_id.as_deref().unwrap_or("unknown");
                if world.game_data.has_failed_red_check(scene_id, skill_id_str) {
                    return Ok(false);
                }
                let skill_id = registry
                    .resolve_skill(skill_id_str)
                    .map_err(|_| EvalError::UnknownSkill(skill_id_str.to_string()))?;
                let skill_value = world.player.skill(skill_id);
                let roll = ctx.get_or_roll_skill(skill_id_str);
                let target = (skill_value + (50 - dc)).clamp(5, 95);
                Ok(roll <= target)
                // NOTE: Marking failure is an Effect (FailRedCheck), not done here.
            }
            "hadTraitBefore" => {
                let id = str_arg(0)?;
                match &world.player.before {
                    None => Ok(false),
                    Some(before) => {
                        let trait_id = registry
                            .resolve_trait(id)
                            .map_err(|_| EvalError::UnknownTrait(id.to_string()))?;
                        Ok(before.traits.contains(&trait_id))
                    }
                }
            }
            "inCategory" => {
                let cat = str_arg(0)?;
                match registry.get_category(cat) {
                    None => Ok(false),
                    Some(cat_def) => match cat_def.category_type {
                        CategoryType::Age => {
                            let value = format!("{:?}", world.player.age);
                            Ok(cat_def.members.iter().any(|m| m == &value))
                        }
                        CategoryType::Race => {
                            Ok(cat_def.members.iter().any(|m| m == &world.player.race))
                        }
                        CategoryType::Trait => Ok(cat_def.members.iter().any(|m| {
                            registry
                                .resolve_trait(m)
                                .map(|id| world.player.traits.contains(&id))
                                .unwrap_or(false)
                        })),
                        CategoryType::Personality => Ok(false),
                    },
                }
            }
            "beforeInCategory" => {
                let cat = str_arg(0)?;
                match &world.player.before {
                    None => Ok(false),
                    Some(before) => match registry.get_category(cat) {
                        None => Ok(false),
                        Some(cat_def) => match cat_def.category_type {
                            CategoryType::Age => {
                                let value = format!("{:?}", before.age);
                                Ok(cat_def.members.iter().any(|m| m == &value))
                            }
                            CategoryType::Race => {
                                Ok(cat_def.members.iter().any(|m| m == &before.race))
                            }
                            CategoryType::Trait => Ok(cat_def.members.iter().any(|m| {
                                registry
                                    .resolve_trait(m)
                                    .map(|id| before.traits.contains(&id))
                                    .unwrap_or(false)
                            })),
                            CategoryType::Personality => Ok(false),
                        },
                    },
                }
            }
            _ => Err(EvalError::UnknownMethod {
                receiver: "w".into(),
                method: call.method.clone(),
            }),
        },

        Receiver::MaleNpc => {
            let key = ctx.active_male.ok_or(EvalError::NoActiveMaleNpc)?;
            let npc = world.male_npc(key).ok_or(EvalError::NpcNotFound)?;
            match call.method.as_str() {
                "isPartner" => Ok(npc.core.is_partner()),
                "isFriend" => Ok(npc.core.is_friend()),
                "isCohabiting" => Ok(npc.core.is_cohabiting()),
                "isContactable" => Ok(npc.core.contactable),
                "hadOrgasm" => Ok(npc.had_orgasm),
                "hasTrait" => {
                    let id = str_arg(0)?;
                    let trait_id = registry
                        .resolve_npc_trait(id)
                        .map_err(|_| EvalError::UnknownNpcTrait(id.to_string()))?;
                    Ok(npc.core.has_trait(trait_id))
                }
                "isNpcAttractionOk" => {
                    Ok(npc.core.npc_attraction >= undone_domain::AttractionLevel::Ok)
                }
                "isNpcAttractionLust" => {
                    Ok(npc.core.npc_attraction == undone_domain::AttractionLevel::Lust)
                }
                "isWAttractionOk" => {
                    Ok(npc.core.pc_attraction >= undone_domain::AttractionLevel::Ok)
                }
                "isNpcLoveCrush" => Ok(npc.core.npc_love >= undone_domain::LoveLevel::Crush),
                "isNpcLoveSome" => Ok(npc.core.npc_love >= undone_domain::LoveLevel::Some),
                "isWLoveCrush" => Ok(npc.core.pc_love >= undone_domain::LoveLevel::Crush),
                "hasFlag" => {
                    let flag = str_arg(0)?;
                    Ok(npc.core.relationship_flags.contains(flag))
                }
                "hasRole" => {
                    let role = str_arg(0)?;
                    Ok(npc.core.roles.contains(role))
                }
                _ => Err(EvalError::UnknownMethod {
                    receiver: "m".into(),
                    method: call.method.clone(),
                }),
            }
        }

        Receiver::Scene => match call.method.as_str() {
            "hasFlag" => {
                let flag = str_arg(0)?;
                Ok(ctx.has_flag(flag))
            }
            _ => Err(EvalError::UnknownMethod {
                receiver: "scene".into(),
                method: call.method.clone(),
            }),
        },

        Receiver::GameData => match call.method.as_str() {
            "hasGameFlag" => {
                let flag = str_arg(0)?;
                Ok(world.game_data.has_flag(flag))
            }
            "isWeekday" => Ok(world.game_data.is_weekday()),
            "isWeekend" => Ok(world.game_data.is_weekend()),
            "arcStarted" => {
                let arc_id = str_arg(0)?;
                Ok(world.game_data.arc_state(arc_id).is_some())
            }
            _ => Err(EvalError::UnknownMethod {
                receiver: "gd".into(),
                method: call.method.clone(),
            }),
        },

        Receiver::FemaleNpc => {
            let key = ctx.active_female.ok_or(EvalError::NoActiveFemaleNpc)?;
            let npc = world.female_npc(key).ok_or(EvalError::NpcNotFound)?;
            match call.method.as_str() {
                "isPartner" => Ok(npc.core.is_partner()),
                "isFriend" => Ok(npc.core.is_friend()),
                "isPregnant" => Ok(npc.pregnancy.is_some()),
                "isVirgin" => Ok(npc.virgin),
                "hasFlag" => {
                    let flag = str_arg(0)?;
                    Ok(npc.core.relationship_flags.contains(flag))
                }
                "hasRole" => {
                    let role = str_arg(0)?;
                    Ok(npc.core.roles.contains(role))
                }
                _ => Err(EvalError::UnknownMethod {
                    receiver: "f".into(),
                    method: call.method.clone(),
                }),
            }
        }
    }
}

/// Evaluate a method call that returns an integer (e.g. getSkill, getStat, week).
pub fn eval_call_int(
    call: &Call,
    world: &World,
    _ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<i64, EvalError> {
    let str_arg = |i: usize| -> Result<&str, EvalError> {
        match call.args.get(i) {
            Some(Value::Str(s)) => Ok(s.as_str()),
            _ => Err(EvalError::BadArg(call.method.clone())),
        }
    };

    match call.receiver {
        Receiver::Player => match call.method.as_str() {
            "getMoney" => Ok(world.player.money as i64),
            "getStress" => Ok(world.player.stress as i64),
            "getAnxiety" => Ok(world.player.anxiety as i64),
            "getSkill" => {
                let id = str_arg(0)?;
                let skill_id = registry
                    .resolve_skill(id)
                    .map_err(|_| EvalError::UnknownSkill(id.to_string()))?;
                Ok(world.player.skill(skill_id) as i64)
            }
            _ => Err(EvalError::UnknownMethod {
                receiver: "w".into(),
                method: call.method.clone(),
            }),
        },
        Receiver::GameData => match call.method.as_str() {
            "week" => Ok(world.game_data.week as i64),
            "day" => Ok(world.game_data.day as i64),
            "getStat" => {
                let id = str_arg(0)?;
                match registry.get_stat(id) {
                    Some(stat_id) => Ok(world.game_data.get_stat(stat_id) as i64),
                    None => Ok(0), // stat never interned = was never set
                }
            }
            _ => Err(EvalError::UnknownMethod {
                receiver: "gd".into(),
                method: call.method.clone(),
            }),
        },
        _ => Err(EvalError::UnknownMethod {
            receiver: format!("{:?}", call.receiver),
            method: call.method.clone(),
        }),
    }
}

/// Evaluate a method call that returns a string (e.g. pcOrigin).
pub fn eval_call_string(
    call: &Call,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<String, EvalError> {
    let str_arg = |i: usize| -> Result<&str, EvalError> {
        match call.args.get(i) {
            Some(Value::Str(s)) => Ok(s.as_str()),
            _ => Err(EvalError::BadArg(call.method.clone())),
        }
    };
    match call.receiver {
        Receiver::Player => match call.method.as_str() {
            "pcOrigin" => {
                let s = match world.player.origin {
                    PcOrigin::CisMaleTransformed => "CisMaleTransformed",
                    PcOrigin::TransWomanTransformed => "TransWomanTransformed",
                    PcOrigin::CisFemaleTransformed => "CisFemaleTransformed",
                    PcOrigin::AlwaysFemale => "AlwaysFemale",
                };
                Ok(s.to_string())
            }
            "beforeName" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| b.name.clone())
                .unwrap_or_default()),
            "beforeRace" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| b.race.clone())
                .unwrap_or_default()),
            "beforeAge" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.age))
                .unwrap_or_default()),
            "beforeSexuality" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.sexuality))
                .unwrap_or_default()),
            "getName" => {
                let fem_id = registry
                    .resolve_skill("FEMININITY")
                    .map_err(|_| EvalError::UnknownSkill("FEMININITY".into()))?;
                Ok(world.player.active_name(fem_id).to_string())
            }
            "getRace" => Ok(world.player.race.clone()),
            "getAge" => Ok(format!("{:?}", world.player.age)),
            "getArousal" => Ok(format!("{:?}", world.player.arousal)),
            "getAlcohol" => Ok(format!("{:?}", world.player.alcohol)),

            // Physical attributes
            "getHeight" => Ok(format!("{:?}", world.player.height)),
            "getFigure" => Ok(format!("{:?}", world.player.figure)),
            "getBreasts" => Ok(format!("{:?}", world.player.breasts)),
            "getButt" => Ok(format!("{:?}", world.player.butt)),
            "getWaist" => Ok(format!("{:?}", world.player.waist)),
            "getLips" => Ok(format!("{:?}", world.player.lips)),
            "getHairColour" => Ok(format!("{:?}", world.player.hair_colour)),
            "getHairLength" => Ok(format!("{:?}", world.player.hair_length)),
            "getEyeColour" => Ok(format!("{:?}", world.player.eye_colour)),
            "getSkinTone" => Ok(format!("{:?}", world.player.skin_tone)),
            "getComplexion" => Ok(format!("{:?}", world.player.complexion)),
            "getAppearance" => Ok(format!("{:?}", world.player.appearance)),

            // Sexual/intimate attributes
            "getNippleSensitivity" => Ok(format!("{:?}", world.player.nipple_sensitivity)),
            "getClitSensitivity" => Ok(format!("{:?}", world.player.clit_sensitivity)),
            "getPubicHair" => Ok(format!("{:?}", world.player.pubic_hair)),
            "getNaturalPubicHair" => Ok(format!("{:?}", world.player.natural_pubic_hair)),
            "getInnerLabia" => Ok(format!("{:?}", world.player.inner_labia)),
            "getWetness" => Ok(format!("{:?}", world.player.wetness_baseline)),

            // Before-life attributes
            "beforeVoice" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.voice))
                .unwrap_or_default()),
            "beforeHeight" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.height))
                .unwrap_or_default()),
            "beforeHairColour" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.hair_colour))
                .unwrap_or_default()),
            "beforeEyeColour" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.eye_colour))
                .unwrap_or_default()),
            "beforeSkinTone" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.skin_tone))
                .unwrap_or_default()),
            "beforePenisSize" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.penis_size))
                .unwrap_or_default()),
            "beforeFigure" => Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.figure))
                .unwrap_or_default()),

            _ => Err(EvalError::UnknownMethod {
                receiver: "w".into(),
                method: call.method.clone(),
            }),
        },
        Receiver::GameData => match call.method.as_str() {
            "timeSlot" => Ok(format!("{:?}", world.game_data.time_slot)),
            "getJobTitle" => Ok(world.game_data.job_title.clone()),
            "arcState" => {
                let arc_id = str_arg(0)?;
                Ok(world.game_data.arc_state(arc_id).unwrap_or("").to_string())
            }
            "npcLiking" => {
                let role = str_arg(0)?;
                let liking = world
                    .male_npcs
                    .values()
                    .find(|npc| npc.core.roles.contains(role))
                    .map(|npc| npc.core.pc_liking.to_string())
                    .or_else(|| {
                        world
                            .female_npcs
                            .values()
                            .find(|npc| npc.core.roles.contains(role))
                            .map(|npc| npc.core.pc_liking.to_string())
                    })
                    .unwrap_or_else(|| "Neutral".to_string());
                Ok(liking)
            }
            _ => Err(EvalError::UnknownMethod {
                receiver: "gd".into(),
                method: call.method.clone(),
            }),
        },
        Receiver::MaleNpc => {
            let key = ctx.active_male.ok_or(EvalError::NoActiveMaleNpc)?;
            let npc = world.male_npc(key).ok_or(EvalError::NpcNotFound)?;
            match call.method.as_str() {
                "getLiking" => Ok(npc.core.pc_liking.to_string()),
                "getLove" => Ok(format!("{:?}", npc.core.pc_love)),
                "getAttraction" => Ok(npc.core.pc_attraction.to_string()),
                "getBehaviour" => Ok(format!("{:?}", npc.core.behaviour)),
                _ => Err(EvalError::UnknownMethod {
                    receiver: "m".into(),
                    method: call.method.clone(),
                }),
            }
        }
        Receiver::FemaleNpc => {
            let key = ctx.active_female.ok_or(EvalError::NoActiveFemaleNpc)?;
            let npc = world.female_npc(key).ok_or(EvalError::NpcNotFound)?;
            match call.method.as_str() {
                "getLiking" => Ok(npc.core.pc_liking.to_string()),
                "getLove" => Ok(format!("{:?}", npc.core.pc_love)),
                "getAttraction" => Ok(npc.core.pc_attraction.to_string()),
                "getBehaviour" => Ok(format!("{:?}", npc.core.behaviour)),
                _ => Err(EvalError::UnknownMethod {
                    receiver: "f".into(),
                    method: call.method.clone(),
                }),
            }
        }
        _ => Err(EvalError::UnknownMethod {
            receiver: format!("{:?}", call.receiver),
            method: call.method.clone(),
        }),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};

    use super::*;
    use crate::parser::parse;

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
                stress: 10,
                anxiety: 0,
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

    #[test]
    fn eval_bool_literal_true() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("true").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn eval_is_virgin() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.isVirgin()").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn eval_is_not_drunk() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("!w.isDrunk()").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn eval_game_flag_absent() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.hasGameFlag('SOME_FLAG')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn eval_game_flag_present() {
        let mut world = make_world();
        world.game_data.set_flag("SOME_FLAG");
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.hasGameFlag('SOME_FLAG')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn eval_money_comparison() {
        let world = make_world(); // money = 500
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.getMoney() > 100").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn eval_scene_flag() {
        let world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        ctx.set_flag("offered_umbrella");
        let expr = parse("scene.hasFlag('offered_umbrella')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn hasTrait_true_when_player_has_trait() {
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        let shy_id = reg.resolve_trait("SHY").unwrap();
        let mut world = make_world();
        world.player.traits.insert(shy_id);
        let ctx = SceneCtx::new();
        let expr = parse("w.hasTrait('SHY')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn hasTrait_false_when_player_lacks_trait() {
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        let world = make_world();
        let ctx = SceneCtx::new();
        let expr = parse("w.hasTrait('SHY')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn hasStuff_true_when_player_has_item() {
        let mut reg = undone_packs::PackRegistry::new();
        let stuff_id = reg.intern_stuff("UMBRELLA");
        let mut world = make_world();
        world.player.stuff.insert(stuff_id);
        let ctx = SceneCtx::new();
        let expr = parse("w.hasStuff('UMBRELLA')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn hasStuff_false_when_player_lacks_item() {
        let mut reg = undone_packs::PackRegistry::new();
        reg.intern_stuff("UMBRELLA");
        let world = make_world();
        let ctx = SceneCtx::new();
        let expr = parse("w.hasStuff('UMBRELLA')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn pcOrigin_returns_correct_string_for_cis_male() {
        let world = make_world(); // origin = CisMaleTransformed
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.pcOrigin() == 'CisMaleTransformed'").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn pcOrigin_returns_correct_string_for_always_female() {
        let mut world = make_world();
        world.player.origin = PcOrigin::AlwaysFemale;
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.pcOrigin() == 'AlwaysFemale'").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn alwaysFemale_false_for_cis_male_transformed() {
        let world = make_world(); // origin = CisMaleTransformed
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.alwaysFemale()").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn alwaysFemale_true_for_always_female() {
        let mut world = make_world();
        world.player.origin = PcOrigin::AlwaysFemale;
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.alwaysFemale()").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn getSkill_returns_effective_value() {
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_skills(vec![undone_packs::SkillDef {
            id: "FITNESS".into(),
            name: "Fitness".into(),
            description: "...".into(),
            min: 0,
            max: 100,
        }]);
        let skill_id = reg.resolve_skill("FITNESS").unwrap();
        let mut world = make_world();
        world.player.skills.insert(
            skill_id,
            undone_domain::SkillValue {
                value: 60,
                modifier: -10,
            },
        );
        let ctx = SceneCtx::new();
        let expr = parse("w.getSkill('FITNESS') > 40").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    // ── Skill roll cache tests ─────────────────────────────────────────────────

    #[test]
    fn set_and_get_skill_roll_returns_same_value() {
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 42);
        assert_eq!(ctx.get_or_roll_skill("CHARM"), 42);
    }

    #[test]
    fn get_or_roll_is_idempotent_without_set() {
        let ctx = SceneCtx::new();
        let first = ctx.get_or_roll_skill("FITNESS");
        let second = ctx.get_or_roll_skill("FITNESS");
        assert_eq!(first, second);
    }

    #[test]
    fn different_skills_get_independent_rolls() {
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 30);
        ctx.set_skill_roll("FITNESS", 80);
        assert_eq!(ctx.get_or_roll_skill("CHARM"), 30);
        assert_eq!(ctx.get_or_roll_skill("FITNESS"), 80);
    }

    // ── checkSkill tests ───────────────────────────────────────────────────────

    fn make_registry_with_charm() -> undone_packs::PackRegistry {
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_skills(vec![undone_packs::SkillDef {
            id: "CHARM".into(),
            name: "Charm".into(),
            description: "".into(),
            min: 0,
            max: 100,
        }]);
        reg
    }

    #[test]
    fn checkSkill_succeeds_when_roll_below_target() {
        let reg = make_registry_with_charm();
        let skill_id = reg.resolve_skill("CHARM").unwrap();
        let mut world = make_world();
        world.player.skills.insert(
            skill_id,
            undone_domain::SkillValue {
                value: 60,
                modifier: 0,
            },
        );
        // skill=60, dc=50 → target=60. roll=40 → success
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 40);
        let expr = parse("w.checkSkill('CHARM', 50)").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn checkSkill_fails_when_roll_above_target() {
        let reg = make_registry_with_charm();
        let skill_id = reg.resolve_skill("CHARM").unwrap();
        let mut world = make_world();
        world.player.skills.insert(
            skill_id,
            undone_domain::SkillValue {
                value: 60,
                modifier: 0,
            },
        );
        // skill=60, dc=50 → target=60. roll=80 → fail
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 80);
        let expr = parse("w.checkSkill('CHARM', 50)").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn checkSkill_tiered_uses_same_roll() {
        // roll=65; hard dc=70 → target=40 (fail); easy dc=30 → target=80 (success)
        let reg = make_registry_with_charm();
        let skill_id = reg.resolve_skill("CHARM").unwrap();
        let mut world = make_world();
        world.player.skills.insert(
            skill_id,
            undone_domain::SkillValue {
                value: 60,
                modifier: 0,
            },
        );
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 65);
        let hard = parse("w.checkSkill('CHARM', 70)").unwrap();
        assert!(!eval(&hard, &world, &ctx, &reg).unwrap()); // fail
        let easy = parse("w.checkSkill('CHARM', 30)").unwrap();
        assert!(eval(&easy, &world, &ctx, &reg).unwrap()); // success, same roll
    }

    #[test]
    fn checkSkill_minimum_5_percent_chance() {
        let reg = make_registry_with_charm();
        let skill_id = reg.resolve_skill("CHARM").unwrap();
        let mut world = make_world();
        world.player.skills.insert(
            skill_id,
            undone_domain::SkillValue {
                value: 0,
                modifier: 0,
            },
        );
        // skill=0, dc=100 → raw=-50 → clamped to 5. roll=4 → success
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 4);
        let expr = parse("w.checkSkill('CHARM', 100)").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn checkSkillRed_blocked_after_permanent_failure() {
        let reg = make_registry_with_charm();
        let skill_id = reg.resolve_skill("CHARM").unwrap();
        let mut world = make_world();
        world.player.skills.insert(
            skill_id,
            undone_domain::SkillValue {
                value: 100,
                modifier: 0,
            },
        );
        // Record permanent failure
        world.game_data.fail_red_check("my_scene", "CHARM");
        let mut ctx = SceneCtx::new();
        ctx.scene_id = Some("my_scene".to_string());
        // roll=1 would succeed normally, but red check is blocked
        ctx.set_skill_roll("CHARM", 1);
        let expr = parse("w.checkSkillRed('CHARM', 50)").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn arcState_returns_empty_when_not_started() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.arcState('base::jake') == ''").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn arcState_returns_current_state() {
        let mut world = make_world();
        world.game_data.advance_arc("base::jake", "acquaintance");
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.arcState('base::jake') == 'acquaintance'").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn arcStarted_false_initially() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.arcStarted('base::jake')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn arcStarted_true_after_advance() {
        let mut world = make_world();
        world.game_data.advance_arc("base::jake", "met");
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.arcStarted('base::jake')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    // ── New tests ─────────────────────────────────────────────────────────────

    #[test]
    fn beforeRace_returns_before_race() {
        let world = make_world(); // before.race = "white"
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.beforeRace() == 'white'").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn getRace_returns_current_race() {
        let world = make_world(); // race = "east_asian"
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.getRace() == 'east_asian'").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn wasMale_true_for_cis_male() {
        let world = make_world(); // origin = CisMaleTransformed
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.wasMale()").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn wasTransformed_false_for_always_female() {
        let mut world = make_world();
        world.player.origin = PcOrigin::AlwaysFemale;
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.wasTransformed()").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn getAnxiety_returns_value() {
        let world = make_world(); // anxiety = 0
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.getAnxiety() == 0").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn gd_day_returns_day() {
        let world = make_world(); // game_data.day defaults to 0
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.day() == 0").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn gd_isWeekday_true_for_day_0() {
        let world = make_world(); // game_data.day defaults to 0 (Monday)
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.isWeekday()").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn gd_timeSlot_returns_morning() {
        let world = make_world(); // game_data.time_slot defaults to Morning
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("gd.timeSlot() == 'Morning'").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    // ── inCategory tests ──────────────────────────────────────────────────────

    #[test]
    fn inCategory_returns_true_for_matching_age() {
        let world = make_world(); // age = LateTeen
        let ctx = SceneCtx::new();
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_categories(vec![undone_packs::CategoryDef {
            id: "AGE_YOUNG".into(),
            description: "...".into(),
            category_type: undone_packs::CategoryType::Age,
            members: vec![
                "LateTeen".into(),
                "EarlyTwenties".into(),
                "MidLateTwenties".into(),
            ],
        }]);
        let expr = parse("w.inCategory('AGE_YOUNG')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn inCategory_returns_false_for_non_matching_age() {
        let world = make_world(); // age = LateTeen
        let ctx = SceneCtx::new();
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_categories(vec![undone_packs::CategoryDef {
            id: "AGE_MATURE".into(),
            description: "...".into(),
            category_type: undone_packs::CategoryType::Age,
            members: vec!["Thirties".into(), "Forties".into()],
        }]);
        let expr = parse("w.inCategory('AGE_MATURE')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn inCategory_returns_false_for_unknown_category() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.inCategory('NONEXISTENT')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn inCategory_race_returns_true() {
        let world = make_world(); // race = "east_asian"
        let ctx = SceneCtx::new();
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_categories(vec![undone_packs::CategoryDef {
            id: "NON_PRIVILEGED".into(),
            description: "...".into(),
            category_type: undone_packs::CategoryType::Race,
            members: vec!["east_asian".into(), "Black".into()],
        }]);
        let expr = parse("w.inCategory('NON_PRIVILEGED')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    // ── beforeInCategory tests ────────────────────────────────────────────────

    #[test]
    fn beforeInCategory_returns_true_for_before_age() {
        let world = make_world(); // before.age = MidLateTwenties
        let ctx = SceneCtx::new();
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_categories(vec![undone_packs::CategoryDef {
            id: "AGE_YOUNG".into(),
            description: "...".into(),
            category_type: undone_packs::CategoryType::Age,
            members: vec![
                "LateTeen".into(),
                "EarlyTwenties".into(),
                "MidLateTwenties".into(),
            ],
        }]);
        let expr = parse("w.beforeInCategory('AGE_YOUNG')").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn beforeInCategory_returns_false_when_no_before() {
        let mut world = make_world();
        world.player.before = None;
        world.player.origin = PcOrigin::AlwaysFemale;
        let ctx = SceneCtx::new();
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_categories(vec![undone_packs::CategoryDef {
            id: "AGE_YOUNG".into(),
            description: "...".into(),
            category_type: undone_packs::CategoryType::Age,
            members: vec!["LateTeen".into()],
        }]);
        let expr = parse("w.beforeInCategory('AGE_YOUNG')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    // ── before-identity None path tests ──────────────────────────────────────

    #[test]
    fn beforeRace_returns_empty_when_no_before() {
        let mut world = make_world();
        world.player.before = None;
        let ctx = SceneCtx::new();
        let reg = undone_packs::PackRegistry::new();
        let expr = parse("w.beforeRace() == ''").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn hadTraitBefore_returns_false_when_no_before() {
        let mut world = make_world();
        world.player.before = None;
        let ctx = SceneCtx::new();
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        let expr = parse("w.hadTraitBefore('SHY')").unwrap();
        assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
    }

    #[test]
    fn gd_npcLiking_returns_neutral_when_role_not_found() {
        let world = make_world();
        let reg = undone_packs::PackRegistry::new();
        let ctx = SceneCtx::new();
        // No NPC with ROLE_NOBODY exists — should return "Neutral"
        let expr = parse("gd.npcLiking('ROLE_NOBODY') == 'Neutral'").unwrap();
        assert!(eval(&expr, &world, &ctx, &reg).unwrap());
    }
}
