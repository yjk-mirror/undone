use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindDefinitionParams {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindReferencesParams {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkspaceSymbolsParams {
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameSymbolParams {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
    pub new_name: String,
}
