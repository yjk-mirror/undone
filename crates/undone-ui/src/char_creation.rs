use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dropdown::Dropdown;
use floem::views::Checkbox;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;
use undone_domain::{
    Age, Appearance, BeforeIdentity, BeforeSexuality, BeforeVoice, BreastSize, ButtSize,
    ClitSensitivity, Complexion, EyeColour, HairColour, HairLength, Height, InnerLabiaSize,
    LipShape, MaleFigure, NaturalPubicHair, NippleSensitivity, PcOrigin, PenisSize, PlayerFigure,
    PubicHairStyle, SkinTone, TraitId, WaistSize, WetnessBaseline,
};
use undone_packs::{char_creation::CharCreationConfig, PackRegistry};
use undone_scene::scheduler::Scheduler;

use crate::game_state::{start_game, GameState, PreGameState};
use crate::theme::ThemeColors;
use crate::{AppPhase, AppSignals, PartialCharState};

// ── Preset character data ─────────────────────────────────────────────────────

struct PresetData {
    // Identity
    before_name: &'static str,
    before_age: Age,
    origin: PcOrigin,
    before_sexuality: BeforeSexuality,
    before_race: &'static str,
    trait_ids: &'static [&'static str],
    blurb: &'static str,
    /// Starting game flags seeded at game start. Presets use these to opt into
    /// a route; custom players start freeform with no preset flags.
    starting_flags: &'static [&'static str],

    // Before-life physical
    before_figure: MaleFigure,
    before_height: Height,
    before_hair_colour: HairColour,
    before_eye_colour: EyeColour,
    before_skin_tone: SkinTone,
    before_penis_size: PenisSize,
    before_voice: BeforeVoice,

    // After-transformation physical
    age: Age,
    race: &'static str,
    figure: PlayerFigure,
    height: Height,
    breasts: BreastSize,
    butt: ButtSize,
    waist: WaistSize,
    lips: LipShape,
    hair_colour: HairColour,
    hair_length: HairLength,
    eye_colour: EyeColour,
    skin_tone: SkinTone,
    complexion: Complexion,
    appearance: Appearance,
    pubic_hair: PubicHairStyle,
    natural_pubic_hair: NaturalPubicHair,

    // Sexual attributes
    nipple_sensitivity: NippleSensitivity,
    clit_sensitivity: ClitSensitivity,
    inner_labia: InnerLabiaSize,
    wetness_baseline: WetnessBaseline,

    // Names (post-transformation)
    name_fem: &'static str,
    name_masc: &'static str,
}

const PRESET_ROBIN: PresetData = PresetData {
    // Identity
    before_name: "Robin",
    before_age: Age::Thirties,
    origin: PcOrigin::CisMaleTransformed,
    before_sexuality: BeforeSexuality::AttractedToWomen,
    before_race: "White",
    trait_ids: &[
        // Personality
        "AMBITIOUS",
        "ANALYTICAL",
        "DOWN_TO_EARTH",
        "OBJECTIFYING",
        // Physical
        "STRAIGHT_HAIR",
        "SWEET_VOICE",
        "ALMOND_EYES",
        "WIDE_HIPS",
        "NARROW_WAIST",
        "SMALL_HANDS",
        "PRONOUNCED_COLLARBONES",
        "THIGH_GAP",
        "SOFT_SKIN",
        "NATURALLY_SMOOTH",
        "INTOXICATING_SCENT",
        // Sexual response
        "HAIR_TRIGGER",
        "HEAVY_SQUIRTER",
        "MULTI_ORGASMIC",
        "ORAL_FIXATION",
        "SENSITIVE_NECK",
        "SENSITIVE_EARS",
        "SENSITIVE_INNER_THIGHS",
        "SUBMISSIVE",
        "PRAISE_KINK",
        "EASILY_WET",
        "BACK_ARCHER",
        "TOE_CURLER",
        // Arousal response
        "NIPPLE_GETTER",
        "FLUSHER",
        "THIGH_CLENCHER",
        "BREATH_CHANGER",
        "LIP_BITER",
        // Sexual preference
        "LIKES_ORAL_GIVING",
        "LIKES_DOUBLE_PENETRATION",
        // Dark content
        "FREEZE_RESPONSE",
        // Body
        "REGULAR_PERIODS",
    ],
    blurb: "You're thirty-two, a software engineer with ten years of experience. \
            You took a job offer in a city you didn't know — new company, new start, \
            boxes shipped to an apartment you've never seen. When things go sideways, \
            you inventory and solve. You're very good at that.",
    starting_flags: &["ROUTE_WORKPLACE"],

    // Before-life physical (all unremarkable)
    before_figure: MaleFigure::Average,
    before_height: Height::Average,
    before_hair_colour: HairColour::Brown,
    before_eye_colour: EyeColour::Brown,
    before_skin_tone: SkinTone::Light,
    before_penis_size: PenisSize::Average,
    before_voice: BeforeVoice::Average,

    // After physical
    age: Age::LateTeen,
    race: "East Asian",
    figure: PlayerFigure::Petite,
    height: Height::Short,
    breasts: BreastSize::Huge,
    butt: ButtSize::Big,
    waist: WaistSize::Narrow,
    lips: LipShape::Full,
    hair_colour: HairColour::Black,
    hair_length: HairLength::Long,
    eye_colour: EyeColour::DarkBrown,
    skin_tone: SkinTone::Light,
    complexion: Complexion::Glowing,
    appearance: Appearance::Stunning,
    pubic_hair: PubicHairStyle::Bare,
    natural_pubic_hair: NaturalPubicHair::None,

    // Sexual
    nipple_sensitivity: NippleSensitivity::High,
    clit_sensitivity: ClitSensitivity::High,
    inner_labia: InnerLabiaSize::Average,
    wetness_baseline: WetnessBaseline::Wet,

    // Names: Robin keeps the same name (gender-neutral)
    name_fem: "Robin",
    name_masc: "Robin",
};

const PRESET_RAUL: PresetData = PresetData {
    // Identity
    before_name: "Raul",
    before_age: Age::LateTeen,
    origin: PcOrigin::CisMaleTransformed,
    before_sexuality: BeforeSexuality::AttractedToWomen,
    before_race: "Latina",
    trait_ids: &["AMBITIOUS", "CONFIDENT", "OUTGOING", "SEXIST", "HOMOPHOBIC"],
    blurb: "You're eighteen, starting at a university your family has talked about for years. \
            You arrived with your expectations calibrated: you knew who you were, where you \
            were headed, and what the next four years were supposed to look like. \
            Things have always worked out. You've never had a real reason to think they wouldn't.",
    starting_flags: &["ROUTE_CAMPUS"],

    // Before-life physical
    before_figure: MaleFigure::Toned,
    before_height: Height::Tall,
    before_hair_colour: HairColour::Black,
    before_eye_colour: EyeColour::DarkBrown,
    before_skin_tone: SkinTone::Olive,
    before_penis_size: PenisSize::AboveAverage,
    before_voice: BeforeVoice::Average,

    // After physical
    age: Age::LateTeen,
    race: "Latina",
    figure: PlayerFigure::Hourglass,
    height: Height::Average,
    breasts: BreastSize::Full,
    butt: ButtSize::Round,
    waist: WaistSize::Average,
    lips: LipShape::Average,
    hair_colour: HairColour::DarkBrown,
    hair_length: HairLength::Shoulder,
    eye_colour: EyeColour::DarkBrown,
    skin_tone: SkinTone::Olive,
    complexion: Complexion::Normal,
    appearance: Appearance::Attractive,
    pubic_hair: PubicHairStyle::Trimmed,
    natural_pubic_hair: NaturalPubicHair::Full,

    // Sexual
    nipple_sensitivity: NippleSensitivity::Normal,
    clit_sensitivity: ClitSensitivity::Normal,
    inner_labia: InnerLabiaSize::Average,
    wetness_baseline: WetnessBaseline::Normal,

    // Names
    name_fem: "Camila",
    name_masc: "Raul",
};

const CUSTOM_STARTING_TRAIT_IDS: &[&str] = &[
    "SHY",
    "CUTE",
    "POSH",
    "SULTRY",
    "DOWN_TO_EARTH",
    "BITCHY",
    "REFINED",
    "ROMANTIC",
    "FLIRTY",
    "AMBITIOUS",
    "OUTGOING",
    "OVERACTIVE_IMAGINATION",
    "ANALYTICAL",
    "CONFIDENT",
    "SEXIST",
    "HOMOPHOBIC",
    "OBJECTIFYING",
];

