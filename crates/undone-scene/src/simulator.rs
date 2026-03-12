use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rand::{rngs::SmallRng, SeedableRng};
use undone_domain::TimeSlot;
use undone_packs::PackRegistry;
use undone_world::World;

use crate::engine::{EngineCommand, EngineEvent, SceneEngine};
use crate::scheduler::{PickResult, Scheduler};
use crate::types::SceneDefinition;

pub struct SimulationConfig {
    pub weeks: u32,
    pub runs: u32,
    pub seed: u64,
}

pub struct SimulationResult {
    pub scene_counts: HashMap<String, u64>,
    pub total_picks: u64,
    pub runs: u32,
    pub weeks: u32,
}

pub struct SceneStats {
    pub scene_id: String,
    pub count: u64,
    pub percentage: f64,
    pub avg_per_run: f64,
    pub warning: Option<String>,
}

#[derive(Clone, Copy)]
struct SceneTimeAnchor {
    week: u32,
    day: u8,
    time_slot: TimeSlot,
}

impl SceneTimeAnchor {
    fn capture(world: &World) -> Self {
        Self {
            week: world.game_data.week,
            day: world.game_data.day,
            time_slot: world.game_data.time_slot,
        }
    }

    fn matches_world(self, world: &World) -> bool {
        self.week == world.game_data.week
            && self.day == world.game_data.day
            && self.time_slot == world.game_data.time_slot
    }
}

const DOMINANT_THRESHOLD: f64 = 12.0;
const RARE_THRESHOLD: f64 = 1.0;
const MAX_RUNTIME_STEPS_PER_RUN: usize = 4_096;

impl SimulationResult {
    pub fn stats(&self) -> Vec<SceneStats> {
        let mut stats: Vec<SceneStats> = self
            .scene_counts
            .iter()
            .map(|(scene_id, &count)| {
                let percentage = if self.total_picks > 0 {
                    (count as f64 / self.total_picks as f64) * 100.0
                } else {
                    0.0
                };
                let avg_per_run = if self.runs > 0 {
                    count as f64 / self.runs as f64
                } else {
                    0.0
                };
                let warning = if count == 0 {
                    Some("NEVER FIRES".to_string())
                } else if percentage > DOMINANT_THRESHOLD {
                    Some("DOMINANT".to_string())
                } else if percentage < RARE_THRESHOLD {
                    Some("RARE".to_string())
                } else {
                    None
                };
                SceneStats {
                    scene_id: scene_id.clone(),
                    count,
                    percentage,
                    avg_per_run,
                    warning,
                }
            })
            .collect();
        stats.sort_by(|left, right| right.count.cmp(&left.count));
        stats
    }
}

pub fn simulate(
    scheduler: &Scheduler,
    scenes: &HashMap<String, Arc<SceneDefinition>>,
    registry: &PackRegistry,
    base_world: &World,
    config: SimulationConfig,
) -> SimulationResult {
    let mut rng = SmallRng::seed_from_u64(config.seed);
    let mut scene_counts: HashMap<String, u64> = scheduler
        .all_scene_ids()
        .into_iter()
        .map(|scene_id| (scene_id, 0))
        .collect();
    let mut total_picks = 0u64;
    let target_week = base_world.game_data.week + config.weeks;

    for _ in 0..config.runs {
        let mut world = base_world.clone();
        let mut engine = SceneEngine::new(scenes.clone());
        let mut tried_actions: HashSet<(String, String)> = HashSet::new();

        let Some((mut pending_events, mut current_scene_time_anchor)) = start_global_scene(
            scheduler,
            registry,
            &mut world,
            &mut rng,
            &mut engine,
            &mut scene_counts,
            &mut total_picks,
        ) else {
            continue;
        };
        for _ in 0..MAX_RUNTIME_STEPS_PER_RUN {
            if world.game_data.week >= target_week {
                break;
            }

            if let Some(slot_name) = requested_slot(&pending_events) {
                tried_actions.clear();
                if let Some((events, scene_time_anchor)) = start_slot_scene(
                    scheduler,
                    registry,
                    &mut world,
                    &mut rng,
                    &mut engine,
                    &mut scene_counts,
                    &mut total_picks,
                    &slot_name,
                ) {
                    pending_events = events;
                    current_scene_time_anchor = scene_time_anchor;
                    continue;
                }

                consume_scene_time(&mut world, &mut current_scene_time_anchor);
                let Some((events, scene_time_anchor)) = start_global_scene(
                    scheduler,
                    registry,
                    &mut world,
                    &mut rng,
                    &mut engine,
                    &mut scene_counts,
                    &mut total_picks,
                ) else {
                    break;
                };
                pending_events = events;
                current_scene_time_anchor = scene_time_anchor;
                continue;
            }

            if scene_finished(&pending_events) {
                tried_actions.clear();
                consume_scene_time(&mut world, &mut current_scene_time_anchor);
                if world.game_data.week >= target_week {
                    break;
                }

                let Some((events, scene_time_anchor)) = start_global_scene(
                    scheduler,
                    registry,
                    &mut world,
                    &mut rng,
                    &mut engine,
                    &mut scene_counts,
                    &mut total_picks,
                ) else {
                    break;
                };
                pending_events = events;
                current_scene_time_anchor = scene_time_anchor;
                continue;
            }

            let Some(actions) = visible_actions(&pending_events) else {
                break;
            };
            if actions.is_empty() {
                break;
            }

            let scene_id = engine
                .current_scene_id()
                .unwrap_or_else(|| "<no-scene>".to_string());
            let action_id = actions
                .iter()
                .find(|action| tried_actions.insert((scene_id.clone(), action.id.clone())))
                .unwrap_or(&actions[0])
                .id
                .clone();
            pending_events = engine.advance_with_action(&action_id, &mut world, registry);
        }
    }

    SimulationResult {
        scene_counts,
        total_picks,
        runs: config.runs,
        weeks: config.weeks,
    }
}

