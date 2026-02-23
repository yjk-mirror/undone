use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TraitFile {
    #[serde(rename = "trait")]
    pub traits: Vec<TraitDef>,
}

#[derive(Debug, Deserialize)]
pub struct TraitDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Deserialize)]
pub struct NpcTraitFile {
    #[serde(rename = "trait")]
    pub traits: Vec<NpcTraitDef>,
}

#[derive(Debug, Deserialize)]
pub struct NpcTraitDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Deserialize)]
pub struct SkillFile {
    pub skill: Vec<SkillDef>,
}

#[derive(Debug, Deserialize)]
pub struct SkillDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Deserialize)]
pub struct NamesFile {
    pub male_names: Vec<String>,
    pub female_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatDef {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct StatFile {
    #[serde(default)]
    pub stat: Vec<StatDef>,
}
