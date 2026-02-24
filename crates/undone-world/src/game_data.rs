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
