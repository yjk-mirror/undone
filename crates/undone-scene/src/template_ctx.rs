use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use minijinja::{
    value::{Object, ObjectRepr, Value},
    Error, ErrorKind, State,
};
use undone_domain::PcOrigin;
use undone_expr::SceneCtx;
use undone_packs::PackRegistry;
use undone_world::World;

// ---------------------------------------------------------------------------
// PlayerCtx — wraps player data with pre-resolved trait strings
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct PlayerCtx {
    pub trait_strings: HashSet<String>,
    pub virgin: bool,
    pub origin: PcOrigin,
    pub partner: bool, // true = has partner (i.e. NOT single)
    pub on_pill: bool,
    pub pregnant: bool,
    /// skill_id_string → effective value (base + modifier clamped to range)
    pub skills: HashMap<String, i32>,
    pub money: i32,
    pub stress: i32,
    pub anxiety: i32,
    /// Display string for arousal level, e.g. "Comfort"
    pub arousal: String,
    /// Display string for alcohol level, e.g. "Sober"
    pub alcohol: String,
}

impl fmt::Display for PlayerCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlayerCtx")
    }
}

impl Object for PlayerCtx {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State<'_, '_>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        match method {
            "hasTrait" => {
                let id = string_arg(method, args, 0)?;
                Ok(Value::from(self.trait_strings.contains(id.as_str())))
            }
            "isVirgin" => Ok(Value::from(self.virgin)),
            "alwaysFemale" => Ok(Value::from(self.origin.is_always_female())),
            "pcOrigin" => {
                let s = match self.origin {
                    PcOrigin::CisMaleTransformed => "CisMaleTransformed",
                    PcOrigin::TransWomanTransformed => "TransWomanTransformed",
                    PcOrigin::CisFemaleTransformed => "CisFemaleTransformed",
                    PcOrigin::AlwaysFemale => "AlwaysFemale",
                };
                Ok(Value::from(s))
            }
            "isSingle" => Ok(Value::from(!self.partner)),
            "isOnPill" => Ok(Value::from(self.on_pill)),
            "isPregnant" => Ok(Value::from(self.pregnant)),
            "getSkill" => {
                let id = string_arg(method, args, 0)?;
                Ok(Value::from(*self.skills.get(id.as_str()).unwrap_or(&0)))
            }
            "getMoney" => Ok(Value::from(self.money)),
            "getStress" => Ok(Value::from(self.stress)),
            "getAnxiety" => Ok(Value::from(self.anxiety)),
            "getArousal" => Ok(Value::from(self.arousal.as_str())),
            "getAlcohol" => Ok(Value::from(self.alcohol.as_str())),
            "wasMale" => Ok(Value::from(self.origin.was_male_bodied())),
            "wasTransformed" => Ok(Value::from(self.origin.was_transformed())),
            _ => Err(Error::new(
                ErrorKind::UnknownMethod,
                format!("w has no method '{method}'"),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// GameDataCtx
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct GameDataCtx {
    pub week: u32,
    pub day: u8,
    pub time_slot: String,
    pub flags: HashSet<String>,
    pub arc_states: HashMap<String, String>,
}

impl fmt::Display for GameDataCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GameDataCtx")
    }
}

impl Object for GameDataCtx {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State<'_, '_>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        match method {
            "week" => Ok(Value::from(self.week)),
            "day" => Ok(Value::from(self.day as i32)),
            "timeSlot" => Ok(Value::from(self.time_slot.as_str())),
            "isWeekday" => Ok(Value::from(self.day <= 4)),
            "isWeekend" => Ok(Value::from(self.day >= 5)),
            "hasGameFlag" => {
                let flag = string_arg(method, args, 0)?;
                Ok(Value::from(self.flags.contains(flag.as_str())))
            }
            "arcState" => {
                let arc_id = string_arg(method, args, 0)?;
                let state = self
                    .arc_states
                    .get(arc_id.as_str())
                    .map_or("", |s| s.as_str());
                Ok(Value::from(state))
            }
            _ => Err(Error::new(
                ErrorKind::UnknownMethod,
                format!("gd has no method '{method}'"),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// SceneCtxView
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct SceneCtxView {
    pub flags: HashSet<String>,
}

impl fmt::Display for SceneCtxView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SceneCtxView")
    }
}

impl Object for SceneCtxView {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State<'_, '_>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        match method {
            "hasFlag" => {
                let flag = string_arg(method, args, 0)?;
                Ok(Value::from(self.flags.contains(flag.as_str())))
            }
            _ => Err(Error::new(
                ErrorKind::UnknownMethod,
                format!("scene has no method '{method}'"),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn string_arg(method: &str, args: &[Value], idx: usize) -> Result<String, Error> {
    match args.get(idx) {
        Some(v) => match v.as_str() {
            Some(s) => Ok(s.to_owned()),
            None => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("'{method}' expects a string argument, got {v:?}"),
            )),
        },
        None => Err(Error::new(
            ErrorKind::MissingArgument,
            format!("'{method}' requires a string argument"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Public render function
// ---------------------------------------------------------------------------

pub fn render_prose(
    template_str: &str,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<String, minijinja::Error> {
    // Pre-resolve trait IDs to strings
    let trait_strings: HashSet<String> = world
        .player
        .traits
        .iter()
        .map(|&tid| registry.trait_id_to_str(tid).to_string())
        .collect();

    // Pre-resolve skill IDs to (string → effective value) map
    let skills: HashMap<String, i32> = world
        .player
        .skills
        .iter()
        .map(|(&sid, _sv)| {
            let name = registry.skill_id_to_str(sid).to_string();
            (name, world.player.skill(sid))
        })
        .collect();

    let player_ctx = PlayerCtx {
        trait_strings,
        virgin: world.player.virgin,
        origin: world.player.origin,
        partner: world.player.partner.is_some(),
        on_pill: world.player.on_pill,
        pregnant: world.player.pregnancy.is_some(),
        skills,
        money: world.player.money,
        stress: world.player.stress,
        anxiety: world.player.anxiety,
        arousal: format!("{:?}", world.player.arousal),
        alcohol: format!("{:?}", world.player.alcohol),
    };

    let game_data_ctx = GameDataCtx {
        week: world.game_data.week,
        day: world.game_data.day,
        time_slot: format!("{:?}", world.game_data.time_slot),
        flags: world.game_data.flags.clone(),
        arc_states: world.game_data.arc_states.clone(),
    };

    let scene_view = SceneCtxView {
        flags: ctx.scene_flags.clone(),
    };

    let mut env = minijinja::Environment::new();
    env.add_template("prose", template_str)?;
    let tmpl = env.get_template("prose")?;

    let render_ctx = minijinja::context! {
        w => Value::from_object(player_ctx),
        gd => Value::from_object(game_data_ctx),
        scene => Value::from_object(scene_view),
    };

    tmpl.render(render_ctx)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};

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
                    traits: HashSet::new(),
                }),
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
            .insert("base::robin_opening".to_string(), "working".to_string());

        let ctx = SceneCtx::new();
        let template = r#"{% if gd.arcState("base::robin_opening") == "working" %}on-the-job{% else %}not-started{% endif %}"#;
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
        let template = r#"{% if gd.arcState("base::robin_opening") == "" %}not-started{% else %}started{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(
            result.contains("not-started"),
            "expected 'not-started' in '{result}'"
        );
    }
}