#[derive(Clone, Debug, PartialEq)]
struct FemFormDefaults {
    name_fem: String,
    age: Age,
    figure: PlayerFigure,
    breasts: BreastSize,
    race: String,
}

pub fn validate_registry_contract(registry: &PackRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    for trait_id in CUSTOM_STARTING_TRAIT_IDS
        .iter()
        .copied()
        .chain(PRESET_ROBIN.trait_ids.iter().copied())
        .chain(PRESET_RAUL.trait_ids.iter().copied())
    {
        if registry.resolve_trait(trait_id).is_err() {
            errors.push(format!(
                "character creation requires trait '{trait_id}', but it is not registered"
            ));
        }
    }

    if registry.block_rough_trait().is_err() {
        errors.push(
            "character creation requires rough-content opt-out trait 'BLOCK_ROUGH', but it is not registered"
                .to_string(),
        );
    }
    if registry.likes_rough_trait().is_err() {
        errors.push(
            "character creation requires rough-content preference trait 'LIKES_ROUGH', but it is not registered"
                .to_string(),
        );
    }

    errors.sort();
    errors.dedup();
    errors
}

pub fn validate_runtime_contract(registry: &PackRegistry, scheduler: &Scheduler) -> Vec<String> {
    let mut errors = validate_registry_contract(registry);

    for preset in [PRESET_ROBIN, PRESET_RAUL] {
        for flag in preset.starting_flags {
            if !scheduler.references_game_flag(flag) {
                errors.push(format!(
                    "character creation preset '{}' seeds starting flag '{flag}', but the scheduler never references it",
                    preset.before_name
                ));
            }
        }
    }

    errors.sort();
    errors.dedup();
    errors
}

fn preset_by_idx(idx: Option<u8>) -> Option<&'static PresetData> {
    match idx {
        Some(0) => Some(&PRESET_ROBIN),
        Some(1) => Some(&PRESET_RAUL),
        _ => None,
    }
}

fn fem_form_defaults(
    partial: Option<&PartialCharState>,
    fallback_race: Option<&str>,
) -> FemFormDefaults {
    if let Some(preset) = partial.and_then(|partial| preset_by_idx(partial.preset_idx)) {
        return FemFormDefaults {
            name_fem: preset.name_fem.to_string(),
            age: preset.age,
            figure: preset.figure,
            breasts: preset.breasts,
            race: preset.race.to_string(),
        };
    }

    let default_race = partial
        .map(|state| state.before_race.as_str())
        .filter(|race| !race.is_empty())
        .or(fallback_race)
        .unwrap_or("White");
    let default_age = partial
        .map(|state| state.before_age)
        .unwrap_or(Age::EarlyTwenties);

    FemFormDefaults {
        name_fem: "Eva".to_string(),
        age: default_age,
        figure: PlayerFigure::Slim,
        breasts: BreastSize::Full,
        race: default_race.to_string(),
    }
}

/// Build a complete CharCreationConfig for the Robin preset.
/// Used by `--quick` start and later dev tooling entry points.
pub fn robin_quick_config(registry: &PackRegistry) -> CharCreationConfig {
    let starting_traits: Vec<TraitId> = PRESET_ROBIN
        .trait_ids
        .iter()
        .filter_map(|trait_id| registry.resolve_trait(trait_id).ok())
        .collect();

    CharCreationConfig {
        name_fem: PRESET_ROBIN.name_fem.to_string(),
        name_masc: PRESET_ROBIN.name_masc.to_string(),
        age: PRESET_ROBIN.age,
        race: PRESET_ROBIN.race.to_string(),
        figure: PRESET_ROBIN.figure,
        breasts: PRESET_ROBIN.breasts,
        origin: PRESET_ROBIN.origin,
        before: Some(BeforeIdentity {
            name: PRESET_ROBIN.before_name.to_string(),
            age: PRESET_ROBIN.before_age,
            race: PRESET_ROBIN.before_race.to_string(),
            sexuality: PRESET_ROBIN.before_sexuality,
            figure: PRESET_ROBIN.before_figure,
            height: PRESET_ROBIN.before_height,
            hair_colour: PRESET_ROBIN.before_hair_colour,
            eye_colour: PRESET_ROBIN.before_eye_colour,
            skin_tone: PRESET_ROBIN.before_skin_tone,
            penis_size: PRESET_ROBIN.before_penis_size,
            voice: PRESET_ROBIN.before_voice,
            traits: std::collections::HashSet::new(),
        }),
        starting_traits,
        male_count: 6,
        female_count: 3,
        starting_flags: PRESET_ROBIN
            .starting_flags
            .iter()
            .map(|flag| (*flag).to_string())
            .collect(),
        starting_arc_states: std::collections::HashMap::new(),
        height: PRESET_ROBIN.height,
        butt: PRESET_ROBIN.butt,
        waist: PRESET_ROBIN.waist,
        lips: PRESET_ROBIN.lips,
        hair_colour: PRESET_ROBIN.hair_colour,
        hair_length: PRESET_ROBIN.hair_length,
        eye_colour: PRESET_ROBIN.eye_colour,
        skin_tone: PRESET_ROBIN.skin_tone,
        complexion: PRESET_ROBIN.complexion,
        appearance: PRESET_ROBIN.appearance,
        pubic_hair: PRESET_ROBIN.pubic_hair,
        natural_pubic_hair: PRESET_ROBIN.natural_pubic_hair,
        nipple_sensitivity: PRESET_ROBIN.nipple_sensitivity,
        clit_sensitivity: PRESET_ROBIN.clit_sensitivity,
        inner_labia: PRESET_ROBIN.inner_labia,
        wetness_baseline: PRESET_ROBIN.wetness_baseline,
    }
}

// ── PC origin helpers ─────────────────────────────────────────────────────────

fn origin_from_idx(idx: u8) -> PcOrigin {
    match idx {
        0 => PcOrigin::CisMaleTransformed,
        1 => PcOrigin::TransWomanTransformed,
        2 => PcOrigin::CisFemaleTransformed,
        _ => PcOrigin::AlwaysFemale,
    }
}

/// Read the race list from the pack registry, falling back to `["White"]` if empty.
fn read_races(pre_state: &Rc<RefCell<Option<PreGameState>>>) -> Vec<String> {
    if let Some(ref pre) = *pre_state.borrow() {
        if !pre.registry.races().is_empty() {
            return pre.registry.races().to_vec();
        }
    }
    vec!["White".to_string()]
}

/// Read the male names list from the pack registry, falling back to a minimal set.
fn read_male_names(pre_state: &Rc<RefCell<Option<PreGameState>>>) -> Vec<String> {
    if let Some(ref pre) = *pre_state.borrow() {
        if !pre.registry.male_names().is_empty() {
            return pre.registry.male_names().to_vec();
        }
    }
    vec!["Matt".to_string(), "Ryan".to_string(), "David".to_string()]
}

// ── BeforeCreation form signals ───────────────────────────────────────────────

#[derive(Clone, Copy)]
struct BeforeFormSignals {
    origin_idx: RwSignal<u8>,
    before_name: RwSignal<String>,
    before_age: RwSignal<Age>,
    before_sexuality: RwSignal<BeforeSexuality>,
    before_race: RwSignal<String>,
    // personality
    trait_shy: RwSignal<bool>,
    trait_cute: RwSignal<bool>,
    trait_posh: RwSignal<bool>,
    trait_sultry: RwSignal<bool>,
    trait_down_to_earth: RwSignal<bool>,
    trait_bitchy: RwSignal<bool>,
    trait_refined: RwSignal<bool>,
    trait_romantic: RwSignal<bool>,
    trait_flirty: RwSignal<bool>,
    trait_ambitious: RwSignal<bool>,
    trait_outgoing: RwSignal<bool>,
    trait_overactive_imagination: RwSignal<bool>,
    trait_analytical: RwSignal<bool>,
    trait_confident: RwSignal<bool>,
    // attitude traits
    trait_sexist: RwSignal<bool>,
    trait_homophobic: RwSignal<bool>,
    trait_objectifying: RwSignal<bool>,
    appearance: RwSignal<Appearance>,
    // content prefs
    include_rough: RwSignal<bool>,
    likes_rough: RwSignal<bool>,
    // mode: 0=Robin preset, 1=Raul preset, 2=Custom
    char_mode: RwSignal<u8>,
}

