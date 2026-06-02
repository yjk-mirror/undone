//! `w` (player/world) read accessors. Bodies lifted verbatim from
//! `read_api/player.rs`; the thread-local plumbing now lives in the adapters.

use undone_domain::PcOrigin;
use undone_packs::{CategoryType, PackRegistry};
use undone_world::World;

use crate::scene_ctx::SceneCtx;
use crate::script::api::{ApiArg, ApiError, ApiValue};

/// Extract the leading string arg or fail with `BadArgs`.
fn str0<'a>(a: &[ApiArg<'a>], method: &'static str) -> Result<&'a str, ApiError> {
    a.first()
        .and_then(ApiArg::as_str)
        .ok_or(ApiError::BadArgs { method })
}

// ── bool ────────────────────────────────────────────────────────────────────

pub fn has_trait(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let id = str0(a, "hasTrait")?;
    let tid = r.resolve_trait(id).map_err(|_| ApiError::UnknownId {
        kind: "trait",
        id: id.to_string(),
    })?;
    Ok(ApiValue::Bool(w.player.has_trait(tid)))
}

pub fn is_virgin(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.virgin))
}

pub fn is_anal_virgin(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.anal_virgin))
}

pub fn is_drunk(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.is_drunk()))
}

pub fn is_very_drunk(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.is_very_drunk()))
}

pub fn is_max_drunk(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.is_max_drunk()))
}

pub fn is_single(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.partner.is_none()))
}

pub fn is_on_pill(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.on_pill))
}

pub fn is_pregnant(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.pregnancy.is_some()))
}

pub fn always_female(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.origin.is_always_female()))
}

/// Stuff id is intentionally NOT registry-validated: an un-interned id means the
/// player can't have it (returns false), matching the Rhai accessor.
pub fn has_stuff(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let id = str0(a, "hasStuff")?;
    Ok(ApiValue::Bool(match r.resolve_stuff(id) {
        Some(stuff_id) => w.player.stuff.contains(&stuff_id),
        None => false,
    }))
}

pub fn was_male(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.origin.was_male_bodied()))
}

pub fn was_transformed(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.origin.was_transformed()))
}

pub fn has_smooth_legs(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let v = r
        .player_has_smooth_legs(&w.player)
        .map_err(|_| ApiError::UnknownId {
            kind: "trait",
            id: "smooth_legs".to_string(),
        })?;
    Ok(ApiValue::Bool(v))
}

/// CONDITION-ONLY (RNG side effect on the per-scene roll cache, design §4.2).
pub fn check_skill(
    w: &World,
    r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let id = str0(a, "checkSkill")?;
    let dc = a.get(1).and_then(ApiArg::as_int).ok_or(ApiError::BadArgs {
        method: "checkSkill",
    })?;
    let skill_id = r.resolve_skill(id).map_err(|_| ApiError::UnknownId {
        kind: "skill",
        id: id.to_string(),
    })?;
    let skill_value = w.player.skill(skill_id);
    let roll = c.get_or_roll_skill(id);
    let target = (skill_value + (50 - dc as i32)).clamp(5, 95);
    Ok(ApiValue::Bool(roll <= target))
}

/// CONDITION-ONLY (RNG side effect, design §4.2).
pub fn check_skill_red(
    w: &World,
    r: &PackRegistry,
    c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let id = str0(a, "checkSkillRed")?;
    let dc = a.get(1).and_then(ApiArg::as_int).ok_or(ApiError::BadArgs {
        method: "checkSkillRed",
    })?;
    let scene_id = c.scene_id.as_deref().unwrap_or("unknown");
    if w.game_data.has_failed_red_check(scene_id, id) {
        return Ok(ApiValue::Bool(false));
    }
    let skill_id = r.resolve_skill(id).map_err(|_| ApiError::UnknownId {
        kind: "skill",
        id: id.to_string(),
    })?;
    let skill_value = w.player.skill(skill_id);
    let roll = c.get_or_roll_skill(id);
    let target = (skill_value + (50 - dc as i32)).clamp(5, 95);
    Ok(ApiValue::Bool(roll <= target))
}

