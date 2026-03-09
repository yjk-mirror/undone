use std::collections::HashMap;

use rand::{rngs::SmallRng, SeedableRng};
use undone_packs::PackRegistry;
use undone_world::World;

use crate::scheduler::Scheduler;

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

const DOMINANT_THRESHOLD: f64 = 12.0;
const RARE_THRESHOLD: f64 = 1.0;

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

    for _ in 0..config.runs {
        let mut world = base_world.clone();
        for _ in 0..config.weeks {
            for _ in 0..28 {
                // 4 slots/day × 7 days/week — pick once per slot, matching real gameplay
                if let Some(result) = scheduler.pick_next(&world, registry, &mut rng) {
                    *scene_counts.entry(result.scene_id.clone()).or_insert(0) += 1;
                    total_picks += 1;
                    if result.once_only {
                        world
                            .game_data
                            .set_flag(format!("ONCE_{}", result.scene_id));
                    }
                }
                world.game_data.advance_time_slot();
            }
        }
    }

    SimulationResult {
        scene_counts,
        total_picks,
        runs: config.runs,
        weeks: config.weeks,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::scheduler::ScheduleEvent;
    use undone_domain::*;
    use undone_world::{GameData, World};

    use super::*;

    fn make_world() -> World {
        World {
            player: Player {
                name_fem: "Eva".into(),
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
                traits: HashSet::new(),
                skills: HashMap::new(),
                money: 100,
                stress: BoundedStat::new(0),
                anxiety: BoundedStat::new(0),
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
            male_npcs: slotmap::SlotMap::with_key(),
            female_npcs: slotmap::SlotMap::with_key(),
            game_data: GameData::default(),
        }
    }

    #[test]
    fn simulation_counts_scene_frequencies() {
        let scheduler = Scheduler::from_slots_for_tests(HashMap::from([(
            "free_time".to_string(),
            vec![
                ScheduleEvent {
                    scene: "test::a".into(),
                    condition: None,
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
                ScheduleEvent {
                    scene: "test::b".into(),
                    condition: None,
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
            ],
        )]));

        let result = simulate(
            &scheduler,
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
                ScheduleEvent {
                    scene: "test::reachable".into(),
                    condition: None,
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
                ScheduleEvent {
                    scene: "test::never".into(),
                    condition: Some(undone_expr::parse("false").unwrap()),
                    weight: 10,
                    once_only: false,
                    trigger: None,
                },
            ],
        )]));

        let result = simulate(
            &scheduler,
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
}