impl BeforeFormSignals {
    fn new() -> Self {
        Self {
            origin_idx: RwSignal::new(0),
            before_name: RwSignal::new(String::new()),
            before_age: RwSignal::new(Age::EarlyTwenties),
            before_sexuality: RwSignal::new(BeforeSexuality::AttractedToWomen),
            before_race: RwSignal::new(String::new()),
            trait_shy: RwSignal::new(false),
            trait_cute: RwSignal::new(false),
            trait_posh: RwSignal::new(false),
            trait_sultry: RwSignal::new(false),
            trait_down_to_earth: RwSignal::new(false),
            trait_bitchy: RwSignal::new(false),
            trait_refined: RwSignal::new(false),
            trait_romantic: RwSignal::new(false),
            trait_flirty: RwSignal::new(false),
            trait_ambitious: RwSignal::new(false),
            trait_outgoing: RwSignal::new(false),
            trait_overactive_imagination: RwSignal::new(false),
            trait_analytical: RwSignal::new(false),
            trait_confident: RwSignal::new(false),
            trait_sexist: RwSignal::new(false),
            trait_homophobic: RwSignal::new(false),
            trait_objectifying: RwSignal::new(false),
            appearance: RwSignal::new(Appearance::Average),
            include_rough: RwSignal::new(false),
            likes_rough: RwSignal::new(false),
            char_mode: RwSignal::new(0u8),
        }
    }
}

// ── FemCreation form signals ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct FemFormSignals {
    name_fem: RwSignal<String>,
    age: RwSignal<Age>,
    figure: RwSignal<PlayerFigure>,
    breasts: RwSignal<BreastSize>,
    race: RwSignal<String>,
}

impl FemFormSignals {
    fn from_defaults(defaults: &FemFormDefaults) -> Self {
        Self {
            name_fem: RwSignal::new(defaults.name_fem.clone()),
            age: RwSignal::new(defaults.age),
            figure: RwSignal::new(defaults.figure),
            breasts: RwSignal::new(defaults.breasts),
            race: RwSignal::new(defaults.race.clone()),
        }
    }
}

// ── public entry points ───────────────────────────────────────────────────────

/// BeforeCreation phase: who you were before the transformation.
pub fn char_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    let form = BeforeFormSignals::new();
    let char_mode = form.char_mode;

    let races_list = read_races(&pre_state);
    if let Some(first) = races_list.first() {
        form.before_race.set(first.clone());
    }
    let male_names = read_male_names(&pre_state);

    let next_btn = build_next_button(signals, form, pre_state, game_state, partial_char);

    let races_for_dyn = races_list;
    let names_for_dyn = male_names;
    let main_section = dyn_container(
        move || char_mode.get(),
        move |mode| {
            if mode == 2 {
                v_stack((
                    section_your_past(signals, form, races_for_dyn.clone(), names_for_dyn.clone()),
                    section_personality(signals, form),
                    section_content_prefs(signals, form),
                ))
                .into_any()
            } else {
                let preset: &'static PresetData = if mode == 0 {
                    &PRESET_ROBIN
                } else {
                    &PRESET_RAUL
                };
                section_preset_detail(signals, preset).into_any()
            }
        },
    );

    let content = v_stack((
        heading("Your Story Begins", signals),
        section_preset_select(signals, char_mode),
        main_section,
        next_btn,
        empty().style(|s| s.height(40.0)),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .max_width(640.0)
            .padding_horiz(40.0)
            .padding_vert(32.0)
            .color(colors.ink)
    });

    let centered = container(content).style(|s| s.width_full().flex_row().justify_center());

    scroll(centered)
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.size_full().background(colors.page)
        })
}

/// FemCreation phase: who you are now.
pub fn fem_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
    dev_mode: bool,
) -> impl View {
    let races_list = read_races(&pre_state);
    let partial = partial_char.get_untracked();
    let preset_ref = partial
        .as_ref()
        .and_then(|partial| preset_by_idx(partial.preset_idx));
    let defaults = fem_form_defaults(
        partial.as_ref(),
        races_list.first().map(|race| race.as_str()),
    );
    let form = FemFormSignals::from_defaults(&defaults);

    let is_always_female = partial
        .as_ref()
        .map(|p| p.origin == PcOrigin::AlwaysFemale)
        .unwrap_or(false);

    let begin_btn =
        build_begin_button(signals, form, pre_state, game_state, partial_char, dev_mode);

    let age_row: Box<dyn View> = if let Some(preset) = preset_ref {
        Box::new(read_only_row("Age", preset.age.to_string(), signals))
    } else if is_always_female {
        Box::new(form_row(
            "Age",
            signals,
            Dropdown::new_rw(
                form.age,
                vec![
                    Age::LateTeen,
                    Age::EarlyTwenties,
                    Age::MidLateTwenties,
                    Age::LateTwenties,
                    Age::Thirties,
                    Age::Forties,
                    Age::Fifties,
                    Age::Old,
                ],
            )
            .main_view(themed_trigger::<Age>(signals))
            .list_item_view(themed_item::<Age>(signals))
            .style(field_style(signals)),
        ))
    } else {
        Box::new(empty())
    };

    let names_section: Box<dyn View> = if let Some(preset) = preset_ref {
        Box::new(
            v_stack((
                section_title("Your Name", signals),
                read_only_row("Name", preset.name_fem.to_string(), signals),
            ))
            .style(section_style()),
        )
    } else {
        Box::new(
            v_stack((
                section_title("Your Name", signals),
                form_row(
                    "Name",
                    signals,
                    text_input(form.name_fem)
                        .placeholder("e.g. Eva")
                        .style(field_style(signals)),
                ),
            ))
            .style(section_style()),
        )
    };

    let body_section: Box<dyn View> = if let Some(preset) = preset_ref {
        let physical_traits: Vec<String> = preset
            .trait_ids
            .iter()
            .filter(|id| {
                !PERSONALITY_TRAIT_IDS.contains(id) && BODY_APPEARANCE_TRAIT_IDS.contains(id)
            })
            .map(|id| trait_id_to_display(id))
            .collect();
        let sexual_traits: Vec<String> = preset
            .trait_ids
            .iter()
            .filter(|id| {
                !PERSONALITY_TRAIT_IDS.contains(id) && !BODY_APPEARANCE_TRAIT_IDS.contains(id)
            })
            .map(|id| trait_id_to_display(id))
            .collect();
        let physical_row: Box<dyn View> = if physical_traits.is_empty() {
            Box::new(empty())
        } else {
            Box::new(trait_chips("Physical", physical_traits, signals))
        };
        let sexual_row: Box<dyn View> = if sexual_traits.is_empty() {
            Box::new(empty())
        } else {
            Box::new(trait_chips("Sexual", sexual_traits, signals))
        };
        Box::new(
            v_stack((
                section_title("Your Body", signals),
                read_only_row("Figure", preset.figure.to_string(), signals),
                read_only_row("Breasts", preset.breasts.to_string(), signals),
                physical_row,
                sexual_row,
            ))
            .style(section_style()),
        )
    } else {
        Box::new(
            v_stack((
                section_title("Your Body", signals),
                form_row(
                    "Figure",
                    signals,
                    Dropdown::new_rw(
                        form.figure,
                        vec![
                            PlayerFigure::Petite,
                            PlayerFigure::Slim,
                            PlayerFigure::Athletic,
                            PlayerFigure::Hourglass,
                            PlayerFigure::Curvy,
                            PlayerFigure::Thick,
                            PlayerFigure::Plus,
                        ],
                    )
                    .main_view(themed_trigger::<PlayerFigure>(signals))
                    .list_item_view(themed_item::<PlayerFigure>(signals))
                    .style(field_style(signals)),
                ),
                form_row(
                    "Breasts",
                    signals,
                    Dropdown::new_rw(
                        form.breasts,
                        vec![
                            BreastSize::Flat,
                            BreastSize::Perky,
                            BreastSize::Handful,
                            BreastSize::Average,
                            BreastSize::Full,
                            BreastSize::Big,
                            BreastSize::Huge,
                        ],
                    )
                    .main_view(themed_trigger::<BreastSize>(signals))
                    .list_item_view(themed_item::<BreastSize>(signals))
                    .style(field_style(signals)),
                ),
            ))
            .style(section_style()),
        )
    };

    let background_section: Box<dyn View> = if let Some(preset) = preset_ref {
        Box::new(
            v_stack((
                section_title("Background", signals),
                read_only_row("Race", preset.race.to_string(), signals),
                age_row,
            ))
            .style(section_style()),
        )
    } else {
        Box::new(
            v_stack((
                section_title("Background", signals),
                form_row("Race", signals, race_picker(form.race, races_list, signals)),
                age_row,
            ))
            .style(section_style()),
        )
    };

    let framing_prose = label(move || {
        "Somewhere between Ohio and here, everything changed. You don't remember it. \
         You just woke up and the body was different — the weight, the proportions, \
         the face in the airplane bathroom mirror. You're still you. The rest is new."
            .to_string()
    })
    .style(move |s| {
        let prefs = signals.prefs.get();
        let colors = ThemeColors::from_mode(prefs.mode);
        s.width_full()
            .padding_vert(16.0)
            .padding_horiz(4.0)
            .color(colors.ink_dim)
            .font_size(prefs.font_size as f32 * 0.95)
            .line_height(1.6)
    });

    let content = v_stack((
        heading("Who Are You Now?", signals),
        framing_prose,
        names_section,
        body_section,
        background_section,
        begin_btn,
        empty().style(|s| s.height(40.0)),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .max_width(640.0)
            .padding_horiz(40.0)
            .padding_vert(32.0)
            .color(colors.ink)
    });

    let centered = container(content).style(|s| s.width_full().flex_row().justify_center());

    scroll(centered)
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.size_full().background(colors.page)
        })
}

