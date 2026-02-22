use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use undone_domain::StatId;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameData {
    pub flags: HashSet<String>,
    pub stats: HashMap<StatId, i32>,
    pub job_title: String,
    pub allow_anal: bool,
    pub week: u32,
}

impl GameData {
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    pub fn set_flag(&mut self, flag: impl Into<String>) {
        self.flags.insert(flag.into());
    }

    pub fn remove_flag(&mut self, flag: &str) {
        self.flags.remove(flag);
    }

    pub fn get_stat(&self, id: StatId) -> i32 {
        self.stats.get(&id).copied().unwrap_or(0)
    }

    pub fn add_stat(&mut self, id: StatId, amount: i32) {
        *self.stats.entry(id).or_insert(0) += amount;
    }

    pub fn set_stat(&mut self, id: StatId, value: i32) {
        self.stats.insert(id, value);
    }
}
