use std::{collections::HashSet, fmt, sync::Arc};

use minijinja::{
    value::{Object, ObjectRepr, Value},
    Error, ErrorKind, State,
};
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
    pub always_female: bool,
    pub partner: bool, // true = has partner (i.e. NOT single)
    pub on_pill: bool,
    pub pregnant: bool,
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
            "alwaysFemale" => Ok(Value::from(self.always_female)),
            "isSingle" => Ok(Value::from(!self.partner)),
            "isOnPill" => Ok(Value::from(self.on_pill)),
            "isPregnant" => Ok(Value::from(self.pregnant)),
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
    pub flags: HashSet<String>,
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
            "hasGameFlag" => {
                let flag = string_arg(method, args, 0)?;
                Ok(Value::from(self.flags.contains(flag.as_str())))
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

    let player_ctx = PlayerCtx {
        trait_strings,
        virgin: world.player.virgin,
        always_female: world.player.always_female,
        partner: world.player.partner.is_some(),
        on_pill: world.player.on_pill,
        pregnant: world.player.pregnancy.is_some(),
    };

    let game_data_ctx = GameDataCtx {
        week: world.game_data.week,
        flags: world.game_data.flags.clone(),
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
                name: "Eva".into(),
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
                always_female: false,
                femininity: 10,
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
    fn scene_hasFlag_in_template() {
        let registry = undone_packs::PackRegistry::new();
        let world = make_world();
        let mut ctx = SceneCtx::new();
        ctx.set_flag("umbrella_offered");

        let template = r#"{% if scene.hasFlag("umbrella_offered") %}yes{% else %}no{% endif %}"#;
        let result = render_prose(template, &world, &ctx, &registry).unwrap();
        assert!(result.contains("yes"), "expected 'yes' in '{result}'");
    }
}