// ── heading ───────────────────────────────────────────────────────────────────

fn heading(title: &'static str, signals: AppSignals) -> impl View {
    label(move || title.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(28.0)
            .font_weight(floem::text::Weight::LIGHT)
            .color(colors.ink)
            .margin_bottom(32.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

// ── section: Your Past ────────────────────────────────────────────────────────

fn section_your_past(
    signals: AppSignals,
    form: BeforeFormSignals,
    races: Vec<String>,
    male_names: Vec<String>,
) -> impl View {
    let origin_idx = form.origin_idx;

    let origin_radios = v_stack((
        radio_opt(
            "Something happened to me \u{2014} I was a man",
            "Transformed from male. The core experience.",
            move || origin_idx.get() == 0,
            move || origin_idx.set(0),
            signals,
        ),
        radio_opt(
            "Something happened to me \u{2014} I was a trans woman",
            "Already knew yourself. The transformation was recognition.",
            move || origin_idx.get() == 1,
            move || origin_idx.set(1),
            signals,
        ),
        radio_opt(
            "Something happened to me \u{2014} I was a woman",
            "You were female. Something still changed.",
            move || origin_idx.get() == 2,
            move || origin_idx.set(2),
            signals,
        ),
        radio_opt(
            "I was always a woman",
            "No transformation. Play as yourself.",
            move || origin_idx.get() == 3,
            move || origin_idx.set(3),
            signals,
        ),
    ))
    .style(|s| s.margin_bottom(16.0));

    let before_fields = dyn_container(
        move || origin_idx.get(),
        move |idx| {
            if idx == 3 {
                // AlwaysFemale — no before-fields
                return empty().into_any();
            }

            let origin = origin_from_idx(idx);
            let br = races.clone();
            let mn = male_names.clone();
            let hint = mn.first().cloned().unwrap_or_else(|| "Matt".to_string());

            // Name field with Randomize button
            let name_row = {
                let mn_click = mn.clone();
                h_stack((
                    label(move || "Name before".to_string()).style(move |s| {
                        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                        s.width(180.0)
                            .font_size(14.0)
                            .color(colors.ink_dim)
                            .items_center()
                            .font_family("system-ui, -apple-system, sans-serif".to_string())
                    }),
                    text_input(form.before_name)
                        .placeholder(hint)
                        .style(move |s| {
                            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                            s.width(178.0)
                                .height(32.0)
                                .padding_horiz(10.0)
                                .font_size(14.0)
                                .color(colors.ink)
                                .background(colors.page_raised)
                                .border(1.0)
                                .border_color(colors.seam)
                                .border_radius(4.0)
                                .font_family("system-ui, -apple-system, sans-serif".to_string())
                        }),
                    label(|| "Rand".to_string())
                        .keyboard_navigable()
                        .on_click_stop(move |_| {
                            if !mn_click.is_empty() {
                                let idx = rand::random::<usize>() % mn_click.len();
                                form.before_name.set(mn_click[idx].clone());
                            }
                        })
                        .style(move |s| {
                            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                            s.width(40.0)
                                .height(32.0)
                                .margin_left(4.0)
                                .font_size(11.0)
                                .color(colors.ink_dim)
                                .border(1.0)
                                .border_color(colors.seam)
                                .border_radius(4.0)
                                .items_center()
                                .justify_center()
                                .cursor(floem::style::CursorStyle::Pointer)
                        }),
                ))
                .style(|s| s.items_center().margin_bottom(12.0))
            };

            let age_row = form_row(
                "Age before",
                signals,
                Dropdown::new_rw(
                    form.before_age,
                    vec![
                        Age::LateTeen,
                        Age::EarlyTwenties,
                        Age::MidLateTwenties,
                        Age::LateTwenties,
                        Age::Thirties,
                        Age::Forties,
                        Age::Fifties,
                        Age::Old,
                    ],
                )
                .main_view(themed_trigger::<Age>(signals))
                .list_item_view(themed_item::<Age>(signals))
                .style(field_style(signals)),
            );

            let race_row = form_row(
                "Race before",
                signals,
                race_picker(form.before_race, br, signals),
            );

            if origin.was_male_bodied() {
                let sexuality_row = form_row(
                    "Before sexuality",
                    signals,
                    Dropdown::new_rw(
                        form.before_sexuality,
                        vec![
                            BeforeSexuality::AttractedToWomen,
                            BeforeSexuality::AttractedToMen,
                            BeforeSexuality::AttractedToBoth,
                        ],
                    )
                    .main_view(themed_trigger::<BeforeSexuality>(signals))
                    .list_item_view(themed_item::<BeforeSexuality>(signals))
                    .style(field_style(signals)),
                );
                v_stack((name_row, age_row, sexuality_row, race_row)).into_any()
            } else {
                v_stack((name_row, age_row, race_row)).into_any()
            }
        },
    );

    v_stack((
        section_title("Your Past", signals),
        origin_radios,
        before_fields,
    ))
    .style(section_style())
}

// ── section: Personality ──────────────────────────────────────────────────────

fn section_personality(signals: AppSignals, form: BeforeFormSignals) -> impl View {
    let trait_grid = v_stack((
        h_stack((
            trait_checkbox("Shy", form.trait_shy, signals),
            trait_checkbox("Cute", form.trait_cute, signals),
            trait_checkbox("Posh", form.trait_posh, signals),
            trait_checkbox("Sultry", form.trait_sultry, signals),
            trait_checkbox("Down to Earth", form.trait_down_to_earth, signals),
        ))
        .style(|s| {
            s.gap(16.0)
                .margin_bottom(8.0)
                .flex_wrap(floem::style::FlexWrap::Wrap)
        }),
        h_stack((
            trait_checkbox("Bitchy", form.trait_bitchy, signals),
            trait_checkbox("Refined", form.trait_refined, signals),
            trait_checkbox("Romantic", form.trait_romantic, signals),
            trait_checkbox("Flirty", form.trait_flirty, signals),
            trait_checkbox("Ambitious", form.trait_ambitious, signals),
        ))
        .style(|s| {
            s.gap(16.0)
                .margin_bottom(8.0)
                .flex_wrap(floem::style::FlexWrap::Wrap)
        }),
        h_stack((
            trait_checkbox("Outgoing", form.trait_outgoing, signals),
            trait_checkbox(
                "Overactive Imagination",
                form.trait_overactive_imagination,
                signals,
            ),
            trait_checkbox("Analytical", form.trait_analytical, signals),
            trait_checkbox("Confident", form.trait_confident, signals),
        ))
        .style(|s| s.gap(16.0).flex_wrap(floem::style::FlexWrap::Wrap)),
    ));

    let divider = empty().style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.height(1.0)
            .width_full()
            .background(colors.seam)
            .margin_vert(12.0)
    });

    let divider2 = empty().style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.height(1.0)
            .width_full()
            .background(colors.seam)
            .margin_vert(12.0)
    });

    let appearance_row = form_row(
        "Appearance",
        signals,
        Dropdown::new_rw(
            form.appearance,
            vec![
                Appearance::Plain,
                Appearance::Average,
                Appearance::Attractive,
                Appearance::Beautiful,
                Appearance::Stunning,
                Appearance::Devastating,
            ],
        )
        .main_view(themed_trigger::<Appearance>(signals))
        .list_item_view(themed_item::<Appearance>(signals))
        .style(field_style(signals)),
    );

    let attitude_grid = h_stack((
        trait_checkbox("Sexist", form.trait_sexist, signals),
        trait_checkbox("Homophobic", form.trait_homophobic, signals),
        trait_checkbox("Objectifying", form.trait_objectifying, signals),
    ))
    .style(|s| s.gap(16.0).flex_wrap(floem::style::FlexWrap::Wrap));

    v_stack((
        section_title("Personality", signals),
        hint_label("Pick 2\u{2013}3 traits:", signals),
        trait_grid,
        divider,
        appearance_row,
        divider2,
        hint_label("Former attitudes (optional):", signals),
        attitude_grid,
    ))
    .style(section_style())
}

