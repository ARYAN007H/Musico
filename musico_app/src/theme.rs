// theme.rs — Musico design system
// Palettes, font modes, responsive dimensions, and all shared styles.

use iced::{
    Background, Border, Color, Shadow,
    widget::{button, container, slider},
};

// ─── Base Color Constants ───────────────────────────────────────────────────

pub const BASE:      Color = Color::from_rgb(0.016, 0.018, 0.035); // #040409
pub const SURFACE:   Color = Color::from_rgb(0.055, 0.059, 0.086); // #0e0f16
pub const ELEVATED:  Color = Color::from_rgb(0.086, 0.090, 0.129); // #161721
pub const HIGHLIGHT: Color = Color::from_rgb(0.125, 0.133, 0.200); // #202233

pub const BORDER_SUBTLE: Color = Color::from_rgb(0.118, 0.125, 0.200); // #1e2033
#[allow(dead_code)]
pub const BORDER_ACCENT: Color = Color::from_rgb(0.145, 0.149, 0.251); // #252640

pub const TEXT_PRIMARY:   Color = Color::from_rgb(0.886, 0.894, 0.941); // #e2e4f0
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.545, 0.561, 0.659); // #8b8fa8
pub const TEXT_MUTED:     Color = Color::from_rgb(0.290, 0.302, 0.388); // #4a4d63

// ─── Color Palettes ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPalette {
    pub id: &'static str,
    pub name: &'static str,
    pub primary: Color,
    pub secondary: Color,
    pub tinted_surface: Color,
}

pub const PALETTE_NEBULA: ColorPalette = ColorPalette {
    id: "nebula", name: "Nebula",
    primary:        Color::from_rgb(0.616, 0.549, 1.000), // #9d8cff
    secondary:      Color::from_rgb(0.769, 0.710, 0.992), // #c4b5fd
    tinted_surface: Color::from_rgb(0.051, 0.043, 0.102), // #0d0b1a
};

pub const PALETTE_SAKURA: ColorPalette = ColorPalette {
    id: "sakura", name: "Sakura",
    primary:        Color::from_rgb(0.957, 0.447, 0.714), // #f472b6
    secondary:      Color::from_rgb(0.992, 0.643, 0.686), // #fda4af
    tinted_surface: Color::from_rgb(0.102, 0.043, 0.071), // #1a0b12
};

pub const PALETTE_AURORA: ColorPalette = ColorPalette {
    id: "aurora", name: "Aurora",
    primary:        Color::from_rgb(0.204, 0.827, 0.600), // #34d399
    secondary:      Color::from_rgb(0.431, 0.906, 0.718), // #6ee7b7
    tinted_surface: Color::from_rgb(0.043, 0.102, 0.078), // #0b1a14
};

pub const PALETTE_OCEAN: ColorPalette = ColorPalette {
    id: "ocean", name: "Ocean",
    primary:        Color::from_rgb(0.220, 0.741, 0.973), // #38bdf8
    secondary:      Color::from_rgb(0.490, 0.827, 0.988), // #7dd3fc
    tinted_surface: Color::from_rgb(0.043, 0.078, 0.102), // #0b141a
};

pub const PALETTE_EMBER: ColorPalette = ColorPalette {
    id: "ember", name: "Ember",
    primary:        Color::from_rgb(0.984, 0.573, 0.235), // #fb923c
    secondary:      Color::from_rgb(0.992, 0.729, 0.455), // #fdba74
    tinted_surface: Color::from_rgb(0.102, 0.071, 0.043), // #1a120b
};

pub const PALETTE_MONO: ColorPalette = ColorPalette {
    id: "mono", name: "Mono",
    primary:        Color::from_rgb(0.631, 0.631, 0.667), // #a1a1aa
    secondary:      Color::from_rgb(0.831, 0.831, 0.847), // #d4d4d8
    tinted_surface: Color::from_rgb(0.067, 0.067, 0.075), // #111113
};

pub const ALL_PALETTES: [ColorPalette; 6] = [
    PALETTE_NEBULA, PALETTE_SAKURA, PALETTE_AURORA,
    PALETTE_OCEAN, PALETTE_EMBER, PALETTE_MONO,
];

pub fn palette_by_id(id: &str) -> ColorPalette {
    ALL_PALETTES.iter().find(|p| p.id == id).copied().unwrap_or(PALETTE_NEBULA)
}

// ─── Font Modes ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontMode {
    Classic,
    Playful,
    Techno,
    Cozy,
}

