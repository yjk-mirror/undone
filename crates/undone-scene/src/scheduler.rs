use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rand::Rng;
use serde::Deserialize;
use thiserror::Error;
use undone_expr::{eval, parse, Expr, SceneCtx};
use undone_packs::{LoadedPackMeta, PackRegistry};
use undone_world::World;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("toml parse error in {path}: {message}")]
    Toml { path: PathBuf, message: String },
    #[error("expression parse error in condition '{condition}': {message}")]
    ExprParse { condition: String, message: String },
}

// ---------------------------------------------------------------------------
// TOML deserialization types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ScheduleFileToml {
    #[serde(default)]
    slot: Vec<ScheduleSlotToml>,
}

#[derive(Debug, Deserialize)]
struct ScheduleSlotToml {
    name: String,
    #[serde(default)]
    events: Vec<ScheduleEventToml>,
}

#[derive(Debug, Deserialize)]
struct ScheduleEventToml {
    scene: String,
    #[serde(default)]
    condition: Option<String>,
    #[serde(default = "default_weight")]
    weight: u32,
    #[serde(default)]
    once_only: bool,
    #[serde(default)]
    trigger: Option<String>,
}

fn default_weight() -> u32 {
    10
}

// ---------------------------------------------------------------------------
// Parsed runtime types
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ScheduleEvent {
    scene: String,
    condition: Option<Expr>,
    weight: u32,
    once_only: bool,
    trigger: Option<Expr>,
}

// ---------------------------------------------------------------------------
// Public result types
// ---------------------------------------------------------------------------

/// The result of a successful `pick` or `check_triggers` call.
#[derive(Debug, Clone, PartialEq)]
pub struct PickResult {
    pub scene_id: String,
    pub once_only: bool,
}

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

/// Weighted scene selector. Holds all slot definitions loaded from packs.
/// Use `load_schedule` to build from pack metadata, then call `pick` each week.
#[derive(Clone)]
pub struct Scheduler {
    /// slot_name → list of events
    slots: HashMap<String, Vec<ScheduleEvent>>,
}

impl Scheduler {
    /// Create an empty scheduler with no slots. Used as a fallback when
    /// pack loading fails.
    pub fn empty() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    /// Return the names of all defined slots.
    pub fn slot_names(&self) -> impl Iterator<Item = &str> {
        self.slots.keys().map(|s| s.as_str())
    }

    /// Pick a scene for the given slot. Evaluates conditions against the current
    /// world state, performs weighted random selection, and returns a `PickResult`.
    /// Returns `None` if the slot is unknown or no events pass their conditions.
    /// Once-only events that have already fired (flag `ONCE_<scene_id>` set) are excluded.
    pub fn pick(
        &self,
        slot_name: &str,
        world: &World,
        registry: &PackRegistry,
        rng: &mut impl Rng,
    ) -> Option<PickResult> {
        let events = self.slots.get(slot_name)?;

        // Empty SceneCtx — scheduler conditions have no scene-local state.
        let ctx = SceneCtx::new();

        let eligible: Vec<&ScheduleEvent> = events
            .iter()
            .filter(|e| {
                e.weight > 0
                    && !(e.once_only && world.game_data.has_flag(&format!("ONCE_{}", e.scene)))
                    && match &e.condition {
                        Some(expr) => match eval(expr, world, &ctx, registry) {
                            Ok(val) => val,
                            Err(err) => {
                                eprintln!(
                                    "[scheduler] condition error in slot '{}', scene '{}': {}",
                                    slot_name, e.scene, err
                                );
                                false
                            }
                        },
                        None => true,
                    }
            })
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let total: u32 = eligible.iter().map(|e| e.weight).sum();
        let mut roll = rng.gen_range(0..total);
        eligible
            .iter()
            .find(|e| {
                if roll < e.weight {
                    true
                } else {
                    roll -= e.weight;
                    false
                }
            })
            .map(|e| PickResult {
                scene_id: e.scene.clone(),
                once_only: e.once_only,
            })
    }