// ── section: Content Preferences ─────────────────────────────────────────────

fn section_content_prefs(signals: AppSignals, form: BeforeFormSignals) -> impl View {
    v_stack((
        section_title("Content Preferences", signals),
        Checkbox::labeled_rw(form.include_rough, || {
            "Include rough / non-con content".to_string()
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.items_center()
                .gap(8.0)
                .font_size(14.0)
                .color(colors.ink)
                .margin_bottom(10.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        Checkbox::labeled_rw(form.likes_rough, || "I enjoy rougher content".to_string()).style(
            move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.items_center()
                    .gap(8.0)
                    .font_size(14.0)
                    .color(colors.ink)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            },
        ),
    ))
    .style(section_style())
}

// ── section: Preset select ────────────────────────────────────────────────────

fn section_preset_select(signals: AppSignals, char_mode: RwSignal<u8>) -> impl View {
    v_stack((
        section_title("Who Were You?", signals),
        radio_opt(
            "Robin",
            "Thirty-two. Software engineer, a decade in. Moved to a city you didn't know.",
            move || char_mode.get() == 0,
            move || char_mode.set(0),
            signals,
        ),
        radio_opt(
            "Raul",
            "Eighteen. Starting university. Expectations carefully calibrated.",
            move || char_mode.get() == 1,
            move || char_mode.set(1),
            signals,
        ),
        radio_opt(
            "Create your own",
            "Customize your origin, name, traits, and background.",
            move || char_mode.get() == 2,
            move || char_mode.set(2),
            signals,
        ),
    ))
    .style(section_style())
}

// ── section: Preset detail ────────────────────────────────────────────────────

/// Trait IDs that describe the character's personality (shown on the "Who Were You?" screen).
/// Body, sexual, arousal, and other post-transformation traits are NOT shown here.
const PERSONALITY_TRAIT_IDS: &[&str] = &[
    "SHY",
    "CUTE",
    "POSH",
    "SULTRY",
    "DOWN_TO_EARTH",
    "BITCHY",
    "REFINED",
    "ROMANTIC",
    "FLIRTY",
    "AMBITIOUS",
    "OUTGOING",
    "OVERACTIVE_IMAGINATION",
    "ANALYTICAL",
    "CONFIDENT",
    "SEXIST",
    "HOMOPHOBIC",
    "OBJECTIFYING",
];

/// Physical appearance trait IDs shown under "Physical" in the body section.
/// Non-personality traits not in this list appear under "Sexual".
const BODY_APPEARANCE_TRAIT_IDS: &[&str] = &[
    "STRAIGHT_HAIR",
    "WAVY_HAIR",
    "CURLY_HAIR",
    "SWEET_VOICE",
    "SMOKY_VOICE",
    "HUSKY_VOICE",
    "ALMOND_EYES",
    "WIDE_EYES",
    "HOODED_EYES",
    "WIDE_HIPS",
    "NARROW_HIPS",
    "NARROW_WAIST",
    "SMALL_HANDS",
    "LONG_LEGS",
    "LONG_NECK",
    "PRONOUNCED_COLLARBONES",
    "THIGH_GAP",
    "SOFT_SKIN",
    "NATURALLY_SMOOTH",
    "INTOXICATING_SCENT",
    "REGULAR_PERIODS",
    "IRREGULAR_PERIODS",
];

fn section_preset_detail(signals: AppSignals, preset: &'static PresetData) -> impl View {
    let personality_traits: Vec<String> = preset
        .trait_ids
        .iter()
        .filter(|id| PERSONALITY_TRAIT_IDS.contains(id))
        .map(|id| trait_id_to_display(id))
        .collect();

    v_stack((
        label(move || preset.blurb.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .color(colors.ink)
                .margin_bottom(20.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        read_only_row("Name", preset.before_name.to_string(), signals),
        read_only_row("Age", preset.before_age.to_string(), signals),
        read_only_row("Race", preset.before_race.to_string(), signals),
        read_only_row("Build", preset.before_figure.to_string(), signals),
        read_only_row("Height", preset.before_height.to_string(), signals),
        read_only_row("Hair", preset.before_hair_colour.to_string(), signals),
        read_only_row("Eyes", preset.before_eye_colour.to_string(), signals),
        read_only_row("Skin tone", preset.before_skin_tone.to_string(), signals),
        read_only_row("Voice", preset.before_voice.to_string(), signals),
        read_only_row("Penis size", preset.before_penis_size.to_string(), signals),
        trait_chips("Personality", personality_traits, signals),
    ))
    .style(|s| s.flex_col().width_full().margin_bottom(24.0))
}

// ── Next button ───────────────────────────────────────────────────────────────

fn build_next_button(
    signals: AppSignals,
    form: BeforeFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    label(|| "Next \u{2192}".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let char_mode = form.char_mode.get_untracked();

            let origin: PcOrigin;
            let before_name: String;
            let before_age: Age;
            let before_race: String;
            let before_sexuality: BeforeSexuality;
            let trait_names: Vec<&'static str>;

            let preset_ref: Option<&'static PresetData>;
            if char_mode < 2 {
                // Preset mode — Robin (0) or Raul (1)
                let preset: &'static PresetData = if char_mode == 0 {
                    &PRESET_ROBIN
                } else {
                    &PRESET_RAUL
                };
                preset_ref = Some(preset);
                origin = preset.origin;
                before_name = preset.before_name.to_string();
                before_age = preset.before_age;
                before_race = preset.before_race.to_string();
                before_sexuality = preset.before_sexuality;
                trait_names = preset.trait_ids.to_vec();
            } else {
                // Custom mode
                preset_ref = None;
                let origin_idx = form.origin_idx.get_untracked();
                if origin_idx != 3 && form.before_name.get_untracked().trim().is_empty() {
                    return;
                }
                origin = origin_from_idx(origin_idx);
                before_name = form.before_name.get_untracked();
                before_age = form.before_age.get_untracked();
                before_race = form.before_race.get_untracked();
                before_sexuality = form.before_sexuality.get_untracked();

                let mut tn: Vec<&'static str> = Vec::new();
                if form.trait_shy.get_untracked() {
                    tn.push("SHY");
                }
                if form.trait_cute.get_untracked() {
                    tn.push("CUTE");
                }
                if form.trait_posh.get_untracked() {
                    tn.push("POSH");
                }
                if form.trait_sultry.get_untracked() {
                    tn.push("SULTRY");
                }
                if form.trait_down_to_earth.get_untracked() {
                    tn.push("DOWN_TO_EARTH");
                }
                if form.trait_bitchy.get_untracked() {
                    tn.push("BITCHY");
                }
                if form.trait_refined.get_untracked() {
                    tn.push("REFINED");
                }
                if form.trait_romantic.get_untracked() {
                    tn.push("ROMANTIC");
                }
                if form.trait_flirty.get_untracked() {
                    tn.push("FLIRTY");
                }
                if form.trait_ambitious.get_untracked() {
                    tn.push("AMBITIOUS");
                }
                if form.trait_outgoing.get_untracked() {
                    tn.push("OUTGOING");
                }
                if form.trait_overactive_imagination.get_untracked() {
                    tn.push("OVERACTIVE_IMAGINATION");
                }
                if form.trait_analytical.get_untracked() {
                    tn.push("ANALYTICAL");
                }
                if form.trait_confident.get_untracked() {
                    tn.push("CONFIDENT");
                }
                if form.trait_sexist.get_untracked() {
                    tn.push("SEXIST");
                }
                if form.trait_homophobic.get_untracked() {
                    tn.push("HOMOPHOBIC");
                }
                if form.trait_objectifying.get_untracked() {
                    tn.push("OBJECTIFYING");
                }
                trait_names = tn;
            }

            // Resolve trait IDs from registry
            let mut starting_traits: Vec<_> = {
                let pre_borrow = pre_state.borrow();
                if let Some(ref pre) = *pre_borrow {
                    trait_names
                        .iter()
                        .map(|name| {
                            pre.registry.resolve_trait(name).unwrap_or_else(|_| {
                                panic!(
                                    "character creation trait '{name}' must be validated during init"
                                )
                            })
                        })
                        .collect()
                } else {
                    vec![]
                }
            };
            if let Some(ref pre) = *pre_state.borrow() {
                if !form.include_rough.get_untracked() {
                    starting_traits.push(pre.registry.block_rough_trait().unwrap_or_else(|_| {
                        panic!("character creation trait 'BLOCK_ROUGH' must be validated during init")
                    }));
                }
                if form.likes_rough.get_untracked() {
                    starting_traits.push(pre.registry.likes_rough_trait().unwrap_or_else(|_| {
                        panic!("character creation trait 'LIKES_ROUGH' must be validated during init")
                    }));
                }
            }

            // Presets declare their own starting game flags; custom players
            // start freeform with no preset routing flags.
            let starting_flags = preset_ref
                .map(|preset| {
                    preset
                        .starting_flags
                        .iter()
                        .map(|flag| (*flag).to_string())
                        .collect()
                })
                .unwrap_or_default();

            let appearance = if let Some(p) = preset_ref {
                p.appearance
            } else {
                form.appearance.get_untracked()
            };
            let partial = PartialCharState {
                origin,
                before_name: before_name.clone(),
                before_age,
                before_race: before_race.clone(),
                before_sexuality,
                starting_traits,
                starting_flags,
                preset_idx: preset_ref.map(|_| char_mode),
                appearance,
            };
            partial_char.set(Some(partial.clone()));

            if origin == PcOrigin::AlwaysFemale {
                // Skip transformation intro — go straight to fem creation.
                signals.phase.set(AppPhase::FemCreation);
            } else {
                // Create a throwaway world for the transformation intro scene.
                // This world is discarded after the intro — the real world is
                // created at FemCreation submit via new_game().
                let before_identity = if let Some(p) = preset_ref {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: p.before_figure,
                        height: p.before_height,
                        hair_colour: p.before_hair_colour,
                        eye_colour: p.before_eye_colour,
                        skin_tone: p.before_skin_tone,
                        penis_size: p.before_penis_size,
                        voice: p.before_voice,
                        traits: std::collections::HashSet::new(),
                    })
                } else {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: MaleFigure::Average,
                        height: Height::Average,
                        hair_colour: HairColour::DarkBrown,
                        eye_colour: EyeColour::Brown,
                        skin_tone: SkinTone::Medium,
                        penis_size: PenisSize::Average,
                        voice: BeforeVoice::Average,
                        traits: std::collections::HashSet::new(),
                    })
                };
                let throwaway_config = if let Some(p) = preset_ref {
                    CharCreationConfig {
                        name_fem: p.name_fem.to_string(),
                        name_masc: p.name_masc.to_string(),
                        age: p.age,
                        race: p.race.to_string(),
                        figure: p.figure,
                        breasts: p.breasts,
                        origin,
                        before: before_identity,
                        starting_traits: partial.starting_traits.clone(),
                        male_count: 0,
                        female_count: 0,
                        starting_flags: partial.starting_flags.iter().cloned().collect(),
                        starting_arc_states: std::collections::HashMap::new(),
                        height: p.height,
                        butt: p.butt,
                        waist: p.waist,
                        lips: p.lips,
                        hair_colour: p.hair_colour,
                        hair_length: p.hair_length,
                        eye_colour: p.eye_colour,
                        skin_tone: p.skin_tone,
                        complexion: p.complexion,
                        appearance: p.appearance,
                        pubic_hair: p.pubic_hair,
                        natural_pubic_hair: p.natural_pubic_hair,
                        nipple_sensitivity: p.nipple_sensitivity,
                        clit_sensitivity: p.clit_sensitivity,
                        inner_labia: p.inner_labia,
                        wetness_baseline: p.wetness_baseline,
                    }
                } else {
                    CharCreationConfig {
                        name_fem: String::new(),
                        name_masc: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        figure: PlayerFigure::Slim,
                        breasts: BreastSize::Full,
                        origin,
                        before: before_identity,
                        starting_traits: partial.starting_traits.clone(),
                        male_count: 0,
                        female_count: 0,
                        starting_flags: partial.starting_flags.iter().cloned().collect(),
                        starting_arc_states: std::collections::HashMap::new(),
                        height: Height::Average,
                        butt: ButtSize::Round,
                        waist: WaistSize::Average,
                        lips: LipShape::Average,
                        hair_colour: HairColour::DarkBrown,
                        hair_length: HairLength::Shoulder,
                        eye_colour: EyeColour::Brown,
                        skin_tone: SkinTone::Medium,
                        complexion: Complexion::Normal,
                        appearance: Appearance::Average,
                        pubic_hair: PubicHairStyle::Trimmed,
                        natural_pubic_hair: NaturalPubicHair::Full,
                        nipple_sensitivity: NippleSensitivity::Normal,
                        clit_sensitivity: ClitSensitivity::Normal,
                        inner_labia: InnerLabiaSize::Average,
                        wetness_baseline: WetnessBaseline::Normal,
                    }
                };

                {
                    let mut pre_mut = pre_state.borrow_mut();
                    if let Some(ref mut pre) = *pre_mut {
                        let throwaway_world = undone_packs::char_creation::new_game(
                            throwaway_config,
                            &mut pre.registry,
                            &mut pre.rng,
                        );
                        let engine = undone_scene::engine::SceneEngine::new(pre.scenes.clone());
                        let femininity_id = pre
                            .registry
                            .femininity_skill()
                            .expect("PackRegistry must include required skill id FEMININITY");
                        let throwaway_gs = GameState {
                            world: throwaway_world,
                            registry: pre.registry.clone(),
                            engine,
                            scheduler: pre.scheduler.clone(),
                            rng: rand::rngs::SmallRng::from_entropy(),
                            dev_mode: false,
                            init_error: None,
                            opening_scene: pre.registry.opening_scene().map(|s| s.to_owned()),
                            femininity_id,
                            current_scene_time_anchor: None,
                        };
                        *game_state.borrow_mut() = Some(throwaway_gs);
                    }
                }

                signals.phase.set(AppPhase::TransformationIntro);
            }
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.margin_top(24.0)
                .padding_horiz(40.0)
                .padding_vert(16.0)
                .font_size(16.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .border(1.5)
                .border_color(colors.lamp)
                .border_radius(6.0)
                .color(colors.lamp)
                .background(colors.lamp_glow)
                .hover(|s| s.background(colors.lamp_glow))
                .focus_visible(|s| s.outline(2.0).outline_color(colors.lamp))
                .active(|s| s.background(colors.lamp_glow))
        })
}

