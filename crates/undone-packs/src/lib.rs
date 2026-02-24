pub mod char_creation;
pub mod data;
pub mod loader;
pub mod manifest;
pub mod registry;
pub mod spawner;

pub use char_creation::{new_game, CharCreationConfig};
pub use data::{CategoriesFile, CategoryDef, NamesFile, NpcTraitDef, SkillDef, TraitDef};
pub use loader::{load_packs, LoadedPackMeta, PackLoadError};
pub use manifest::{PackContent, PackManifest, PackMeta};
pub use registry::{PackRegistry, RegistryError};
pub use spawner::{spawn_npcs, NpcSpawnConfig};
