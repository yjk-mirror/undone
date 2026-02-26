pub mod effects;
pub mod engine;
pub mod loader;
pub mod scheduler;
pub mod template_ctx;
pub mod types;

pub use effects::{apply_effect, EffectError};
pub use engine::{ActionView, EngineCommand, EngineEvent, NpcActivatedData, SceneEngine};
pub use loader::{load_scenes, validate_cross_references, SceneLoadError};
pub use scheduler::{load_schedule, PickResult, Scheduler, SchedulerError};
pub use types::{Action, EffectDef, NextBranch, NpcAction, SceneDefinition, SceneMeta, SceneToml};

#[cfg(test)]
mod integration_tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use lasso::Key;
    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};

    use crate::engine::{EngineCommand, EngineEvent, SceneEngine};
    use crate::loader::load_scenes;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn make_world_with_shy(registry: &undone_packs::PackRegistry) -> World {
        let shy_id = registry.resolve_trait("SHY").unwrap();
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
                traits: {
                    let mut s = HashSet::new();
                    s.insert(shy_id);
                    s
                },
                skills: HashMap::new(),
                money: 100,
                stress: 0,
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

    fn make_male_npc() -> MaleNpc {
        MaleNpc {
            core: NpcCore {
                name: "Stranger".into(),
                age: Age::Thirties,
                race: "white".into(),
                eye_colour: "grey".into(),
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
    fn rain_shelter_full_flow() {
        // 1. Load packs
        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        assert!(!metas.is_empty());

        // 2. Load scenes
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();
        assert!(scenes.contains_key("base::rain_shelter"));

        // 3. Create world with SHY player
        let mut world = make_world_with_shy(&registry);

        // 4. Build engine
        let mut engine = SceneEngine::new(scenes);

        // 5. Start scene
        engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            &mut world,
            &registry,
        );
        let events = engine.drain();

        // 6. Assert intro prose contains shy branch
        let prose_events: Vec<&str> = events
            .iter()
            .filter_map(|e| {
                if let EngineEvent::ProseAdded(p) = e {
                    Some(p.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(!prose_events.is_empty(), "intro prose should be emitted");
        let all_prose = prose_events.join("\n");
        assert!(
            all_prose.contains("far end"),
            "SHY branch should appear in intro"
        );

        // 7. Assert initial actions (main + leave, NOT accept_umbrella yet)
        let actions_event = events
            .iter()
            .find_map(|e| {
                if let EngineEvent::ActionsAvailable(a) = e {
                    Some(a)
                } else {
                    None
                }
            })
            .unwrap();
        let ids: Vec<&str> = actions_event.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"main"), "main should be available");
        assert!(ids.contains(&"leave"), "leave should be available");
        assert!(
            !ids.contains(&"accept_umbrella"),
            "accept_umbrella not available yet"
        );
    }

    #[test]
    fn workplace_arc_full_playthrough() {
        use crate::scheduler::load_schedule;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;

        // Load packs + schedule + scenes
        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();

        // Start world with ROUTE_WORKPLACE flag (what the Robin preset provides)
        let mut world = make_world_with_shy(&registry);
        world.game_data.set_flag("ROUTE_WORKPLACE");

        let mut engine = SceneEngine::new(scenes);
        let mut rng = SmallRng::seed_from_u64(42);

        let mut visited: Vec<String> = Vec::new();
        let mut all_errors: Vec<String> = Vec::new();

        // Simulate the arc from start to settled
        'arc: for _ in 0..30 {
            let Some(pick) = scheduler.pick_next(&world, &registry, &mut rng) else {
                break;
            };

            let scene_id = pick.scene_id.clone();

            // Game loop responsibility: mark once-only scenes as played
            if pick.once_only {
                world.game_data.set_flag(&format!("ONCE_{}", scene_id));
            }

            visited.push(scene_id.clone());

            engine.send(
                EngineCommand::StartScene(scene_id.clone()),
                &mut world,
                &registry,
            );

            // Play through the scene until SceneFinished
            for _ in 0..10 {
                let events = engine.drain();

                for e in &events {
                    if let EngineEvent::ErrorOccurred(msg) = e {
                        all_errors.push(format!("[{}] {}", scene_id, msg));
                    }
                }

                if events
                    .iter()
                    .any(|e| matches!(e, EngineEvent::SceneFinished))
                {
                    break;
                }

                let available = events.iter().find_map(|e| {
                    if let EngineEvent::ActionsAvailable(a) = e {
                        Some(a.clone())
                    } else {
                        None
                    }
                });

                match available {
                    Some(actions) if !actions.is_empty() => {
                        engine.send(
                            EngineCommand::ChooseAction(actions[0].id.clone()),
                            &mut world,
                            &registry,
                        );
                    }
                    _ => break, // no actions and not finished
                }
            }

            if world.game_data.arc_state("base::workplace_opening") == Some("settled") {
                break 'arc;
            }
        }

        let expected = [
            "base::workplace_arrival",
            "base::workplace_landlord",
            "base::workplace_first_night",
            "base::workplace_first_clothes",
            "base::workplace_first_day",
            "base::workplace_work_meeting",
            "base::workplace_evening",
        ];
        for scene in &expected {
            assert!(
                visited.iter().any(|v| v == scene),
                "scene '{}' was never visited; visited order: {:?}",
                scene,
                visited
            );
        }
        assert!(
            all_errors.is_empty(),
            "unexpected errors during arc playthrough: {:?}",
            all_errors
        );
    }

    #[test]
    fn rain_shelter_npc_fires_and_umbrella_becomes_available() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();

        let mut world = make_world_with_shy(&registry);
        let npc_key = world.male_npcs.insert(make_male_npc());
        assert_eq!(
            world.male_npcs[npc_key].core.pc_liking,
            LikingLevel::Neutral
        );

        let mut engine = SceneEngine::new(scenes);

        // Start scene + wire NPC
        engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            &mut world,
            &registry,
        );
        engine.send(EngineCommand::SetActiveMale(npc_key), &mut world, &registry);
        engine.drain();

        // Pick "main" (allow_npc_actions = true) — NPC should fire and set umbrella_offered.
        // The NPC action always fires when condition passes (umbrella not yet offered, weight=10),
        // so after choosing "main" accept_umbrella should be visible.
        engine.send(
            EngineCommand::ChooseAction("main".into()),
            &mut world,
            &registry,
        );
        let events = engine.drain();

        // accept_umbrella should now be visible
        let actions = events
            .iter()
            .find_map(|e| {
                if let EngineEvent::ActionsAvailable(a) = e {
                    Some(a)
                } else {
                    None
                }
            })
            .unwrap();
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            ids.contains(&"accept_umbrella"),
            "accept_umbrella should be visible after NPC fires"
        );

        // Accept umbrella — finish scene
        engine.send(
            EngineCommand::ChooseAction("accept_umbrella".into()),
            &mut world,
            &registry,
        );
        let events = engine.drain();
        assert!(events
            .iter()
            .any(|e| matches!(e, EngineEvent::SceneFinished)));

        // NPC pc_liking should have increased by 1 step (Neutral → Ok)
        assert_eq!(world.male_npcs[npc_key].core.pc_liking, LikingLevel::Ok);
    }
}