fn start_global_scene(
    scheduler: &Scheduler,
    registry: &PackRegistry,
    world: &mut World,
    rng: &mut SmallRng,
    engine: &mut SceneEngine,
    scene_counts: &mut HashMap<String, u64>,
    total_picks: &mut u64,
) -> Option<(Vec<EngineEvent>, Option<SceneTimeAnchor>)> {
    let pick = scheduler.pick_next(world, registry, rng)?;
    Some(start_scheduler_scene(
        pick,
        registry,
        world,
        engine,
        scene_counts,
        total_picks,
    ))
}

fn start_slot_scene(
    scheduler: &Scheduler,
    registry: &PackRegistry,
    world: &mut World,
    rng: &mut SmallRng,
    engine: &mut SceneEngine,
    scene_counts: &mut HashMap<String, u64>,
    total_picks: &mut u64,
    slot_name: &str,
) -> Option<(Vec<EngineEvent>, Option<SceneTimeAnchor>)> {
    let pick = scheduler.pick(slot_name, world, registry, rng)?;
    Some(start_scheduler_scene(
        pick,
        registry,
        world,
        engine,
        scene_counts,
        total_picks,
    ))
}

fn start_scheduler_scene(
    pick: PickResult,
    registry: &PackRegistry,
    world: &mut World,
    engine: &mut SceneEngine,
    scene_counts: &mut HashMap<String, u64>,
    total_picks: &mut u64,
) -> (Vec<EngineEvent>, Option<SceneTimeAnchor>) {
    *scene_counts.entry(pick.scene_id.clone()).or_insert(0) += 1;
    *total_picks += 1;
    if pick.once_only {
        world.game_data.set_flag(format!("ONCE_{}", pick.scene_id));
    }

    let scene_time_anchor = pick.consumes_time.then(|| SceneTimeAnchor::capture(world));
    engine.send(EngineCommand::StartScene(pick.scene_id), world, registry);
    (engine.drain(), scene_time_anchor)
}

fn requested_slot(events: &[EngineEvent]) -> Option<String> {
    events.iter().find_map(|event| {
        if let EngineEvent::SlotRequested(slot_name) = event {
            Some(slot_name.clone())
        } else {
            None
        }
    })
}

fn scene_finished(events: &[EngineEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, EngineEvent::SceneFinished))
}

fn visible_actions(events: &[EngineEvent]) -> Option<Vec<crate::engine::ActionView>> {
    events.iter().find_map(|event| {
        if let EngineEvent::ActionsAvailable(actions) = event {
            Some(actions.clone())
        } else {
            None
        }
    })
}

