use floem::peniko::Color;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeMode {
    Light,
    Sepia,
    Dark,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumberKeyMode {
    #[default]
    Instant,
    Confirm,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPrefs {
    pub mode: ThemeMode,
    pub font_family: String,
    pub font_size: u8,
    pub line_height: f32,
    #[serde(default)]
    pub number_key_mode: NumberKeyMode,
}

fn prefs_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("undone").join("prefs.json"))
}

pub fn load_prefs() -> UserPrefs {
    prefs_path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_prefs(prefs: &UserPrefs) {
    if let Some(path) = prefs_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(
            &path,
            serde_json::to_string_pretty(prefs).unwrap_or_default(),
        );
    }
}

impl Default for UserPrefs {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Light,
            font_family: "Literata, Palatino, Georgia, serif".to_string(),
            font_size: 17,
            line_height: 1.5,
            number_key_mode: NumberKeyMode::Instant,
        }
    }
}

pub struct ThemeColors {
    pub ground: Color,
    pub page: Color,
    pub page_raised: Color,
    pub sidebar_ground: Color,
    pub ink: Color,
    pub ink_dim: Color,
    pub ink_ghost: Color,
    pub seam: Color,
    pub lamp: Color,
    pub lamp_glow: Color,
}

impl ThemeColors {
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self {
                ground: Color::rgb8(0xF7, 0xF2, 0xE8),
                page: Color::rgb8(0xFD, 0xFA, 0xF4),
                page_raised: Color::rgb8(0xF0, 0xEB, 0xE0),
                sidebar_ground: Color::rgb8(0xED, 0xE8, 0xDC),
                ink: Color::rgb8(0x1C, 0x18, 0x14),
                ink_dim: Color::rgb8(0x5A, 0x52, 0x48),
                ink_ghost: Color::rgb8(0x8C, 0x80, 0x78),
                seam: Color::rgba8(28, 24, 20, 25), // 10%
                lamp: Color::rgb8(0xB0, 0x70, 0x30),
                lamp_glow: Color::rgba8(176, 112, 48, 30), // 12%
            },
            ThemeMode::Sepia => Self {
                ground: Color::rgb8(0xE6, 0xD8, 0xB8),
                page: Color::rgb8(0xF0, 0xE6, 0xCC),
                page_raised: Color::rgb8(0xE2, 0xD4, 0xB4),
                sidebar_ground: Color::rgb8(0xDA, 0xCA, 0xA8),
                ink: Color::rgb8(0x2C, 0x20, 0x0E),
                ink_dim: Color::rgb8(0x5A, 0x48, 0x30),
                ink_ghost: Color::rgb8(0x90, 0x7C, 0x60),
                seam: Color::rgba8(44, 32, 14, 30), // 12%
                lamp: Color::rgb8(0xA8, 0x68, 0x18),
                lamp_glow: Color::rgba8(168, 104, 24, 30), // 12%
            },
            ThemeMode::Dark => Self {
                ground: Color::rgb8(0x14, 0x12, 0x10),
                page: Color::rgb8(0x1A, 0x18, 0x16),
                page_raised: Color::rgb8(0x20, 0x1E, 0x1C),
                sidebar_ground: Color::rgb8(0x11, 0x10, 0x08),
                ink: Color::rgb8(0xE8, 0xDD, 0xD0),
                ink_dim: Color::rgb8(0xA8, 0x9D, 0x90),
                ink_ghost: Color::rgb8(0x6E, 0x65, 0x60),
                seam: Color::rgba8(232, 221, 208, 20), // 8%
                lamp: Color::rgb8(0xC0, 0x80, 0x40),
                lamp_glow: Color::rgba8(192, 128, 64, 30), // 12%
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_key_mode_roundtrip_serde() {
        let prefs = UserPrefs {
            number_key_mode: NumberKeyMode::Confirm,
            ..UserPrefs::default()
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: UserPrefs = serde_json::from_str(&json).unwrap();
        assert_eq!(back.number_key_mode, NumberKeyMode::Confirm);
        // Old prefs without the field should deserialize to Instant
        let old_json = r#"{"mode":"Light","font_family":"x","font_size":17,"line_height":1.5}"#;
        let old: UserPrefs = serde_json::from_str(old_json).unwrap();
        assert_eq!(old.number_key_mode, NumberKeyMode::Instant);
    }

    #[test]
    fn user_prefs_roundtrip_serde() {
        let prefs = UserPrefs {
            mode: ThemeMode::Dark,
            ..UserPrefs::default()
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: UserPrefs = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mode, ThemeMode::Dark);
        assert_eq!(back.font_size, prefs.font_size);
    }
}
