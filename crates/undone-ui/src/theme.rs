use floem::peniko::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Sepia,
    Dark,
}

#[derive(Clone)]
pub struct UserPrefs {
    pub mode: ThemeMode,
    pub font_family: String,
    pub font_size: u8,
    pub line_height: f32,
}

impl Default for UserPrefs {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Light,
            font_family: "Literata, Palatino, Georgia, serif".to_string(),
            font_size: 17,
            line_height: 1.5,
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
