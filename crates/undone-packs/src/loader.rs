use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    data::{NpcTraitFile, SkillFile, TraitFile},
    manifest::PackManifest,
    registry::PackRegistry,
};

#[derive(Debug, Error)]
pub enum PackLoadError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("toml parse error in {path}: {message}")]
    Toml { path: PathBuf, message: String },
    #[error("packs directory not found: {0}")]
    PacksDirNotFound(PathBuf),
}

pub struct LoadedPackMeta {
    pub manifest: PackManifest,
    pub pack_dir: PathBuf,
}

pub fn load_packs(packs_dir: &Path) -> Result<(PackRegistry, Vec<LoadedPackMeta>), PackLoadError> {
    if !packs_dir.exists() {
        return Err(PackLoadError::PacksDirNotFound(packs_dir.to_path_buf()));
    }

    let mut registry = PackRegistry::new();
    let mut metas = Vec::new();

    let entries = std::fs::read_dir(packs_dir).map_err(|e| PackLoadError::Io {
        path: packs_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| PackLoadError::Io {
            path: packs_dir.to_path_buf(),
            source: e,
        })?;
        let pack_dir = entry.path();
        if !pack_dir.is_dir() {
            continue;
        }
        let manifest_path = pack_dir.join("pack.toml");
        if !manifest_path.exists() {
            continue;
        }
        let meta = load_one_pack(&mut registry, &pack_dir)?;
        metas.push(meta);
    }

    Ok((registry, metas))
}

fn load_one_pack(
    registry: &mut PackRegistry,
    pack_dir: &Path,
) -> Result<LoadedPackMeta, PackLoadError> {
    let manifest_path = pack_dir.join("pack.toml");
    let src = read_file(&manifest_path)?;
    let manifest: PackManifest = toml::from_str(&src).map_err(|e| PackLoadError::Toml {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;

    if let Some(ref scene) = manifest.pack.opening_scene {
        registry.set_opening_scene(scene.clone());
    }
    if let Some(ref slot) = manifest.pack.default_slot {
        registry.set_default_slot(slot.clone());
    }

    let traits_path = pack_dir.join(&manifest.content.traits);
    let src = read_file(&traits_path)?;
    let trait_file: TraitFile = toml::from_str(&src).map_err(|e| PackLoadError::Toml {
        path: traits_path.clone(),
        message: e.to_string(),
    })?;
    registry.register_traits(trait_file.traits);

    let npc_traits_path = pack_dir.join(&manifest.content.npc_traits);
    let src = read_file(&npc_traits_path)?;
    let npc_trait_file: NpcTraitFile = toml::from_str(&src).map_err(|e| PackLoadError::Toml {
        path: npc_traits_path.clone(),
        message: e.to_string(),
    })?;
    registry.register_npc_traits(npc_trait_file.traits);

    let skills_path = pack_dir.join(&manifest.content.skills);
    let src = read_file(&skills_path)?;
    let skill_file: SkillFile = toml::from_str(&src).map_err(|e| PackLoadError::Toml {
        path: skills_path.clone(),
        message: e.to_string(),
    })?;
    registry.register_skills(skill_file.skill);

    if let Some(ref names_rel) = manifest.content.names_file {
        let names_path = pack_dir.join(names_rel);
        let src = read_file(&names_path)?;
        let names_file: crate::data::NamesFile =
            toml::from_str(&src).map_err(|e| PackLoadError::Toml {
                path: names_path.clone(),
                message: e.to_string(),
            })?;
        registry.register_names(names_file.male_names, names_file.female_names);
    }

    if let Some(ref stats_rel) = manifest.content.stats_file {
        let stats_path = pack_dir.join(stats_rel);
        let src = read_file(&stats_path)?;
        let stats_file: crate::data::StatFile =
            toml::from_str(&src).map_err(|e| PackLoadError::Toml {
                path: stats_path.clone(),
                message: e.to_string(),
            })?;
        registry.register_stats(stats_file.stat);
    }

    Ok(LoadedPackMeta {
        manifest,
        pack_dir: pack_dir.to_path_buf(),
    })
}

fn read_file(path: &Path) -> Result<String, PackLoadError> {
    std::fs::read_to_string(path).map_err(|e| PackLoadError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap() // exits crates/undone-packs/
            .parent()
            .unwrap() // exits crates/
            .join("packs")
    }

    #[test]
    fn loads_base_pack_traits() {
        let (registry, metas) = load_packs(&packs_dir()).unwrap();
        assert!(!metas.is_empty(), "should have at least one pack");
        assert!(
            registry.resolve_trait("SHY").is_ok(),
            "SHY trait should be registered"
        );
    }

    #[test]
    fn loads_base_pack_skills() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        assert!(
            registry.resolve_skill("FEMININITY").is_ok(),
            "FEMININITY skill should be registered"
        );
    }

    #[test]
    fn error_on_nonexistent_dir() {
        let result = load_packs(std::path::Path::new("/nonexistent/packs"));
        assert!(result.is_err(), "should error on missing directory");
    }

    #[test]
    fn loads_base_pack_stats() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        assert!(
            registry.get_stat("TIMES_KISSED").is_some(),
            "TIMES_KISSED stat should be interned"
        );
    }

    #[test]
    fn loads_base_pack_names() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        assert!(
            !registry.male_names().is_empty(),
            "should have loaded male names"
        );
        assert!(
            !registry.female_names().is_empty(),
            "should have loaded female names"
        );
        assert!(
            registry.male_names().contains(&"James".to_string()),
            "should include James"
        );
    }

    #[test]
    fn base_pack_has_opening_scene() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        assert_eq!(registry.opening_scene(), Some("base::rain_shelter"));
    }

    #[test]
    fn base_pack_has_default_slot() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        assert_eq!(registry.default_slot(), Some("free_time"));
    }
}
