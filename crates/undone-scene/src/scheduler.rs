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
}

fn default_weight() -> u32 {
    10
}

// ---------------------------------------------------------------------------
// Parsed runtime types
// ---------------------------------------------------------------------------

struct ScheduleEvent {
    scene: String,
    condition: Option<Expr>,
    weight: u32,
}

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

/// Weighted scene selector. Holds all slot definitions loaded from packs.
/// Use `load_schedule` to build from pack metadata, then call `pick` each week.
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
    /// world state, performs weighted random selection, and returns the scene ID.
    /// Returns `None` if the slot is unknown or no events pass their conditions.
    pub fn pick(
        &self,
        slot_name: &str,
        world: &World,
        registry: &PackRegistry,
        rng: &mut impl Rng,
    ) -> Option<String> {
        let events = self.slots.get(slot_name)?;

        // Empty SceneCtx — scheduler conditions have no scene-local state.
        let ctx = SceneCtx::new();

        let eligible: Vec<&ScheduleEvent> = events
            .iter()
            .filter(|e| {
                e.weight > 0
                    && match &e.condition {
                        Some(expr) => eval(expr, world, &ctx, registry).unwrap_or(false),
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
            .map(|e| e.scene.clone())
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
                entry.push(ScheduleEvent {
                    scene: ev.scene,
                    condition,
                    weight: ev.weight,
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
                before_age: 30,
                before_race: "white".into(),
                before_sexuality: Sexuality::StraightMale,
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
                always_female: false,
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
            },
            ScheduleEvent {
                scene: "test::scene_b".into(),
                condition: None,
                weight: 5,
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
}