    /// Find the first triggered event in `slot_name` whose trigger condition evaluates to true.
    /// Triggered events are not subject to weighted random selection — the first match wins.
    /// Once-only events that have already fired (flag `ONCE_<scene_id>` set) are excluded.
    pub fn check_triggers(
        &self,
        slot_name: &str,
        world: &World,
        registry: &PackRegistry,
    ) -> Option<PickResult> {
        let events = self.slots.get(slot_name)?;
        let ctx = SceneCtx::new();

        events
            .iter()
            .find(|e| {
                !(e.once_only && world.game_data.has_flag(&format!("ONCE_{}", e.scene)))
                    && match &e.trigger {
                        Some(expr) => match eval(expr, world, &ctx, registry) {
                            Ok(val) => val,
                            Err(err) => {
                                eprintln!(
                                    "[scheduler] trigger error in slot '{}', scene '{}': {}",
                                    slot_name, e.scene, err
                                );
                                false
                            }
                        },
                        None => false,
                    }
            })
            .map(|e| PickResult {
                scene_id: e.scene.clone(),
                once_only: e.once_only,
            })
    }
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Build a `Scheduler` from all packs that define a `schedule_file`.
/// Multiple packs may contribute events to the same slot names.
pub fn load_schedule(pack_metas: &[LoadedPackMeta]) -> Result<Scheduler, SchedulerError> {
    let mut slots: HashMap<String, Vec<ScheduleEvent>> = HashMap::new();

    for meta in pack_metas {
        let schedule_path = match &meta.manifest.content.schedule_file {
            Some(rel) => meta.pack_dir.join(rel),
            None => continue,
        };

        if !schedule_path.exists() {
            continue;
        }

        let src = read_file(&schedule_path)?;
        let file: ScheduleFileToml = toml::from_str(&src).map_err(|e| SchedulerError::Toml {
            path: schedule_path.clone(),
            message: e.to_string(),
        })?;

        for slot_toml in file.slot {
            let entry = slots.entry(slot_toml.name).or_default();
            for ev in slot_toml.events {
                let condition = match ev.condition {
                    Some(ref src) => Some(parse(src).map_err(|e| SchedulerError::ExprParse {
                        condition: src.clone(),
                        message: e.to_string(),
                    })?),
                    None => None,
                };
                let trigger = match ev.trigger {
                    Some(ref src) => Some(parse(src).map_err(|e| SchedulerError::ExprParse {
                        condition: src.clone(),
                        message: e.to_string(),
                    })?),
                    None => None,
                };
                entry.push(ScheduleEvent {
                    scene: ev.scene,
                    condition,
                    weight: ev.weight,
                    once_only: ev.once_only,
                    trigger,
                });
            }
        }
    }

    Ok(Scheduler { slots })
}

fn read_file(path: &Path) -> Result<String, SchedulerError> {
    std::fs::read_to_string(path).map_err(|e| SchedulerError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};

    use super::*;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn make_world() -> World {
        World {
            player: Player {
                name_fem: "Eva".into(),
                name_androg: "Ev".into(),
                name_masc: "Evan".into(),
                before: Some(BeforeIdentity {
                    name: "Evan".into(),
                    age: Age::Twenties,
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

    #[test]
    fn loads_base_pack_schedule() {
        let (_, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas).unwrap();
        let slot_names: Vec<&str> = scheduler.slot_names().collect();
        assert!(
            slot_names.contains(&"free_time"),
            "base pack should define free_time slot"
        );
    }

    #[test]
    fn pick_returns_scene_from_eligible_events() {
        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas).unwrap();
        let world = make_world(); // week = 0
        let mut rng = SmallRng::seed_from_u64(42);

        // week() == 0 and condition is "gd.week() > 0", so no events pass
        let result = scheduler.pick("free_time", &world, &registry, &mut rng);
        assert!(
            result.is_none(),
            "week 0 should not pass the 'gd.week() > 0' condition"
        );
    }

    #[test]
    fn pick_returns_scene_at_week_1() {
        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas).unwrap();
        let mut world = make_world();
        world.game_data.week = 1; // now week() > 0 passes
        let mut rng = SmallRng::seed_from_u64(42);

        let result = scheduler.pick("free_time", &world, &registry, &mut rng);
        assert!(
            result.is_some(),
            "a free_time scene should be picked at week 1"
        );
    }

    #[test]
    fn pick_returns_none_for_unknown_slot() {
        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas).unwrap();
        let world = make_world();
        let mut rng = SmallRng::seed_from_u64(42);

        let result = scheduler.pick("nonexistent_slot", &world, &registry, &mut rng);
        assert!(result.is_none(), "unknown slot should return None");
    }

    #[test]
    fn pick_handles_zero_weight_events() {
        let (registry, _metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let event = ScheduleEvent {
            scene: "test::scene".into(),
            condition: None,
            weight: 0,
            once_only: false,
            trigger: None,
        };
        let mut slots = HashMap::new();
        slots.insert("test_slot".into(), vec![event]);
        let scheduler = Scheduler { slots };
        let world = make_world();
        let mut rng = SmallRng::seed_from_u64(42);

        let result = scheduler.pick("test_slot", &world, &registry, &mut rng);
        assert!(
            result.is_none(),
            "zero-weight event should never be selected"
        );
    }

    #[test]
    fn pick_weighted_selection_is_consistent_with_seed() {
        // Build a scheduler with two events of equal weight
        let (registry, _metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let events = vec![
            ScheduleEvent {
                scene: "test::scene_a".into(),
                condition: None,
                weight: 5,
                once_only: false,
                trigger: None,
            },
            ScheduleEvent {
                scene: "test::scene_b".into(),
                condition: None,
                weight: 5,
                once_only: false,
                trigger: None,
            },
        ];
        let mut slots = HashMap::new();
        slots.insert("test_slot".into(), events);
        let scheduler = Scheduler { slots };
        let world = make_world();

        // Same seed → same pick
        let mut rng1 = SmallRng::seed_from_u64(99);
        let mut rng2 = SmallRng::seed_from_u64(99);
        let r1 = scheduler.pick("test_slot", &world, &registry, &mut rng1);
        let r2 = scheduler.pick("test_slot", &world, &registry, &mut rng2);
        assert_eq!(r1, r2, "same seed should yield same selection");
        assert!(r1.is_some(), "should pick a scene");
    }

    #[test]
    fn empty_scheduler_returns_none_for_any_slot() {
        let scheduler = Scheduler::empty();
        let registry = PackRegistry::new();
        let world = make_world();
        let mut rng = SmallRng::seed_from_u64(42);
        assert!(scheduler
            .pick("anything", &world, &registry, &mut rng)
            .is_none());
    }

    #[test]
    fn once_only_event_filtered_after_flag_set() {
        let registry = PackRegistry::new();
        let event = ScheduleEvent {
            scene: "test::once_scene".into(),
            condition: None,
            weight: 10,
            once_only: true,
            trigger: None,
        };
        let mut slots = HashMap::new();
        slots.insert("test_slot".into(), vec![event]);
        let scheduler = Scheduler { slots };

        // Before flag is set — should pick the scene
        let mut world = make_world();
        let mut rng = SmallRng::seed_from_u64(42);
        let result = scheduler.pick("test_slot", &world, &registry, &mut rng);
        assert!(
            result.is_some(),
            "once_only event should be eligible before flag is set"
        );

        // Set the ONCE_ flag
        world.game_data.set_flag("ONCE_test::once_scene");

        // After flag is set — should be filtered out
        let mut rng2 = SmallRng::seed_from_u64(42);
        let result2 = scheduler.pick("test_slot", &world, &registry, &mut rng2);
        assert!(
            result2.is_none(),
            "once_only event should be excluded after ONCE_ flag is set"
        );
    }

    #[test]
    fn pick_result_includes_once_only() {
        let registry = PackRegistry::new();
        let event = ScheduleEvent {
            scene: "test::flagged_scene".into(),
            condition: None,
            weight: 10,
            once_only: true,
            trigger: None,
        };
        let mut slots = HashMap::new();
        slots.insert("test_slot".into(), vec![event]);
        let scheduler = Scheduler { slots };
        let world = make_world();
        let mut rng = SmallRng::seed_from_u64(42);

        let result = scheduler
            .pick("test_slot", &world, &registry, &mut rng)
            .unwrap();
        assert!(
            result.once_only,
            "PickResult.once_only should be true for a once_only event"
        );
        assert_eq!(result.scene_id, "test::flagged_scene");
    }

    #[test]
    fn check_triggers_returns_scene_when_condition_true() {
        let registry = PackRegistry::new();
        // trigger on "true" — always fires
        let trigger_expr = undone_expr::parse("true").unwrap();
        let event = ScheduleEvent {
            scene: "test::triggered_scene".into(),
            condition: None,
            weight: 0,
            once_only: false,
            trigger: Some(trigger_expr),
        };
        let mut slots = HashMap::new();
        slots.insert("test_slot".into(), vec![event]);
        let scheduler = Scheduler { slots };
        let world = make_world();

        let result = scheduler.check_triggers("test_slot", &world, &registry);
        assert!(
            result.is_some(),
            "triggered event with true condition should be returned"
        );
        assert_eq!(result.unwrap().scene_id, "test::triggered_scene");
    }

    #[test]
    fn check_triggers_returns_none_when_condition_false() {
        let registry = PackRegistry::new();
        // trigger on "false" — never fires
        let trigger_expr = undone_expr::parse("false").unwrap();
        let event = ScheduleEvent {
            scene: "test::triggered_scene".into(),
            condition: None,
            weight: 0,
            once_only: false,
            trigger: Some(trigger_expr),
        };
        let mut slots = HashMap::new();
        slots.insert("test_slot".into(), vec![event]);
        let scheduler = Scheduler { slots };
        let world = make_world();

        let result = scheduler.check_triggers("test_slot", &world, &registry);
        assert!(
            result.is_none(),
            "triggered event with false condition should not fire"
        );
    }

    #[test]
    fn check_triggers_filtered_by_once_only_flag() {
        let registry = PackRegistry::new();
        // trigger on "true" — would fire, but once_only and flag already set
        let trigger_expr = undone_expr::parse("true").unwrap();
        let event = ScheduleEvent {
            scene: "test::once_trigger_scene".into(),
            condition: None,
            weight: 0,
            once_only: true,
            trigger: Some(trigger_expr),
        };
        let mut slots = HashMap::new();
        slots.insert("test_slot".into(), vec![event]);
        let scheduler = Scheduler { slots };

        // Set the ONCE_ flag so the event is filtered out
        let mut world = make_world();
        world.game_data.set_flag("ONCE_test::once_trigger_scene");

        let result = scheduler.check_triggers("test_slot", &world, &registry);
        assert!(
            result.is_none(),
            "once_only triggered event should be excluded after ONCE_ flag is set"
        );
    }
}
