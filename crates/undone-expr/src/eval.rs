use std::collections::{HashMap, HashSet};

use thiserror::Error;
use undone_domain::{FemaleNpcKey, MaleNpcKey};
use undone_world::World;

use crate::parser::{Call, Expr, Receiver, Value};

/// Per-scene mutable state passed to the evaluator.
/// Lives only for the duration of a scene run.
pub struct SceneCtx {
    pub active_male: Option<MaleNpcKey>,
    pub active_female: Option<FemaleNpcKey>,
    pub scene_flags: HashSet<String>,
    pub weighted_map: HashMap<String, i32>,
}

impl SceneCtx {
    pub fn new() -> Self {
        Self {
            active_male: None,
            active_female: None,
            scene_flags: HashSet::new(),
            weighted_map: HashMap::new(),
        }
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.scene_flags.contains(flag)
    }

    pub fn set_flag(&mut self, flag: impl Into<String>) {
        self.scene_flags.insert(flag.into());
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
}

/// Evaluate a parsed expression to bool.
pub fn eval(expr: &Expr, world: &World, ctx: &SceneCtx) -> Result<bool, EvalError> {
    match expr {
        Expr::Lit(Value::Bool(b)) => Ok(*b),
        Expr::Lit(_) => Ok(true), // non-bool literals as conditions are truthy

        Expr::Not(inner) => Ok(!eval(inner, world, ctx)?),

        Expr::And(l, r) => Ok(eval(l, world, ctx)? && eval(r, world, ctx)?),
        Expr::Or(l, r) => Ok(eval(l, world, ctx)? || eval(r, world, ctx)?),

        Expr::Eq(l, r) => {
            let lv = eval_to_value(l, world, ctx)?;
            let rv = eval_to_value(r, world, ctx)?;
            Ok(lv == rv)
        }
        Expr::Ne(l, r) => {
            let lv = eval_to_value(l, world, ctx)?;
            let rv = eval_to_value(r, world, ctx)?;
            Ok(lv != rv)
        }
        Expr::Lt(l, r) => int_compare(l, r, world, ctx, |a, b| a < b),
        Expr::Gt(l, r) => int_compare(l, r, world, ctx, |a, b| a > b),
        Expr::Le(l, r) => int_compare(l, r, world, ctx, |a, b| a <= b),
        Expr::Ge(l, r) => int_compare(l, r, world, ctx, |a, b| a >= b),

        Expr::Call(call) => eval_call_bool(call, world, ctx),
    }
}

fn int_compare(
    l: &Expr,
    r: &Expr,
    world: &World,
    ctx: &SceneCtx,
    cmp: impl Fn(i64, i64) -> bool,
) -> Result<bool, EvalError> {
    let lv = eval_to_int(l, world, ctx)?;
    let rv = eval_to_int(r, world, ctx)?;
    Ok(cmp(lv, rv))
}

/// Evaluate an expression to a generic Value for comparison.
fn eval_to_value(expr: &Expr, world: &World, ctx: &SceneCtx) -> Result<EvalValue, EvalError> {
    match expr {
        Expr::Lit(v) => Ok(match v {
            Value::Str(s) => EvalValue::Str(s.clone()),
            Value::Int(n) => EvalValue::Int(*n),
            Value::Bool(b) => EvalValue::Bool(*b),
        }),
        Expr::Call(call) => {
            // Try to eval as int, then bool
            if let Ok(n) = eval_call_int(call, world, ctx) {
                return Ok(EvalValue::Int(n));
            }
            let b = eval_call_bool(call, world, ctx)?;
            Ok(EvalValue::Bool(b))
        }
        other => {
            let b = eval(other, world, ctx)?;
            Ok(EvalValue::Bool(b))
        }
    }
}

fn eval_to_int(expr: &Expr, world: &World, ctx: &SceneCtx) -> Result<i64, EvalError> {
    match expr {
        Expr::Lit(Value::Int(n)) => Ok(*n),
        Expr::Call(call) => eval_call_int(call, world, ctx),
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
pub fn eval_call_bool(call: &Call, world: &World, ctx: &SceneCtx) -> Result<bool, EvalError> {
    let str_arg = |i: usize| -> Result<&str, EvalError> {
        match call.args.get(i) {
            Some(Value::Str(s)) => Ok(s.as_str()),
            _ => Err(EvalError::BadArg(call.method.clone())),
        }
    };

    match call.receiver {
        Receiver::Player => match call.method.as_str() {
            "hasTrait" => {
                // TODO: wire to registry in scene engine — for now always false on empty traits
                let _ = str_arg(0)?; // validate arg is present and is a string
                Ok(false)
            }
            "isVirgin" => Ok(world.player.virgin),
            "isAnalVirgin" => Ok(world.player.anal_virgin),
            "isDrunk" => Ok(world.player.is_drunk()),
            "isVeryDrunk" => Ok(world.player.is_very_drunk()),
            "isMaxDrunk" => Ok(world.player.is_max_drunk()),
            "isSingle" => Ok(world.player.partner.is_none()),
            "isOnPill" => Ok(world.player.on_pill),
            "isPregnant" => Ok(world.player.pregnancy.is_some()),
            "alwaysFemale" => Ok(world.player.always_female),
            "hasStuff" => {
                let _ = str_arg(0)?; // validate arg
                Ok(false) // TODO: wire to StuffId
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
                    let _ = str_arg(0)?; // validate arg
                    Ok(false) // TODO: wire to registry
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
                "isNpcLoveCrush" => {
                    Ok(npc.core.npc_love >= undone_domain::LoveLevel::Crush)
                }
                "isNpcLoveSome" => Ok(npc.core.npc_love >= undone_domain::LoveLevel::Some),
                "isWLoveCrush" => Ok(npc.core.pc_love >= undone_domain::LoveLevel::Crush),
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
                _ => Err(EvalError::UnknownMethod {
                    receiver: "f".into(),
                    method: call.method.clone(),
                }),
            }
        }
    }
}

/// Evaluate a method call that returns an integer (e.g. getSkill, getStat, week).
pub fn eval_call_int(call: &Call, world: &World, _ctx: &SceneCtx) -> Result<i64, EvalError> {
    match call.receiver {
        Receiver::Player => match call.method.as_str() {
            "getMoney" => Ok(world.player.money as i64),
            "getStress" => Ok(world.player.stress as i64),
            "getSkill" => Ok(0), // TODO: wire to SkillId via registry
            _ => Err(EvalError::UnknownMethod {
                receiver: "w".into(),
                method: call.method.clone(),
            }),
        },
        Receiver::GameData => match call.method.as_str() {
            "week" => Ok(world.game_data.week as i64),
            "getStat" => Ok(0), // TODO: wire to StatId via registry
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

#[cfg(test)]
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
                name: "Eva".into(),
                age: Age::LateTeen,
                race: "east_asian".into(),
                figure: PlayerFigure::Slim,
                breasts: BreastSize::Large,
                eye_colour: "brown".into(),
                hair_colour: "dark".into(),
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
                always_female: false,
                femininity: 10,
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
        let expr = parse("true").unwrap();
        assert!(eval(&expr, &world, &ctx).unwrap());
    }

    #[test]
    fn eval_is_virgin() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let expr = parse("w.isVirgin()").unwrap();
        assert!(eval(&expr, &world, &ctx).unwrap());
    }

    #[test]
    fn eval_is_not_drunk() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let expr = parse("!w.isDrunk()").unwrap();
        assert!(eval(&expr, &world, &ctx).unwrap());
    }

    #[test]
    fn eval_game_flag_absent() {
        let world = make_world();
        let ctx = SceneCtx::new();
        let expr = parse("gd.hasGameFlag('SOME_FLAG')").unwrap();
        assert!(!eval(&expr, &world, &ctx).unwrap());
    }

    #[test]
    fn eval_game_flag_present() {
        let mut world = make_world();
        world.game_data.set_flag("SOME_FLAG");
        let ctx = SceneCtx::new();
        let expr = parse("gd.hasGameFlag('SOME_FLAG')").unwrap();
        assert!(eval(&expr, &world, &ctx).unwrap());
    }

    #[test]
    fn eval_money_comparison() {
        let world = make_world(); // money = 500
        let ctx = SceneCtx::new();
        let expr = parse("w.getMoney() > 100").unwrap();
        assert!(eval(&expr, &world, &ctx).unwrap());
    }

    #[test]
    fn eval_scene_flag() {
        let world = make_world();
        let mut ctx = SceneCtx::new();
        ctx.set_flag("offered_umbrella");
        let expr = parse("scene.hasFlag('offered_umbrella')").unwrap();
        assert!(eval(&expr, &world, &ctx).unwrap());
    }
}