fn consume_scene_time(world: &mut World, current_scene_time_anchor: &mut Option<SceneTimeAnchor>) {
    let should_advance =
        current_scene_time_anchor.is_some_and(|anchor| anchor.matches_world(world));
    *current_scene_time_anchor = None;
    if should_advance {
        world.game_data.advance_time_slot();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::loader::load_scenes;
    use crate::scheduler::load_schedule;
    use lasso::Key;
    use undone_domain::{
        Age, AlcoholLevel, ArousalLevel, AttractionLevel, Behaviour, LikingLevel, LoveLevel,
        MaleClothing, MaleFigure, MaleNpc, NpcCore, PersonalityId, RelationshipStatus,
    };
    use undone_packs::load_packs;
    use undone_world::test_helpers::make_test_world as make_world;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn simple_scene(id: &str) -> Arc<SceneDefinition> {
        Arc::new(SceneDefinition {
            id: id.into(),
            pack: "test".into(),
            intro_prose: "Test scene.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![],
            npc_actions: vec![],
        })
    }

    fn make_scheduler_scene_map(ids: &[&str]) -> HashMap<String, Arc<SceneDefinition>> {
        ids.iter()
            .map(|id| ((*id).to_string(), simple_scene(id)))
            .collect()
    }

    fn make_male_npc() -> MaleNpc {
        MaleNpc {
            core: NpcCore {
                name: "Marcus".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "blue".into(),
                hair_colour: "brown".into(),
                personality: PersonalityId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
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
    fn simulation_counts_scene_frequencies() {
        let scheduler = Scheduler::from_slots_for_tests(HashMap::from([(
            "free_time".to_string(),
            vec![
                crate::scheduler::ScheduleEvent {
                    scene: "test::a".into(),
                    condition: None,
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
                crate::scheduler::ScheduleEvent {
                    scene: "test::b".into(),
                    condition: None,
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
            ],
        )]));
        let scenes = make_scheduler_scene_map(&["test::a", "test::b"]);

        let result = simulate(
            &scheduler,
            &scenes,
            &PackRegistry::new(),
            &make_world(),
            SimulationConfig {
                weeks: 10,
                runs: 100,
                seed: 7,
            },
        );

        let count_a = result.scene_counts["test::a"];
        let count_b = result.scene_counts["test::b"];
        assert!(count_a > 0);
        assert!(count_b > 0);
        assert!((count_a as i64 - count_b as i64).abs() < 250);
    }

    #[test]
    fn simulation_detects_never_fires() {
        let scheduler = Scheduler::from_slots_for_tests(HashMap::from([(
            "free_time".to_string(),
            vec![
                crate::scheduler::ScheduleEvent {
                    scene: "test::reachable".into(),
                    condition: None,
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
                crate::scheduler::ScheduleEvent {
                    scene: "test::never".into(),
                    condition: Some(undone_expr::parse("false").unwrap()),
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
            ],
        )]));
        let scenes = make_scheduler_scene_map(&["test::reachable", "test::never"]);

        let result = simulate(
            &scheduler,
            &scenes,
            &PackRegistry::new(),
            &make_world(),
            SimulationConfig {
                weeks: 8,
                runs: 20,
                seed: 42,
            },
        );

        assert_eq!(result.scene_counts["test::never"], 0);
        let stats = result.stats();
        assert!(stats
            .iter()
            .any(|stat| stat.scene_id == "test::never"
                && stat.warning.as_deref() == Some("NEVER FIRES")));
    }

    #[test]
    fn simulation_can_reach_follow_up_scenes_that_depend_on_runtime_progression() {
        let (registry, metas) = load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas, &registry).unwrap();

        let mut scenes = HashMap::new();
        for meta in &metas {
            let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
            scenes.extend(load_scenes(&scene_dir, &registry).unwrap());
        }

        let mut world = make_world();
        world.game_data.set_flag("ROUTE_WORKPLACE");
        world.male_npcs.insert(make_male_npc());

        let result = simulate(
            &scheduler,
            &scenes,
            &registry,
            &world,
            SimulationConfig {
                weeks: 4,
                runs: 3,
                seed: 42,
            },
        );

        assert!(
            result.scene_counts["base::workplace_landlord"] > 0,
            "runtime-driven simulation should reach workplace_landlord, got {:?}",
            result.scene_counts
        );
        assert!(
            result.scene_counts["base::workplace_first_night"] > 0,
            "runtime-driven simulation should reach workplace_first_night, got {:?}",
            result.scene_counts
        );
    }
}