// ── Begin Your Story button (FemCreation) ─────────────────────────────────────

fn build_begin_button(
    signals: AppSignals,
    form: FemFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
    dev_mode: bool,
) -> impl View {
    label(|| "Begin Your Story".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let pre = match pre_state.borrow_mut().take() {
                Some(p) => p,
                None => return, // already started
            };

            let partial = partial_char.get_untracked().unwrap_or_else(|| {
                // AlwaysFemale path: partial may not carry before-data.
                PartialCharState {
                    origin: PcOrigin::AlwaysFemale,
                    before_name: String::new(),
                    before_age: form.age.get_untracked(),
                    before_race: form.race.get_untracked(),
                    before_sexuality: BeforeSexuality::AttractedToWomen,
                    starting_traits: vec![],
                    starting_flags: vec![],
                    preset_idx: None,
                    appearance: Appearance::Average,
                }
            });

            let origin = partial.origin;

            // Resolve preset reference (if any) so we can pull physical attributes
            let preset_ref = preset_by_idx(partial.preset_idx);

            let config = if let Some(p) = preset_ref {
                // Preset mode: all physical/sexual attributes come from PresetData
                let before = if origin.was_transformed() {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: p.before_figure,
                        height: p.before_height,
                        hair_colour: p.before_hair_colour,
                        eye_colour: p.before_eye_colour,
                        skin_tone: p.before_skin_tone,
                        penis_size: p.before_penis_size,
                        voice: p.before_voice,
                        traits: std::collections::HashSet::new(),
                    })
                } else {
                    None
                };
                CharCreationConfig {
                    name_fem: p.name_fem.to_string(),
                    name_masc: p.name_masc.to_string(),
                    age: p.age,
                    race: p.race.to_string(),
                    figure: p.figure,
                    breasts: p.breasts,
                    origin,
                    before,
                    starting_traits: partial.starting_traits,
                    male_count: 6,
                    female_count: 2,
                    starting_flags: partial.starting_flags.into_iter().collect(),
                    starting_arc_states: std::collections::HashMap::new(),
                    height: p.height,
                    butt: p.butt,
                    waist: p.waist,
                    lips: p.lips,
                    hair_colour: p.hair_colour,
                    hair_length: p.hair_length,
                    eye_colour: p.eye_colour,
                    skin_tone: p.skin_tone,
                    complexion: p.complexion,
                    appearance: p.appearance,
                    pubic_hair: p.pubic_hair,
                    natural_pubic_hair: p.natural_pubic_hair,
                    nipple_sensitivity: p.nipple_sensitivity,
                    clit_sensitivity: p.clit_sensitivity,
                    inner_labia: p.inner_labia,
                    wetness_baseline: p.wetness_baseline,
                }
            } else {
                // Custom mode: form signals + defaults for unexposed fields
                let fem_race = form.race.get_untracked();
                let before = if origin.was_transformed() {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: MaleFigure::Average,
                        height: Height::Average,
                        hair_colour: HairColour::DarkBrown,
                        eye_colour: EyeColour::Brown,
                        skin_tone: SkinTone::Medium,
                        penis_size: PenisSize::Average,
                        voice: BeforeVoice::Average,
                        traits: std::collections::HashSet::new(),
                    })
                } else {
                    None
                };
                let pc_age = if origin == PcOrigin::AlwaysFemale {
                    form.age.get_untracked()
                } else {
                    partial.before_age
                };
                CharCreationConfig {
                    name_fem: form.name_fem.get_untracked(),
                    name_masc: partial.before_name.clone(),
                    age: pc_age,
                    race: fem_race,
                    figure: form.figure.get_untracked(),
                    breasts: form.breasts.get_untracked(),
                    origin,
                    before,
                    starting_traits: partial.starting_traits,
                    male_count: 6,
                    female_count: 2,
                    starting_flags: partial.starting_flags.into_iter().collect(),
                    starting_arc_states: std::collections::HashMap::new(),
                    height: Height::Average,
                    butt: ButtSize::Round,
                    waist: WaistSize::Average,
                    lips: LipShape::Average,
                    hair_colour: HairColour::DarkBrown,
                    hair_length: HairLength::Shoulder,
                    eye_colour: EyeColour::Brown,
                    skin_tone: SkinTone::Medium,
                    complexion: Complexion::Normal,
                    appearance: partial.appearance,
                    pubic_hair: PubicHairStyle::Trimmed,
                    natural_pubic_hair: NaturalPubicHair::Full,
                    nipple_sensitivity: NippleSensitivity::Normal,
                    clit_sensitivity: ClitSensitivity::Normal,
                    inner_labia: InnerLabiaSize::Average,
                    wetness_baseline: WetnessBaseline::Normal,
                }
            };

            let gs = start_game(pre, config, dev_mode);
            *game_state.borrow_mut() = Some(gs);
            signals.tab.set(crate::AppTab::Game);
            signals.phase.set(AppPhase::InGame);
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.margin_top(24.0)
                .padding_horiz(40.0)
                .padding_vert(16.0)
                .font_size(16.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .border(1.5)
                .border_color(colors.lamp)
                .border_radius(6.0)
                .color(colors.lamp)
                .background(colors.lamp_glow)
                .hover(|s| s.background(colors.lamp_glow))
                .focus_visible(|s| s.outline(2.0).outline_color(colors.lamp))
                .active(|s| s.background(colors.lamp_glow))
        })
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn section_title(title: &'static str, signals: AppSignals) -> impl View {
    label(move || title.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .font_weight(floem::text::Weight::SEMIBOLD)
            .color(colors.ink_ghost)
            .margin_bottom(16.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn hint_label(text: &'static str, signals: AppSignals) -> impl View {
    label(move || text.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .color(colors.ink_dim)
            .margin_bottom(12.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn form_row(label_text: &'static str, signals: AppSignals, input: impl IntoView) -> impl View {
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(180.0)
                .font_size(14.0)
                .color(colors.ink_dim)
                .items_center()
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        input.into_view(),
    ))
    .style(|s| s.items_center().margin_bottom(12.0))
}

fn read_only_row(label_text: &'static str, value: String, signals: AppSignals) -> impl View {
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.min_width(180.0)
                .width(180.0)
                .font_size(14.0)
                .color(colors.ink_dim)
                .items_center()
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        label(move || value.clone()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .flex_basis(0.0)
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .max_width(400.0)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(|s| s.items_start().margin_bottom(12.0).max_width(600.0))
}

fn trait_id_to_display(id: &str) -> String {
    let s = id.to_lowercase().replace('_', " ");
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn trait_chips(label_text: &'static str, traits: Vec<String>, signals: AppSignals) -> impl View {
    let traits_sig = RwSignal::new(traits);
    let chips = dyn_stack(
        move || traits_sig.get(),
        |t| t.clone(),
        move |t| {
            label(move || t.clone()).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.padding_horiz(8.0)
                    .padding_vert(3.0)
                    .margin_right(4.0)
                    .margin_bottom(4.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .border_color(colors.seam)
                    .color(colors.ink_ghost)
                    .font_size(12.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            })
        },
    )
    .style(|s| {
        s.flex_row()
            .flex_wrap(floem::style::FlexWrap::Wrap)
            .flex_basis(0.0)
            .flex_grow(1.0)
    });
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.min_width(180.0)
                .width(180.0)
                .font_size(14.0)
                .color(colors.ink_dim)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        chips,
    ))
    .style(|s| s.items_start().margin_bottom(12.0).max_width(600.0))
}

fn trait_checkbox(name: &'static str, sig: RwSignal<bool>, signals: AppSignals) -> impl View {
    Checkbox::labeled_rw(sig, move || name.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.items_center()
            .gap(6.0)
            .font_size(14.0)
            .color(colors.ink)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn radio_opt(
    opt_label: &'static str,
    subtitle: &'static str,
    is_active: impl Fn() -> bool + Copy + 'static,
    on_select: impl Fn() + Copy + 'static,
    signals: AppSignals,
) -> impl View {
    let indicator = empty().style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        let bg = if is_active() {
            colors.lamp
        } else {
            colors.page_raised
        };
        let border_col = if is_active() {
            colors.lamp
        } else {
            colors.seam
        };
        s.width(13.0)
            .height(13.0)
            .border_radius(7.0)
            .border(1.5)
            .border_color(border_col)
            .background(bg)
            .margin_right(8.0)
            .margin_top(3.0)
            .flex_shrink(0.0)
    });
    let text_col = v_stack((
        label(move || opt_label.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        label(move || subtitle.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(12.0)
                .color(colors.ink_dim)
                .margin_top(2.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ));
    h_stack((indicator, text_col))
        .style(|s| {
            s.items_start()
                .cursor(floem::style::CursorStyle::Pointer)
                .margin_bottom(10.0)
        })
        .on_click_stop(move |_| on_select())
}

fn race_picker(selection: RwSignal<String>, races: Vec<String>, signals: AppSignals) -> impl View {
    let races_signal = RwSignal::new(races);
    dyn_stack(
        move || races_signal.get(),
        |r| r.clone(),
        move |race| {
            let race_for_cmp = race.clone();
            let race_for_set = race.clone();
            let is_sel = move || selection.get() == race_for_cmp;
            let set_race = move || selection.set(race_for_set.clone());
            label(move || race.clone())
                .on_click_stop(move |_| set_race())
                .style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    let selected = is_sel();
                    s.padding_horiz(12.0)
                        .padding_vert(6.0)
                        .margin_right(4.0)
                        .margin_bottom(4.0)
                        .border(1.0)
                        .border_radius(4.0)
                        .font_size(14.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                        .cursor(floem::style::CursorStyle::Pointer)
                        .border_color(if selected { colors.lamp } else { colors.seam })
                        .color(if selected { colors.lamp } else { colors.ink })
                        .background(if selected {
                            colors.lamp_glow
                        } else {
                            floem::peniko::Color::TRANSPARENT
                        })
                })
        },
    )
    .style(|s| s.flex_row().flex_wrap(floem::style::FlexWrap::Wrap))
}

fn section_style() -> impl Fn(floem::style::Style) -> floem::style::Style {
    |s| {
        s.flex_col()
            .width_full()
            .margin_bottom(32.0)
            .padding_bottom(24.0)
    }
}

/// Returns a closure suitable for `Dropdown::list_item_view` that renders each item
/// with the current theme's ink color and page_raised background.
fn themed_item<T: std::fmt::Display + 'static>(
    signals: AppSignals,
) -> impl Fn(T) -> floem::AnyView {
    move |item| {
        let s = item.to_string();
        label(move || s.clone())
            .style(move |style| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                style
                    .color(colors.ink)
                    .background(colors.page_raised)
                    .padding_horiz(10.0)
                    .padding_vert(6.0)
                    .font_size(14.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            })
            .into_any()
    }
}

/// Returns a closure suitable for `Dropdown::main_view` that renders the selected item
/// with the current theme's ink color — used to fix text-invisible bug in Night theme
/// (floem's default_main_view uses unstyled `text()` which doesn't inherit ink color).
fn themed_trigger<T: std::fmt::Display + 'static>(
    signals: AppSignals,
) -> impl Fn(T) -> floem::AnyView {
    move |item| {
        let s = item.to_string();
        label(move || s.clone())
            .style(move |style| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                style
                    .color(colors.ink)
                    .font_size(14.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            })
            .into_any()
    }
}

/// Shared style for text inputs and dropdowns — same dimensions, border, and font.
fn field_style(signals: AppSignals) -> impl Fn(floem::style::Style) -> floem::style::Style {
    move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width(220.0)
            .height(32.0)
            .padding_horiz(10.0)
            .font_size(14.0)
            .color(colors.ink)
            .background(colors.page_raised)
            .border(1.0)
            .border_color(colors.seam)
            .border_radius(4.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use undone_packs::load_packs;
    use undone_scene::scheduler::load_schedule;

    #[test]
    fn validate_registry_contract_reports_missing_traits() {
        let registry = PackRegistry::new();
        let errors = validate_registry_contract(&registry);
        assert!(
            errors.iter().any(|error| error.contains("BLOCK_ROUGH")),
            "expected missing trait error, got: {:?}",
            errors
        );
    }

    #[test]
    fn fem_form_defaults_use_preset_values_when_present() {
        let partial = PartialCharState {
            origin: PcOrigin::CisMaleTransformed,
            before_name: "Robin".into(),
            before_age: Age::Thirties,
            before_race: "White".into(),
            before_sexuality: BeforeSexuality::AttractedToWomen,
            starting_traits: vec![],
            starting_flags: vec!["ROUTE_WORKPLACE".into()],
            preset_idx: Some(0),
            appearance: Appearance::Average,
        };

        let defaults = fem_form_defaults(Some(&partial), Some("White"));
        assert_eq!(defaults.name_fem, "Robin");
        assert_eq!(defaults.figure, PRESET_ROBIN.figure);
        assert_eq!(defaults.breasts, PRESET_ROBIN.breasts);
        assert_eq!(defaults.race, PRESET_ROBIN.race);
    }

    #[test]
    fn fem_form_defaults_use_camila_name_for_raul_preset() {
        // preset_idx=1 → PRESET_RAUL → name_fem should be "Camila", not "Eva"
        let partial = PartialCharState {
            origin: PcOrigin::CisMaleTransformed,
            before_name: "Raul".into(),
            before_age: Age::LateTeen,
            before_race: "Latina".into(),
            before_sexuality: BeforeSexuality::AttractedToWomen,
            starting_traits: vec![],
            starting_flags: vec!["ROUTE_CAMPUS".into()],
            preset_idx: Some(1),
            appearance: Appearance::Average,
        };

        let defaults = fem_form_defaults(Some(&partial), Some("White"));
        assert_eq!(defaults.name_fem, "Camila");
        assert_eq!(defaults.figure, PRESET_RAUL.figure);
        assert_eq!(defaults.breasts, PRESET_RAUL.breasts);
        assert_eq!(defaults.race, PRESET_RAUL.race);
    }

    #[test]
    fn fem_form_defaults_fall_back_to_before_race_for_custom_mode() {
        let partial = PartialCharState {
            origin: PcOrigin::CisMaleTransformed,
            before_name: "Evan".into(),
            before_age: Age::EarlyTwenties,
            before_race: "Latina".into(),
            before_sexuality: BeforeSexuality::AttractedToWomen,
            starting_traits: vec![],
            starting_flags: vec![],
            preset_idx: None,
            appearance: Appearance::Average,
        };

        let defaults = fem_form_defaults(Some(&partial), Some("White"));
        assert_eq!(defaults.name_fem, "Eva");
        assert_eq!(defaults.race, "Latina");
        assert_eq!(defaults.age, Age::EarlyTwenties);
    }

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    #[test]
    fn validate_runtime_contract_accepts_base_pack_routes() {
        let (registry, metas) = load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas, &registry).unwrap();

        let errors = validate_runtime_contract(&registry, &scheduler);

        assert!(
            errors.is_empty(),
            "expected no runtime contract errors, got: {:?}",
            errors
        );
    }

    #[test]
    fn robin_quick_config_builds_workplace_preset() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();

        let config = robin_quick_config(&registry);

        assert_eq!(config.name_fem, PRESET_ROBIN.name_fem);
        assert_eq!(config.name_masc, PRESET_ROBIN.name_masc);
        assert!(config.starting_flags.contains("ROUTE_WORKPLACE"));
        assert_eq!(config.male_count, 6);
        assert_eq!(config.female_count, 3);
        assert_eq!(config.appearance, PRESET_ROBIN.appearance);
        assert_eq!(config.starting_traits.len(), PRESET_ROBIN.trait_ids.len());
    }

    /// Physical/body traits from PRESET_ROBIN must NOT appear in BeforeCreation's
    /// personality display — they are post-transformation attributes.
    #[test]
    fn before_creation_personality_display_excludes_physical_traits() {
        let physical_traits = [
            "STRAIGHT_HAIR",
            "SWEET_VOICE",
            "ALMOND_EYES",
            "WIDE_HIPS",
            "NARROW_WAIST",
            "SMALL_HANDS",
        ];
        for id in physical_traits {
            assert!(
                !PERSONALITY_TRAIT_IDS.contains(&id),
                "Physical trait '{}' must not be in PERSONALITY_TRAIT_IDS",
                id
            );
        }
    }

    /// Robin's personality display (BeforeCreation) must only contain personality traits —
    /// filtering on PERSONALITY_TRAIT_IDS must exclude all physical/sexual traits.
    #[test]
    fn robin_preset_personality_display_excludes_body_and_sexual_traits() {
        let displayed: Vec<&str> = PRESET_ROBIN
            .trait_ids
            .iter()
            .copied()
            .filter(|id| PERSONALITY_TRAIT_IDS.contains(id))
            .collect();

        assert!(
            !displayed.is_empty(),
            "Robin should have personality traits to display"
        );

        let non_personality = [
            "STRAIGHT_HAIR",
            "SWEET_VOICE",
            "ALMOND_EYES",
            "WIDE_HIPS",
            "NARROW_WAIST",
            "SMALL_HANDS",
            "HAIR_TRIGGER",
            "HEAVY_SQUIRTER",
            "MULTI_ORGASMIC",
            "REGULAR_PERIODS",
        ];
        for id in displayed {
            assert!(
                !non_personality.contains(&id),
                "Non-personality trait '{}' incorrectly appears in BeforeCreation display",
                id
            );
        }
    }
}
