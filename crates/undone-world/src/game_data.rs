use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use undone_domain::{StatId, TimeSlot};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameData {
    pub flags: HashSet<String>,
    pub stats: HashMap<StatId, i32>,
    pub job_title: String,
    pub allow_anal: bool,
    pub week: u32,
    #[serde(default)]
    pub day: u8, // 0–6 (Mon–Sun)
    #[serde(default = "default_time_slot")]
    pub time_slot: TimeSlot,
    /// Arc state machine: arc_id → current state name.
    /// Added with serde(default) for backward-compat with v3 saves.
    #[serde(default)]
    pub arc_states: HashMap<String, String>,
    /// Red-check permanent failure registry: "scene_id::skill_id".
    /// Once a red check fails it is blocked for the rest of the game.
    #[serde(default)]
    pub red_check_failures: HashSet<String>,
}

fn default_time_slot() -> TimeSlot {
    TimeSlot::Morning
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

    /// Advance to the next time slot. Returns true if the week rolled over.
    pub fn advance_time_slot(&mut self) -> bool {
        match self.time_slot.next() {
            Some(next) => {
                self.time_slot = next;
                false
            }
            None => {
                // Day is over — advance to next day
                self.time_slot = TimeSlot::Morning;
                self.day += 1;
                if self.day > 6 {
                    self.day = 0;
                    self.week += 1;
                    true // week rolled over
                } else {
                    false
                }
            }
        }
    }

    pub fn is_weekday(&self) -> bool {
        self.day <= 4 // 0=Mon through 4=Fri
    }

    pub fn is_weekend(&self) -> bool {
        self.day >= 5 // 5=Sat, 6=Sun
    }

    /// Record a permanent red-check failure for this scene+skill combination.
    pub fn fail_red_check(&mut self, scene_id: &str, skill_id: &str) {
        self.red_check_failures
            .insert(format!("{scene_id}::{skill_id}"));
    }

    /// Returns true if a red check for this scene+skill has been permanently failed.
    pub fn has_failed_red_check(&self, scene_id: &str, skill_id: &str) -> bool {
        self.red_check_failures
            .contains(&format!("{scene_id}::{skill_id}"))
    }

    /// Returns the current state name for an arc, if the arc has been started.
    pub fn arc_state(&self, arc_id: &str) -> Option<&str> {
        self.arc_states.get(arc_id).map(|s| s.as_str())
    }

    /// Advance an arc to a new state (or start it at the given state).
    pub fn advance_arc(&mut self, arc_id: impl Into<String>, state: impl Into<String>) {
        self.arc_states.insert(arc_id.into(), state.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_time_slot_morning_to_afternoon() {
        let mut gd = GameData::default();
        assert_eq!(gd.time_slot, TimeSlot::Morning);
        assert!(!gd.advance_time_slot());
        assert_eq!(gd.time_slot, TimeSlot::Afternoon);
    }

    #[test]
    fn advance_time_slot_night_rolls_day() {
        let mut gd = GameData::default();
        gd.time_slot = TimeSlot::Night;
        gd.day = 0; // Monday
        assert!(!gd.advance_time_slot());
        assert_eq!(gd.time_slot, TimeSlot::Morning);
        assert_eq!(gd.day, 1); // Tuesday
    }

    #[test]
    fn advance_time_slot_sunday_night_rolls_week() {
        let mut gd = GameData::default();
        gd.time_slot = TimeSlot::Night;
        gd.day = 6; // Sunday
        gd.week = 0;
        assert!(gd.advance_time_slot()); // week rolled over
        assert_eq!(gd.time_slot, TimeSlot::Morning);
        assert_eq!(gd.day, 0); // Monday
        assert_eq!(gd.week, 1);
    }

    #[test]
    fn red_check_absent_initially() {
        let gd = GameData::default();
        assert!(!gd.has_failed_red_check("base::some_scene", "CHARM"));
    }

    #[test]
    fn red_check_present_after_fail() {
        let mut gd = GameData::default();
        gd.fail_red_check("base::some_scene", "CHARM");
        assert!(gd.has_failed_red_check("base::some_scene", "CHARM"));
    }

    #[test]
    fn red_check_does_not_cross_contaminate() {
        let mut gd = GameData::default();
        gd.fail_red_check("base::some_scene", "CHARM");
        // Different skill — not failed
        assert!(!gd.has_failed_red_check("base::some_scene", "FITNESS"));
        // Different scene — not failed
        assert!(!gd.has_failed_red_check("base::other_scene", "CHARM"));
    }

    #[test]
    fn arc_state_absent_initially() {
        let gd = GameData::default();
        assert_eq!(gd.arc_state("base::jake"), None);
    }

    #[test]
    fn arc_advance_and_query() {
        let mut gd = GameData::default();
        gd.advance_arc("base::jake", "acquaintance");
        assert_eq!(gd.arc_state("base::jake"), Some("acquaintance"));
    }

    #[test]
    fn arc_advance_overwrites_previous_state() {
        let mut gd = GameData::default();
        gd.advance_arc("base::jake", "acquaintance");
        gd.advance_arc("base::jake", "friend");
        assert_eq!(gd.arc_state("base::jake"), Some("friend"));
    }

    #[test]
    fn is_weekday_and_weekend() {
        let mut gd = GameData::default();
        gd.day = 0;
        assert!(gd.is_weekday());
        assert!(!gd.is_weekend());
        gd.day = 5;
        assert!(!gd.is_weekday());
        assert!(gd.is_weekend());
    }
}
