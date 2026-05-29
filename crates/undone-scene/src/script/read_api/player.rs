//! The `w` receiver — player/world reads.
//!
//! Worked reference for the read-API port. Every method mirrors a `Receiver::Player`
//! arm in `undone_expr::eval` (`eval_call_bool` / `eval_call_int` / `eval_call_string`),
//! preserving the exact method name and argument shape so existing condition
//! strings work verbatim. Content-id resolution returns a Rhai `Err` on an unknown
//! id (the runtime half of the fail-fast guarantee).

use undone_domain::PcOrigin;
use undone_packs::CategoryType;

use crate::script::context::{unknown_id_err, with_read_ctx};

type RhaiResult<T> = Result<T, Box<rhai::EvalAltResult>>;

/// Zero-sized `w` handle; reads the thread-local evaluation context.
#[derive(Clone)]
pub struct W;

impl W {
    // ── bool methods (eval_call_bool, Receiver::Player) ──────────────────────

    fn has_trait(&mut self, id: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, _ctx| {
            let tid = reg
                .resolve_trait(id)
                .map_err(|_| unknown_id_err("trait", id))?;
            Ok(world.player.has_trait(tid))
        })
    }

    fn is_virgin(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.virgin))
    }

    fn is_anal_virgin(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.anal_virgin))
    }

    fn is_drunk(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.is_drunk()))
    }

    fn is_very_drunk(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.is_very_drunk()))
    }

    fn is_max_drunk(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.is_max_drunk()))
    }

    fn is_single(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.partner.is_none()))
    }

    fn is_on_pill(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.on_pill))
    }

    fn is_pregnant(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.pregnancy.is_some()))
    }

    fn always_female(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.origin.is_always_female()))
    }

    fn has_stuff(&mut self, id: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, _ctx| match reg.resolve_stuff(id) {
            // never interned = player can't have it (matches eval.rs)
            Some(stuff_id) => Ok(world.player.stuff.contains(&stuff_id)),
            None => Ok(false),
        })
    }

    fn was_male(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.origin.was_male_bodied()))
    }

    fn was_transformed(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.origin.was_transformed()))
    }

    fn has_smooth_legs(&mut self) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, _ctx| {
            reg.player_has_smooth_legs(&world.player)
                .map_err(|_| unknown_id_err("trait", "smooth_legs"))
        })
    }

    fn check_skill(&mut self, id: &str, dc: i64) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, ctx| {
            let skill_id = reg
                .resolve_skill(id)
                .map_err(|_| unknown_id_err("skill", id))?;
            let skill_value = world.player.skill(skill_id);
            let roll = ctx.get_or_roll_skill(id);
            let target = (skill_value + (50 - dc as i32)).clamp(5, 95);
            Ok(roll <= target)
        })
    }

    fn check_skill_red(&mut self, id: &str, dc: i64) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, ctx| {
            let scene_id = ctx.scene_id.as_deref().unwrap_or("unknown");
            if world.game_data.has_failed_red_check(scene_id, id) {
                return Ok(false);
            }
            let skill_id = reg
                .resolve_skill(id)
                .map_err(|_| unknown_id_err("skill", id))?;
            let skill_value = world.player.skill(skill_id);
            let roll = ctx.get_or_roll_skill(id);
            let target = (skill_value + (50 - dc as i32)).clamp(5, 95);
            Ok(roll <= target)
            // NOTE: marking failure is an effect (FailRedCheck), not done here.
        })
    }

    fn had_trait_before(&mut self, id: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, _ctx| match &world.player.before {
            None => Ok(false),
            Some(before) => {
                let tid = reg
                    .resolve_trait(id)
                    .map_err(|_| unknown_id_err("trait", id))?;
                Ok(before.traits.contains(&tid))
            }
        })
    }

    fn in_category(&mut self, cat: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, _ctx| match reg.get_category(cat) {
            None => Ok(false),
            Some(cat_def) => match cat_def.category_type {
                CategoryType::Age => {
                    let value = format!("{:?}", world.player.age);
                    Ok(cat_def.members.iter().any(|m| m == &value))
                }
                CategoryType::Race => Ok(cat_def.members.iter().any(|m| m == &world.player.race)),
                CategoryType::Trait => Ok(cat_def.members.iter().any(|m| {
                    reg.resolve_trait(m)
                        .map(|id| world.player.traits.contains(&id))
                        .unwrap_or(false)
                })),
                CategoryType::Personality => Ok(false),
            },
        })
    }

    fn before_in_category(&mut self, cat: &str) -> RhaiResult<bool> {
        with_read_ctx(|world, reg, _ctx| match &world.player.before {
            None => Ok(false),
            Some(before) => match reg.get_category(cat) {
                None => Ok(false),
                Some(cat_def) => match cat_def.category_type {
                    CategoryType::Age => {
                        let value = format!("{:?}", before.age);
                        Ok(cat_def.members.iter().any(|m| m == &value))
                    }
                    CategoryType::Race => Ok(cat_def.members.iter().any(|m| m == &before.race)),
                    CategoryType::Trait => Ok(cat_def.members.iter().any(|m| {
                        reg.resolve_trait(m)
                            .map(|id| before.traits.contains(&id))
                            .unwrap_or(false)
                    })),
                    CategoryType::Personality => Ok(false),
                },
            },
        })
    }

    // ── int methods (eval_call_int, Receiver::Player) ────────────────────────

    fn get_money(&mut self) -> RhaiResult<i64> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.money as i64))
    }

    fn get_stress(&mut self) -> RhaiResult<i64> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.stress.get() as i64))
    }

    fn get_anxiety(&mut self) -> RhaiResult<i64> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.anxiety.get() as i64))
    }

    fn get_skill(&mut self, id: &str) -> RhaiResult<i64> {
        with_read_ctx(|world, reg, _ctx| {
            let skill_id = reg
                .resolve_skill(id)
                .map_err(|_| unknown_id_err("skill", id))?;
            Ok(world.player.skill(skill_id) as i64)
        })
    }

    // ── string methods (eval_call_string, Receiver::Player) ──────────────────

    fn pc_origin(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            let s = match world.player.origin {
                PcOrigin::CisMaleTransformed => "CisMaleTransformed",
                PcOrigin::TransWomanTransformed => "TransWomanTransformed",
                PcOrigin::CisFemaleTransformed => "CisFemaleTransformed",
                PcOrigin::AlwaysFemale => "AlwaysFemale",
            };
            Ok(s.to_string())
        })
    }

    fn before_name(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| b.name.clone())
                .unwrap_or_default())
        })
    }

    fn before_race(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| b.race.clone())
                .unwrap_or_default())
        })
    }

    fn before_age(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.age))
                .unwrap_or_default())
        })
    }

    fn before_sexuality(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.sexuality))
                .unwrap_or_default())
        })
    }

    fn get_name(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, reg, _ctx| {
            let fem_id = reg
                .femininity_skill()
                .map_err(|_| unknown_id_err("skill", "FEMININITY"))?;
            Ok(world.player.active_name(fem_id).to_string())
        })
    }

    fn get_race(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(world.player.race.clone()))
    }

    fn get_age(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.age)))
    }

    fn get_arousal(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.arousal)))
    }

    fn get_alcohol(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.alcohol)))
    }

    fn get_height(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.height)))
    }

    fn get_figure(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.figure)))
    }

    fn get_breasts(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.breasts)))
    }

    fn get_butt(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.butt)))
    }

    fn get_waist(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.waist)))
    }

    fn get_lips(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.lips)))
    }

    fn get_hair_colour(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.hair_colour)))
    }

    fn get_hair_length(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.hair_length)))
    }

    fn get_eye_colour(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.eye_colour)))
    }

    fn get_skin_tone(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.skin_tone)))
    }

    fn get_complexion(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.complexion)))
    }

    fn get_appearance(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.appearance)))
    }

    fn get_nipple_sensitivity(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.nipple_sensitivity)))
    }

    fn get_clit_sensitivity(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.clit_sensitivity)))
    }

    fn get_pubic_hair(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.pubic_hair)))
    }

    fn get_natural_pubic_hair(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.natural_pubic_hair)))
    }

    fn get_inner_labia(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.inner_labia)))
    }

    fn get_wetness(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| Ok(format!("{:?}", world.player.wetness_baseline)))
    }

    fn before_voice(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.voice))
                .unwrap_or_default())
        })
    }

    fn before_height(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.height))
                .unwrap_or_default())
        })
    }

    fn before_hair_colour(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.hair_colour))
                .unwrap_or_default())
        })
    }

    fn before_eye_colour(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.eye_colour))
                .unwrap_or_default())
        })
    }

    fn before_skin_tone(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.skin_tone))
                .unwrap_or_default())
        })
    }

    fn before_penis_size(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.penis_size))
                .unwrap_or_default())
        })
    }

    fn before_figure(&mut self) -> RhaiResult<String> {
        with_read_ctx(|world, _reg, _ctx| {
            Ok(world
                .player
                .before
                .as_ref()
                .map(|b| format!("{:?}", b.figure))
                .unwrap_or_default())
        })
    }
}

