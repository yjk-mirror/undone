use serde::{Deserialize, Deserializer, Serialize};

/// A stat clamped to [0, 100]. Used for stress and anxiety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct BoundedStat(i32);

impl BoundedStat {
    pub const MIN: i32 = 0;
    pub const MAX: i32 = 100;

    pub fn new(value: i32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> i32 {
        self.0
    }

    pub fn apply_delta(&mut self, delta: i32) {
        self.0 = (self.0 + delta).clamp(Self::MIN, Self::MAX);
    }
}

impl Default for BoundedStat {
    fn default() -> Self {
        Self::new(Self::MIN)
    }
}

impl std::fmt::Display for BoundedStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for BoundedStat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Ok(Self::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_to_min() {
        assert_eq!(BoundedStat::new(-50).get(), 0);
    }

    #[test]
    fn new_clamps_to_max() {
        assert_eq!(BoundedStat::new(200).get(), 100);
    }

    #[test]
    fn new_preserves_valid_value() {
        assert_eq!(BoundedStat::new(42).get(), 42);
    }

    #[test]
    fn apply_delta_positive() {
        let mut s = BoundedStat::new(50);
        s.apply_delta(30);
        assert_eq!(s.get(), 80);
    }

    #[test]
    fn apply_delta_negative() {
        let mut s = BoundedStat::new(50);
        s.apply_delta(-30);
        assert_eq!(s.get(), 20);
    }

    #[test]
    fn apply_delta_clamps_floor() {
        let mut s = BoundedStat::new(10);
        s.apply_delta(-50);
        assert_eq!(s.get(), 0);
    }

    #[test]
    fn apply_delta_clamps_ceiling() {
        let mut s = BoundedStat::new(90);
        s.apply_delta(50);
        assert_eq!(s.get(), 100);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(BoundedStat::default().get(), 0);
    }

    #[test]
    fn serde_roundtrips_as_bare_i32() {
        let stat = BoundedStat::new(42);
        let json = serde_json::to_string(&stat).unwrap();
        assert_eq!(json, "42");
        let back: BoundedStat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, stat);
    }

    #[test]
    fn serde_clamps_on_deserialize() {
        let back: BoundedStat = serde_json::from_str("-10").unwrap();
        assert_eq!(back.get(), 0);

        let back: BoundedStat = serde_json::from_str("200").unwrap();
        assert_eq!(back.get(), 100);
    }
}
