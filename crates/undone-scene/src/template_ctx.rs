//! Prose rendering.
//!
//! The six receiver objects (`w`/`gd`/`scene`/`role`/`m`/`f`) are zero-sized `Object`
//! views defined in `script::api::minijinja_bind`. They read live `World` through the
//! thread-local read guard during render — there is NO materialized snapshot. The only
//! snapshot-era logic that remains is NPC *presence*: `m`/`f` bind to their view when an
//! NPC of that sex is active, else `Value::UNDEFINED` so `{% if m %}` stays falsy.

use minijinja::value::Value;
use undone_packs::PackRegistry;
use undone_world::World;

use crate::scene_ctx::SceneCtx;
use crate::script::api::minijinja_bind::{FView, GdView, MView, RoleView, SceneView, WView};
use crate::script::context::ReadCtxGuard;

/// Render a prose template against live game state.
///
/// Reads flow through the same registry accessors the Rhai engine uses, so prose and
/// conditions can never diverge on a value (design §6). The six receivers are bound as
/// ZST views; `w`/`gd`/`scene`/`role` are always present, `m`/`f` only when an NPC of
/// that sex is active in `ctx`.
pub fn render_prose(
    template_str: &str,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<String, minijinja::Error> {
    // NPC presence — computed from the owned `ctx` borrow BEFORE installing the guard,
    // so the guard's "ctx borrowed for the whole call" invariant is unaffected (§6.1).
    let active_male = if ctx.active_male.is_some() {
        Value::from_object(MView)
    } else {
        Value::UNDEFINED
    };
    let active_female = if ctx.active_female.is_some() {
        Value::from_object(FView)
    } else {
        Value::UNDEFINED
    };

    // SAFETY/INVARIANT: `render` is synchronous and single-threaded; this guard lives
    // for the entire render call, exactly the invariant the ZST views rely on to read
    // live `World`. NEVER switch to `render_and_return_state` (design §6): it can retain
    // the root `Value` (and thus the view objects) past this call, which — combined with
    // live-context reading — is the one way to invoke a view after the guard drops.
    let _guard = ReadCtxGuard::install(world, registry, ctx);

    let mut env = minijinja::Environment::new();
    env.add_template("prose", template_str)?;
    let tmpl = env.get_template("prose")?;
    tmpl.render(minijinja::context! {
        w => Value::from_object(WView),
        gd => Value::from_object(GdView),
        scene => Value::from_object(SceneView),
        role => Value::from_object(RoleView),
        m => active_male,
        f => active_female,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use lasso::Key;
    use std::collections::{HashMap, HashSet};
    use undone_domain::{Appearance, BeforeVoice};

    use crate::scene_ctx::SceneNpcRef;
    use undone_world::test_helpers::make_test_world as make_world;

    #[test]
    fn hasTrait_in_template_branches_correctly() {
        // Register SHY trait and give it to the player
        let mut registry = undone_packs::PackRegistry::new();
        registry.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        let shy_id = registry.resolve_trait("SHY").unwrap();
        let mut world = make_world();
        world.player.traits.insert(shy_id);

        let ctx = SceneCtx::new();
        let template = r#"{% if w.hasTrait("SHY") %}shy{% else %}bold{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(result.contains("shy"), "expected 'shy' in '{result}'");
        assert!(
            !result.contains("bold"),
            "did not expect 'bold' in '{result}'"
        );
    }

    #[test]
    fn getSkill_in_template_returns_value() {
        let mut registry = undone_packs::PackRegistry::new();
        registry.register_skills(vec![undone_packs::SkillDef {
            id: "CHARM".into(),
            name: "Charm".into(),
            description: "".into(),
            min: 0,
            max: 100,
        }]);
        let skill_id = registry.resolve_skill("CHARM").unwrap();
        let mut world = make_world();
        world.player.skills.insert(
            skill_id,
            undone_domain::SkillValue {
                value: 65,
                modifier: 0,
            },
        );
        let ctx = SceneCtx::new();
        let template = r#"{% if w.getSkill("CHARM") > 50 %}skilled{% else %}unskilled{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(
            result.contains("skilled"),
            "expected 'skilled' in '{result}'"
        );
    }

    #[test]
    fn timeSlot_in_template() {
        let registry = undone_packs::PackRegistry::new();
        let world = make_world(); // time_slot = Morning
        let ctx = SceneCtx::new();
        let template = r#"{% if gd.timeSlot() == "Morning" %}morning{% else %}other{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(
            result.contains("morning"),
            "expected 'morning' in '{result}'"
        );
    }

    #[test]
    fn scene_hasFlag_in_template() {
        let registry = undone_packs::PackRegistry::new();
        let world = make_world();
        let mut ctx = SceneCtx::new();
        ctx.set_flag("umbrella_offered");

        let template = r#"{% if scene.hasFlag("umbrella_offered") %}yes{% else %}no{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(result.contains("yes"), "expected 'yes' in '{result}'");
    }

    #[test]
    fn arcState_in_template_branches_on_state() {
        let registry = undone_packs::PackRegistry::new();
        let mut world = make_world();
        world
            .game_data
            .arc_states
            .insert("base::workplace_opening".to_string(), "working".to_string());

        let ctx = SceneCtx::new();
        let template = r#"{% if gd.arcState("base::workplace_opening") == "working" %}on-the-job{% else %}not-started{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(
            result.contains("on-the-job"),
            "expected 'on-the-job' in '{result}'"
        );
    }

    #[test]
    fn arcState_in_template_returns_empty_when_arc_not_started() {
        let registry = undone_packs::PackRegistry::new();
        let world = make_world();
        let ctx = SceneCtx::new();
        let template = r#"{% if gd.arcState("base::workplace_opening") == "" %}not-started{% else %}started{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(
            result.contains("not-started"),
            "expected 'not-started' in '{result}'"
        );
    }

    #[test]
    fn getAppearance_in_template() {
        let registry = undone_packs::PackRegistry::new();
        let mut world = make_world();
        world.player.appearance = Appearance::Stunning;
        let ctx = SceneCtx::new();
        let template = r#"{% if w.getAppearance() == "Stunning" %}wow{% else %}meh{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(result.contains("wow"), "expected 'wow' in '{result}'");
    }

    #[test]
    fn beforeVoice_in_template() {
        let registry = undone_packs::PackRegistry::new();
        let mut world = make_world();
        world.player.before.as_mut().unwrap().voice = BeforeVoice::Deep;
        let ctx = SceneCtx::new();
        let template = r#"{{ w.beforeVoice() }}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert_eq!(result.trim(), "Deep");
    }

    #[test]
    fn hasSmoothLegs_true_with_naturally_smooth() {
        let mut registry = undone_packs::PackRegistry::new();
        registry.register_traits(vec![undone_packs::TraitDef {
            id: "NATURALLY_SMOOTH".into(),
            name: "Naturally Smooth".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);
        let trait_id = registry.resolve_trait("NATURALLY_SMOOTH").unwrap();
        let mut world = make_world();
        world.player.traits.insert(trait_id);

        let ctx = SceneCtx::new();
        let template = r#"{% if w.hasSmoothLegs() %}smooth{% else %}hairy{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(result.contains("smooth"), "expected 'smooth' in '{result}'");
    }

    #[test]
    fn getName_returns_active_name() {
        let mut registry = undone_packs::PackRegistry::new();
        registry.register_skills(vec![undone_packs::SkillDef {
            id: "FEMININITY".into(),
            name: "Femininity".into(),
            description: "".into(),
            min: 0,
            max: 100,
        }]);
        let fem_id = registry.resolve_skill("FEMININITY").unwrap();
        let mut world = make_world();
        // FEMININITY = 10 → should use name_masc ("Evan")
        world.player.skills.insert(
            fem_id,
            undone_domain::SkillValue {
                value: 10,
                modifier: 0,
            },
        );

        let ctx = SceneCtx::new();
        let template = r#"{{ w.getName() }}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert_eq!(result.trim(), "Evan");

        // FEMININITY = 75 → should use name_fem ("Eva")
        world.player.skills.get_mut(&fem_id).unwrap().value = 75;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert_eq!(result.trim(), "Eva");
    }

    #[test]
    fn role_lookup_can_render_multiple_bound_npcs() {
        let mut registry = undone_packs::PackRegistry::new();
        let romantic = registry.intern_personality("ROMANTIC");
        let calm = registry.intern_personality("CALM");
        let mut world = make_world();

        let male = undone_domain::MaleNpc {
            core: undone_domain::NpcCore {
                name: "Dan".into(),
                display_name: None,
                age: undone_domain::Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "blue".into(),
                hair_colour: "brown".into(),
                personality: romantic,
                traits: HashSet::new(),
                relationship: undone_domain::RelationshipStatus::Acquaintance,
                pc_liking: undone_domain::LikingLevel::Like,
                npc_liking: undone_domain::LikingLevel::Neutral,
                pc_love: undone_domain::LoveLevel::None,
                npc_love: undone_domain::LoveLevel::None,
                pc_attraction: undone_domain::AttractionLevel::Attracted,
                npc_attraction: undone_domain::AttractionLevel::Ok,
                behaviour: undone_domain::Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: undone_domain::ArousalLevel::Comfort,
                alcohol: undone_domain::AlcoholLevel::Sober,
                roles: HashSet::new(),
            },
            figure: undone_domain::MaleFigure::Average,
            clothing: undone_domain::MaleClothing::default(),
            had_orgasm: false,
            has_baby_with_pc: false,
        };
        let female = undone_domain::FemaleNpc {
            core: undone_domain::NpcCore {
                name: "Mia".into(),
                display_name: None,
                age: undone_domain::Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "green".into(),
                hair_colour: "black".into(),
                personality: calm,
                traits: HashSet::new(),
                relationship: undone_domain::RelationshipStatus::Acquaintance,
                pc_liking: undone_domain::LikingLevel::Like,
                npc_liking: undone_domain::LikingLevel::Neutral,
                pc_love: undone_domain::LoveLevel::None,
                npc_love: undone_domain::LoveLevel::None,
                pc_attraction: undone_domain::AttractionLevel::Unattracted,
                npc_attraction: undone_domain::AttractionLevel::Unattracted,
                behaviour: undone_domain::Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: undone_domain::ArousalLevel::Comfort,
                alcohol: undone_domain::AlcoholLevel::Sober,
                roles: HashSet::new(),
            },
            char_type: undone_domain::CharTypeId::from_spur(
                lasso::Spur::try_from_usize(0).unwrap(),
            ),
            figure: undone_domain::PlayerFigure::Slim,
            breasts: undone_domain::BreastSize::Average,
            clothing: undone_domain::FemaleClothing::default(),
            pregnancy: None,
            virgin: true,
        };

        let male_key = world.male_npcs.insert(male);
        let female_key = world.female_npcs.insert(female);
        let mut ctx = SceneCtx::new();
        ctx.bind_role("ROLE_TEAM_LEAD", SceneNpcRef::Male(male_key));
        ctx.bind_role("ROLE_DESIGNER", SceneNpcRef::Female(female_key));

        let template =
            r#"{{ role.getName("ROLE_TEAM_LEAD") }} and {{ role.getName("ROLE_DESIGNER") }}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(result.contains("Dan"));
        assert!(result.contains("Mia"));
    }

    // ── NPC presence + loud-error acceptance (design §6.1) ────────────────────

    fn male_personality_world() -> (World, undone_domain::MaleNpcKey, undone_packs::PackRegistry) {
        let mut registry = undone_packs::PackRegistry::new();
        let personality = registry.intern_personality("ROMANTIC");
        let mut world = make_world();
        let key = world
            .male_npcs
            .insert(undone_world::test_helpers::make_test_male_npc(personality));
        (world, key, registry)
    }

    #[test]
    fn if_m_truthy_when_male_bound_falsy_when_not() {
        let (world, key, registry) = male_personality_world();
        let template = r#"{% if m %}Y{% else %}N{% endif %}"#;

        let mut ctx = SceneCtx::new();
        assert_eq!(
            render_prose(template, &world, &ctx, &registry).unwrap(),
            "N",
            "no male bound → m is falsy"
        );

        ctx.active_male = Some(key);
        assert_eq!(
            render_prose(template, &world, &ctx, &registry).unwrap(),
            "Y",
            "male bound → m is truthy"
        );
    }

    #[test]
    fn m_method_with_no_male_errors_loud() {
        let registry = undone_packs::PackRegistry::new();
        let world = make_world();
        let ctx = SceneCtx::new();
        // No male bound: m is UNDEFINED, so calling a method on it must error (loud),
        // not render empty.
        let err = render_prose(r#"{{ m.getName() }}"#, &world, &ctx, &registry);
        assert!(
            err.is_err(),
            "m.getName() with no male must error, got {err:?}"
        );
    }

    #[test]
    fn unbound_role_lookup_errors() {
        let registry = undone_packs::PackRegistry::new();
        let world = make_world();
        let ctx = SceneCtx::new();
        let err = render_prose(r#"{{ role.getName("NOPE") }}"#, &world, &ctx, &registry);
        assert!(err.is_err(), "unbound role lookup must error, got {err:?}");
    }

    #[test]
    fn unknown_prose_method_errors() {
        let registry = undone_packs::PackRegistry::new();
        let world = make_world();
        let ctx = SceneCtx::new();
        let err = render_prose(r#"{{ w.notAReal() }}"#, &world, &ctx, &registry);
        assert!(err.is_err(), "unknown method must error");
    }
}