pub fn had_trait_before(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let id = str0(a, "hadTraitBefore")?;
    let v = match &w.player.before {
        None => false,
        Some(before) => {
            let tid = r.resolve_trait(id).map_err(|_| ApiError::UnknownId {
                kind: "trait",
                id: id.to_string(),
            })?;
            before.traits.contains(&tid)
        }
    };
    Ok(ApiValue::Bool(v))
}

pub fn in_category(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let cat = str0(a, "inCategory")?;
    let v = match r.get_category(cat) {
        None => false,
        Some(cat_def) => match cat_def.category_type {
            CategoryType::Age => {
                let value = format!("{:?}", w.player.age);
                cat_def.members.iter().any(|m| m == &value)
            }
            CategoryType::Race => cat_def.members.iter().any(|m| m == &w.player.race),
            CategoryType::Trait => cat_def.members.iter().any(|m| {
                r.resolve_trait(m)
                    .map(|id| w.player.traits.contains(&id))
                    .unwrap_or(false)
            }),
            CategoryType::Personality => false,
        },
    };
    Ok(ApiValue::Bool(v))
}

pub fn before_in_category(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let cat = str0(a, "beforeInCategory")?;
    let v = match &w.player.before {
        None => false,
        Some(before) => match r.get_category(cat) {
            None => false,
            Some(cat_def) => match cat_def.category_type {
                CategoryType::Age => {
                    let value = format!("{:?}", before.age);
                    cat_def.members.iter().any(|m| m == &value)
                }
                CategoryType::Race => cat_def.members.iter().any(|m| m == &before.race),
                CategoryType::Trait => cat_def.members.iter().any(|m| {
                    r.resolve_trait(m)
                        .map(|id| before.traits.contains(&id))
                        .unwrap_or(false)
                }),
                CategoryType::Personality => false,
            },
        },
    };
    Ok(ApiValue::Bool(v))
}

// ── int ───────────────────────────────────────────────────────────────────────

pub fn get_money(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Int(w.player.money as i64))
}

pub fn get_stress(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Int(w.player.stress.get() as i64))
}

pub fn get_anxiety(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Int(w.player.anxiety.get() as i64))
}

/// Unknown id → `UnknownId` (matches the Rhai accessor; NOT the snapshot's silent 0).
pub fn get_skill(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let id = str0(a, "getSkill")?;
    let skill_id = r.resolve_skill(id).map_err(|_| ApiError::UnknownId {
        kind: "skill",
        id: id.to_string(),
    })?;
    Ok(ApiValue::Int(w.player.skill(skill_id) as i64))
}

pub fn composure(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let skill_id = r.composure_skill().map_err(|_| ApiError::UnknownId {
        kind: "skill",
        id: "COMPOSURE".to_string(),
    })?;
    Ok(ApiValue::Int(w.player.skill(skill_id) as i64))
}

// ── string ──────────────────────────────────────────────────────────────────

pub fn pc_origin(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let s = match w.player.origin {
        PcOrigin::CisMaleTransformed => "CisMaleTransformed",
        PcOrigin::TransWomanTransformed => "TransWomanTransformed",
        PcOrigin::CisFemaleTransformed => "CisFemaleTransformed",
        PcOrigin::AlwaysFemale => "AlwaysFemale",
    };
    Ok(ApiValue::Str(s.to_string()))
}

pub fn before_name(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| b.name.clone())
            .unwrap_or_default(),
    ))
}

pub fn before_race(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| b.race.clone())
            .unwrap_or_default(),
    ))
}

pub fn before_age(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.age))
            .unwrap_or_default(),
    ))
}

pub fn before_sexuality(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.sexuality))
            .unwrap_or_default(),
    ))
}

/// Player name = `active_name(FEMININITY)` — the SAME computation prose uses today
/// (NOT an NPC name; the `getName` divergence is on m/f/role, not w).
pub fn get_name(
    w: &World,
    r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    let fem_id = r.femininity_skill().map_err(|_| ApiError::UnknownId {
        kind: "skill",
        id: "FEMININITY".to_string(),
    })?;
    Ok(ApiValue::Str(w.player.active_name(fem_id).to_string()))
}

pub fn get_race(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(w.player.race.clone()))
}

pub fn get_age(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.age)))
}

pub fn get_arousal(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.arousal)))
}

pub fn get_alcohol(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.alcohol)))
}

pub fn get_height(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.height)))
}