/// Register the `W` type and its methods. Names match the authored condition
/// syntax (`w.hasTrait(...)`, `w.getSkill(...)`, …) exactly.
pub fn register(engine: &mut rhai::Engine) {
    engine
        .register_type::<W>()
        // bool
        .register_fn("hasTrait", W::has_trait)
        .register_fn("isVirgin", W::is_virgin)
        .register_fn("isAnalVirgin", W::is_anal_virgin)
        .register_fn("isDrunk", W::is_drunk)
        .register_fn("isVeryDrunk", W::is_very_drunk)
        .register_fn("isMaxDrunk", W::is_max_drunk)
        .register_fn("isSingle", W::is_single)
        .register_fn("isOnPill", W::is_on_pill)
        .register_fn("isPregnant", W::is_pregnant)
        .register_fn("alwaysFemale", W::always_female)
        .register_fn("hasStuff", W::has_stuff)
        .register_fn("wasMale", W::was_male)
        .register_fn("wasTransformed", W::was_transformed)
        .register_fn("hasSmoothLegs", W::has_smooth_legs)
        .register_fn("checkSkill", W::check_skill)
        .register_fn("checkSkillRed", W::check_skill_red)
        .register_fn("hadTraitBefore", W::had_trait_before)
        .register_fn("inCategory", W::in_category)
        .register_fn("beforeInCategory", W::before_in_category)
        // int
        .register_fn("getMoney", W::get_money)
        .register_fn("getStress", W::get_stress)
        .register_fn("getAnxiety", W::get_anxiety)
        .register_fn("getSkill", W::get_skill)
        // string
        .register_fn("pcOrigin", W::pc_origin)
        .register_fn("beforeName", W::before_name)
        .register_fn("beforeRace", W::before_race)
        .register_fn("beforeAge", W::before_age)
        .register_fn("beforeSexuality", W::before_sexuality)
        .register_fn("getName", W::get_name)
        .register_fn("getRace", W::get_race)
        .register_fn("getAge", W::get_age)
        .register_fn("getArousal", W::get_arousal)
        .register_fn("getAlcohol", W::get_alcohol)
        .register_fn("getHeight", W::get_height)
        .register_fn("getFigure", W::get_figure)
        .register_fn("getBreasts", W::get_breasts)
        .register_fn("getButt", W::get_butt)
        .register_fn("getWaist", W::get_waist)
        .register_fn("getLips", W::get_lips)
        .register_fn("getHairColour", W::get_hair_colour)
        .register_fn("getHairLength", W::get_hair_length)
        .register_fn("getEyeColour", W::get_eye_colour)
        .register_fn("getSkinTone", W::get_skin_tone)
        .register_fn("getComplexion", W::get_complexion)
        .register_fn("getAppearance", W::get_appearance)
        .register_fn("getNippleSensitivity", W::get_nipple_sensitivity)
        .register_fn("getClitSensitivity", W::get_clit_sensitivity)
        .register_fn("getPubicHair", W::get_pubic_hair)
        .register_fn("getNaturalPubicHair", W::get_natural_pubic_hair)
        .register_fn("getInnerLabia", W::get_inner_labia)
        .register_fn("getWetness", W::get_wetness)
        .register_fn("beforeVoice", W::before_voice)
        .register_fn("beforeHeight", W::before_height)
        .register_fn("beforeHairColour", W::before_hair_colour)
        .register_fn("beforeEyeColour", W::before_eye_colour)
        .register_fn("beforeSkinTone", W::before_skin_tone)
        .register_fn("beforePenisSize", W::before_penis_size)
        .register_fn("beforeFigure", W::before_figure);
}
