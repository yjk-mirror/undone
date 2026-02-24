use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackManifest {
    pub pack: PackMeta,
    pub content: PackContent,
}

#[derive(Debug, Deserialize)]
pub struct PackMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub opening_scene: Option<String>,
    #[serde(default)]
    pub default_slot: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PackContent {
    pub traits: String,
    pub npc_traits: String,
    pub skills: String,
    pub scenes_dir: String,
    #[serde(default)]
    pub schedule_file: Option<String>,
    #[serde(default)]
    pub names_file: Option<String>,
    #[serde(default)]
    pub stats_file: Option<String>,
    #[serde(default)]
    pub races_file: Option<String>,
    #[serde(default)]
    pub categories_file: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pack_toml() {
        let src = r#"
            [pack]
            id       = "base"
            name     = "Base Game"
            version  = "0.1.0"
            author   = "Undone"
            requires = []
            opening_scene = "base::rain_shelter"
            default_slot  = "free_time"

            [content]
            traits     = "data/traits.toml"
            npc_traits = "data/npc_traits.toml"
            skills     = "data/skills.toml"
            scenes_dir = "scenes/"
        "#;
        let manifest: PackManifest = toml::from_str(src).unwrap();
        assert_eq!(manifest.pack.id, "base");
        assert_eq!(manifest.content.scenes_dir, "scenes/");
        assert_eq!(
            manifest.pack.opening_scene.as_deref(),
            Some("base::rain_shelter")
        );
        assert_eq!(manifest.pack.default_slot.as_deref(), Some("free_time"));
    }
}