impl FontMode {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Classic  => "classic",
            Self::Playful  => "playful",
            Self::Techno   => "techno",
            Self::Cozy     => "cozy",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Classic  => "Classic",
            Self::Playful  => "Playful",
            Self::Techno   => "Techno",
            Self::Cozy     => "Cozy",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "playful" => Self::Playful,
            "techno"  => Self::Techno,
            "cozy"    => Self::Cozy,
            _         => Self::Classic,
        }
    }

    pub fn display_font(&self) -> iced::Font {
        match self {
            Self::Classic  => iced::Font::with_name("SF Pro Display"),
            Self::Playful  => iced::Font::with_name("Comfortaa"),
            Self::Techno   => iced::Font::with_name("JetBrains Mono"),
            Self::Cozy     => iced::Font::with_name("Nunito"),
        }
    }

    pub fn text_font(&self) -> iced::Font {
        match self {
            Self::Classic  => iced::Font::with_name("SF Pro Text"),
            Self::Playful  => iced::Font::with_name("Quicksand"),
            Self::Techno   => iced::Font::with_name("Space Mono"),
            Self::Cozy     => iced::Font::with_name("Nunito"),
        }
    }

    pub fn rounded_font(&self) -> iced::Font {
        match self {
            Self::Classic  => iced::Font::with_name("SF Pro Rounded"),
            Self::Playful  => iced::Font::with_name("Nunito"),
            Self::Techno   => iced::Font::with_name("JetBrains Mono"),
            Self::Cozy     => iced::Font::with_name("Varela Round"),
        }
    }

    /// Radius adjustments per mode (LG, MD, SM)
    pub fn radii(&self) -> (f32, f32, f32) {
        match self {
            Self::Classic  => (16.0, 10.0, 6.0),
            Self::Playful  => (20.0, 14.0, 10.0),
            Self::Techno   => (12.0, 8.0, 4.0),
            Self::Cozy     => (22.0, 16.0, 12.0),
        }
    }
}

pub const ALL_FONT_MODES: [FontMode; 4] = [
    FontMode::Classic, FontMode::Playful, FontMode::Techno, FontMode::Cozy,
];

// ─── Active Theme Context ───────────────────────────────────────────────────
// A snapshot of the current palette + font mode for easy passing into views.

#[derive(Debug, Clone, Copy)]
pub struct ThemeCtx {
    pub palette: ColorPalette,
    pub font_mode: FontMode,
    // Resolved fields for convenience:
    pub accent: Color,
    pub accent_secondary: Color,
    pub font_display: iced::Font,
    pub font_text: iced::Font,
    pub font_rounded: iced::Font,
    pub radius_lg: f32,
    pub radius_md: f32,
    pub radius_sm: f32,
}

impl ThemeCtx {
    pub fn new(palette: ColorPalette, font_mode: FontMode) -> Self {
        let (rl, rm, rs) = font_mode.radii();
        Self {
            palette,
            font_mode,
            accent: palette.primary,
            accent_secondary: palette.secondary,
            font_display: font_mode.display_font(),
            font_text: font_mode.text_font(),
            font_rounded: font_mode.rounded_font(),
            radius_lg: rl,
            radius_md: rm,
            radius_sm: rs,
        }
    }

    pub fn default_ctx() -> Self {
        Self::new(PALETTE_NEBULA, FontMode::Classic)
    }
}

// ─── Legacy Compat (used by code that hasn't migrated yet) ──────────────────

pub const FONT_DISPLAY: iced::Font = iced::Font::with_name("SF Pro Display");
pub const FONT_TEXT: iced::Font = iced::Font::with_name("SF Pro Text");
pub const FONT_ROUNDED: iced::Font = iced::Font::with_name("SF Pro Rounded");

pub const RADIUS_LG: f32 = 16.0;
pub const RADIUS_MD: f32 = 10.0;
pub const RADIUS_SM: f32 =  6.0;

// Font sizes
pub const SIZE_HERO:      f32 = 24.0;
pub const SIZE_TITLE:     f32 = 18.0;
pub const SIZE_BODY:      f32 = 14.0;
pub const SIZE_LABEL:     f32 = 13.0;
pub const SIZE_CAPTION:   f32 = 12.0;
#[allow(dead_code)]
pub const SIZE_MICRO:     f32 = 11.0;

pub const TEXT_HERO: f32 = SIZE_HERO;
pub const TEXT_TITLE: f32 = SIZE_TITLE;
pub const TEXT_BODY: f32 = SIZE_BODY;
pub const TEXT_CAPTION: f32 = SIZE_CAPTION;

// ─── Responsive Dimensions ──────────────────────────────────────────────────

pub const SIDEBAR_WIDTH: f32 = 180.0; // Standard mode
pub const SIDEBAR_COMPACT_WIDTH: f32 = 56.0;
#[allow(dead_code)]
pub const BOTTOM_BAR_HEIGHT: f32 = 78.0;

