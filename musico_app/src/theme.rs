// theme.rs — Musico Celestia Shell design system
// Every color, radius, and style function lives here.

use iced::{
    Background, Border, Color, Shadow,
    widget::{button, container},
};

// ─── Color Palette ──────────────────────────────────────────────────────────

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

pub const ACCENT_PURPLE: Color = Color::from_rgb(0.616, 0.549, 1.000); // #9d8cff
#[allow(dead_code)]
pub const ACCENT_ROSE:   Color = Color::from_rgb(1.000, 0.561, 0.639); // #ff8fa3
#[allow(dead_code)]
pub const ACCENT_CYAN:   Color = Color::from_rgb(0.490, 0.812, 1.000); // #7dcfff
#[allow(dead_code)]
pub const ACCENT_AMBER:  Color = Color::from_rgb(1.000, 0.620, 0.392); // #ff9e64
#[allow(dead_code)]
pub const ACCENT_GREEN:  Color = Color::from_rgb(0.431, 0.906, 0.718); // #6ee7b7

// ─── Dimensions ─────────────────────────────────────────────────────────────

pub const SIDEBAR_WIDTH:     f32 = 208.0;
#[allow(dead_code)]
pub const BOTTOM_BAR_HEIGHT: f32 = 78.0;
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

// Aliases for compatibility
pub const TEXT_HERO: f32 = SIZE_HERO;
pub const TEXT_TITLE: f32 = SIZE_TITLE;
pub const TEXT_BODY: f32 = SIZE_BODY;
pub const TEXT_CAPTION: f32 = SIZE_CAPTION;

// Fonts
pub const FONT_DISPLAY: iced::Font = iced::Font::with_name("SF Pro Display");
pub const FONT_TEXT: iced::Font = iced::Font::with_name("SF Pro Text");
pub const FONT_ROUNDED: iced::Font = iced::Font::with_name("SF Pro Rounded");

// ─── Helper: semi-transparent color ─────────────────────────────────────────

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

pub fn musico_theme() -> iced::Theme {
    iced::Theme::Dark
}

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
            accent: ACCENT_PURPLE,
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

/// Transparent ghost button — used for sidebar nav items (inactive)
pub struct NavButton {
    pub is_active: bool,
}

impl button::StyleSheet for NavButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: if self.is_active {
                Some(Background::Color(ACCENT_PURPLE))
            } else {
                None
            },
            text_color: if self.is_active { BASE } else { TEXT_SECONDARY },
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active { ACCENT_PURPLE } else { ELEVATED })),
            text_color: if self.is_active { BASE } else { TEXT_PRIMARY },
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.hovered(_style)
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

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.hovered(_style)
    }
}