pub fn get_figure(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.figure)))
}

pub fn get_breasts(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.breasts)))
}

pub fn get_butt(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.butt)))
}

pub fn get_waist(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.waist)))
}

pub fn get_lips(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.lips)))
}

pub fn get_hair_colour(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.hair_colour)))
}

pub fn get_hair_length(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.hair_length)))
}

pub fn get_eye_colour(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.eye_colour)))
}

pub fn get_skin_tone(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.skin_tone)))
}

pub fn get_complexion(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.complexion)))
}

pub fn get_appearance(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.appearance)))
}

pub fn get_nipple_sensitivity(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.nipple_sensitivity)))
}

pub fn get_clit_sensitivity(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.clit_sensitivity)))
}

pub fn get_pubic_hair(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.pubic_hair)))
}

pub fn get_natural_pubic_hair(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.natural_pubic_hair)))
}

pub fn get_inner_labia(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.inner_labia)))
}

pub fn get_wetness(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(format!("{:?}", w.player.wetness_baseline)))
}

pub fn before_voice(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.voice))
            .unwrap_or_default(),
    ))
}

pub fn before_height(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.height))
            .unwrap_or_default(),
    ))
}

pub fn before_hair_colour(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.hair_colour))
            .unwrap_or_default(),
    ))
}

pub fn before_eye_colour(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.eye_colour))
            .unwrap_or_default(),
    ))
}

pub fn before_skin_tone(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.skin_tone))
            .unwrap_or_default(),
    ))
}

pub fn before_penis_size(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.penis_size))
            .unwrap_or_default(),
    ))
}

pub fn before_figure(
    w: &World,
    _r: &PackRegistry,
    _c: &SceneCtx,
    _a: &[ApiArg],
) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Str(
        w.player
            .before
            .as_ref()
            .map(|b| format!("{:?}", b.figure))
            .unwrap_or_default(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_ctx::SceneCtx;
    use undone_world::test_helpers::make_test_world;

    fn reg_with_fem() -> PackRegistry {
        let mut r = PackRegistry::new();
        r.register_skills(vec![undone_packs::SkillDef {
            id: "FEMININITY".into(),
            name: "Femininity".into(),
            description: String::new(),
            min: 0,
            max: 100,
        }]);
        r
    }

    #[test]
    fn is_virgin_matches_world() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(
            is_virgin(&w, &r, &c, &[]).unwrap(),
            ApiValue::Bool(w.player.virgin)
        );
    }

    #[test]
    fn get_money_matches_world() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(
            get_money(&w, &r, &c, &[]).unwrap(),
            ApiValue::Int(w.player.money as i64)
        );
    }

    #[test]
    fn debug_getters_match_world() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(
            get_height(&w, &r, &c, &[]).unwrap(),
            ApiValue::Str(format!("{:?}", w.player.height))
        );
        assert_eq!(
            get_arousal(&w, &r, &c, &[]).unwrap(),
            ApiValue::Str(format!("{:?}", w.player.arousal))
        );
        assert_eq!(
            get_race(&w, &r, &c, &[]).unwrap(),
            ApiValue::Str(w.player.race.clone())
        );
    }

    #[test]
    fn get_name_uses_active_name() {
        let r = reg_with_fem();
        let fem = r.resolve_skill("FEMININITY").unwrap();
        let mut w = make_test_world();
        w.player.skills.insert(
            fem,
            undone_domain::SkillValue {
                value: 10,
                modifier: 0,
            },
        );
        let c = SceneCtx::new();
        // FEMININITY 10 → masculine name "Evan"
        assert_eq!(
            get_name(&w, &r, &c, &[]).unwrap(),
            ApiValue::Str("Evan".to_string())
        );
    }

    #[test]
    fn get_skill_unknown_id_errors() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert!(matches!(
            get_skill(&w, &r, &c, &[ApiArg::Str("NOPE")]),
            Err(ApiError::UnknownId { kind: "skill", .. })
        ));
    }

    #[test]
    fn has_stuff_unresolved_is_false() {
        let w = make_test_world();
        let r = PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(
            has_stuff(&w, &r, &c, &[ApiArg::Str("NOPE")]).unwrap(),
            ApiValue::Bool(false)
        );
    }
}
