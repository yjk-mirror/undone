use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TraitFile {
    #[serde(rename = "trait")]
    pub traits: Vec<TraitDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraitDef {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct NpcTraitFile {
    #[serde(rename = "trait")]
    pub traits: Vec<NpcTraitDef>,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct RacesFile {
    pub races: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CategoriesFile {
    #[serde(default)]
    pub category: Vec<CategoryDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategoryType {
    Race,
    Age,
    Trait,
    Personality,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CategoryDef {
    pub id: String,
    pub description: String,
    #[serde(rename = "type")]
    pub category_type: CategoryType,
    pub members: Vec<String>,
}

// ---------------------------------------------------------------------------
// Arc data
// ---------------------------------------------------------------------------

/// A named story arc with an ordered list of valid state names.
#[derive(Debug, Clone, Deserialize)]
pub struct ArcDef {
    pub id: String,
    /// Ordered list of valid state names (informational; engine doesn't validate transitions).
    pub states: Vec<String>,
    /// Optional NPC role tag for the arc's primary NPC.
    #[serde(default)]
    pub npc_role: Option<String>,
    /// If set, new games auto-start the arc in this state.
    #[serde(default)]
    pub initial_state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArcsFile {
    #[serde(default)]
    pub arc: Vec<ArcDef>,
}