pub fn sidebar_width(window_width: f32) -> f32 {
    if window_width < 700.0 {
        SIDEBAR_COMPACT_WIDTH
    } else {
        SIDEBAR_WIDTH
    }
}

pub fn is_compact(window_width: f32) -> bool {
    window_width < 700.0
}

// ─── Helper ─────────────────────────────────────────────────────────────────

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

pub fn musico_theme() -> iced::Theme {
    iced::Theme::Dark
}

// Legacy Palette struct kept for backward compat during migration
pub struct Palette {
    pub base: Color,
    pub surface: Color,
    pub elevated: Color,
    pub highlight: Color,
    pub accent: Color,
    pub border_subtle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
}

impl Palette {
    pub fn default_palette() -> Self {
        Self {
            base: BASE,
            surface: SURFACE,
            elevated: ELEVATED,
            highlight: HIGHLIGHT,
            accent: PALETTE_NEBULA.primary,
            border_subtle: BORDER_SUBTLE,
            text_primary: TEXT_PRIMARY,
            text_secondary: TEXT_SECONDARY,
            text_muted: TEXT_MUTED,
        }
    }

    /// Build palette from active color palette
    pub fn from_color_palette(cp: &ColorPalette) -> Self {
        Self {
            base: BASE,
            surface: SURFACE,
            elevated: ELEVATED,
            highlight: HIGHLIGHT,
            accent: cp.primary,
            border_subtle: BORDER_SUBTLE,
            text_primary: TEXT_PRIMARY,
            text_secondary: TEXT_SECONDARY,
            text_muted: TEXT_MUTED,
        }
    }
}

// ─── Container Styles ────────────────────────────────────────────────────────

pub fn floating_panel(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 24.0.into(),
        },
        shadow: Shadow {
            color: Color { a: 0.3, ..BASE },
            offset: iced::Vector { x: 0.0, y: 10.0 },
            blur_radius: 30.0,
        },
        ..Default::default()
    }
}

pub fn glass_card(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(Color { a: 0.6, ..ELEVATED })),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: RADIUS_LG.into(),
        },
        ..Default::default()
    }
}

pub fn elevated_card(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(ELEVATED)),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: RADIUS_MD.into(),
        },
        ..Default::default()
    }
}

// ─── Button Styles ───────────────────────────────────────────────────────────

/// Sidebar nav button — accent left-bar indicator when active, subtle tint bg
pub struct NavButton {
    pub is_active: bool,
    pub accent: Color,
}

impl button::StyleSheet for NavButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: if self.is_active {
                Some(Background::Color(with_alpha(self.accent, 0.12)))
            } else {
                None
            },
            text_color: if self.is_active { self.accent } else { TEXT_SECONDARY },
            border: Border {
                radius: 10.0.into(),
                color: if self.is_active { self.accent } else { Color::TRANSPARENT },
                width: if self.is_active { 0.0 } else { 0.0 },
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active {
                with_alpha(self.accent, 0.18)
            } else {
                with_alpha(ELEVATED, 0.8)
            })),
            text_color: if self.is_active { self.accent } else { TEXT_PRIMARY },
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }
}

pub struct SvgStyle(pub Color);
impl iced::widget::svg::StyleSheet for SvgStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::svg::Appearance {
        iced::widget::svg::Appearance {
            color: Some(self.0),
        }
    }
}

/// Ghost transport button (prev, next, shuffle, repeat)
pub struct TransportButton;

impl button::StyleSheet for TransportButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            text_color: TEXT_SECONDARY,
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(ELEVATED)),
            text_color: TEXT_PRIMARY,
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }
}

/// Accent-tinted transport button — hover uses accent tint
pub struct AccentTransportButton(pub Color);

impl button::StyleSheet for AccentTransportButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: None,
            text_color: TEXT_SECONDARY,
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(with_alpha(self.0, 0.12))),
            text_color: TEXT_PRIMARY,
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }
}

/// Volume slider style
pub struct VolumeSliderStyle(pub Color);

impl slider::StyleSheet for VolumeSliderStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (self.0, Color { a: 0.3, ..ELEVATED }),
                width: 4.0,
                border_radius: 2.0.into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 6.0 },
                color: Color::WHITE,
                border_width: 2.0,
                border_color: self.0,
            },
        }
    }

    fn hovered(&self, _style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (self.0, Color { a: 0.3, ..ELEVATED }),
                width: 4.0,
                border_radius: 2.0.into(),
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 7.0 },
                color: Color::WHITE,
                border_width: 2.0,
                border_color: self.0,
            },
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        self.hovered(style)
    }
}
