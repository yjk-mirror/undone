//! Load-time validation contracts: registry, runtime, and startup invariants.
use undone_domain::PcOrigin;
use undone_packs::PackRegistry;
use undone_scene::scheduler::Scheduler;

use super::config::CUSTOM_STARTING_TRAIT_IDS;

pub fn validate_registry_contract(registry: &PackRegistry) -> Vec<String> {
    let mut errors = Vec::new();

    let preset_trait_ids: Vec<&str> = registry
        .presets()
        .iter()
        .flat_map(|preset| preset.trait_ids.iter().map(|s| s.as_str()))
        .collect();

    for trait_id in CUSTOM_STARTING_TRAIT_IDS
        .iter()
        .copied()
        .chain(preset_trait_ids)
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

    for preset in registry.presets() {
        for flag in &preset.starting_flags {
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

pub fn validate_startup_contract(registry: &PackRegistry, origin: PcOrigin) -> Vec<String> {
    let mut errors = Vec::new();

    if registry.femininity_skill().is_err() {
        errors.push(
            "character creation requires skill 'FEMININITY', but it is not registered".to_string(),
        );
    }

    match origin {
        PcOrigin::TransWomanTransformed => {
            if registry.trans_woman_trait().is_err() {
                errors.push(
                    "character creation requires trait 'TRANS_WOMAN', but it is not registered"
                        .to_string(),
                );
            }
        }
        PcOrigin::CisFemaleTransformed => {
            if registry.always_female_trait().is_err() {
                errors.push(
                    "character creation requires trait 'ALWAYS_FEMALE', but it is not registered"
                        .to_string(),
                );
            }
        }
        PcOrigin::AlwaysFemale => {
            if registry.always_female_trait().is_err() {
                errors.push(
                    "character creation requires trait 'ALWAYS_FEMALE', but it is not registered"
                        .to_string(),
                );
            }
            if registry.not_transformed_trait().is_err() {
                errors.push(
                    "character creation requires trait 'NOT_TRANSFORMED', but it is not registered"
                        .to_string(),
                );
            }
        }
        PcOrigin::CisMaleTransformed => {}
    }

    errors.sort();
    errors.dedup();
    errors
}
