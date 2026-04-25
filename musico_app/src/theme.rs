// theme.rs — Musico Celestia Shell design system
// Every color, radius, and style function lives here.

use iced::{
    Background, Border, Color, Shadow,
    widget::{button, container, text_input},
};

// ─── Color Palette ──────────────────────────────────────────────────────────

pub const BASE:      Color = Color::from_rgb(0.024, 0.027, 0.051); // #06070d
pub const SURFACE:   Color = Color::from_rgb(0.051, 0.055, 0.078); // #0d0e14
pub const ELEVATED:  Color = Color::from_rgb(0.059, 0.063, 0.098); // #0f1019
pub const HIGHLIGHT: Color = Color::from_rgb(0.102, 0.106, 0.180); // #1a1b2e

pub const BORDER_SUBTLE: Color = Color::from_rgb(0.118, 0.125, 0.200); // #1e2033
pub const BORDER_ACCENT: Color = Color::from_rgb(0.145, 0.149, 0.251); // #252640

pub const TEXT_PRIMARY:   Color = Color::from_rgb(0.886, 0.894, 0.941); // #e2e4f0
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.545, 0.561, 0.659); // #8b8fa8
pub const TEXT_MUTED:     Color = Color::from_rgb(0.290, 0.302, 0.388); // #4a4d63

pub const ACCENT_PURPLE: Color = Color::from_rgb(0.616, 0.549, 1.000); // #9d8cff
pub const ACCENT_ROSE:   Color = Color::from_rgb(1.000, 0.561, 0.639); // #ff8fa3
pub const ACCENT_CYAN:   Color = Color::from_rgb(0.490, 0.812, 1.000); // #7dcfff
pub const ACCENT_AMBER:  Color = Color::from_rgb(1.000, 0.620, 0.392); // #ff9e64
pub const ACCENT_GREEN:  Color = Color::from_rgb(0.431, 0.906, 0.718); // #6ee7b7

// ─── Dimensions ─────────────────────────────────────────────────────────────

pub const SIDEBAR_WIDTH:     f32 = 208.0;
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

pub const NOW_PLAYING_ART_MAX: f32 = 400.0;

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

pub fn sidebar_container(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(ELEVATED)),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn base_container(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(BASE)),
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

pub fn surface_card(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: RADIUS_LG.into(),
        },
        ..Default::default()
    }
}

pub fn bottom_bar_container(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: BORDER_SUBTLE,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn album_art_container(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(ELEVATED)),
        border: Border {
            color: BORDER_ACCENT,
            width: 1.0,
            radius: 20.0.into(),
        },
        shadow: Shadow {
            color: Color { a: 0.6, ..BASE },
            offset: iced::Vector { x: 0.0, y: 20.0 },
            blur_radius: 60.0,
        },
        ..Default::default()
    }
}

pub fn highlight_row(_theme: &iced::Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Background::Color(HIGHLIGHT)),
        border: Border {
            radius: RADIUS_MD.into(),
            ..Default::default()
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
                Some(Background::Color(HIGHLIGHT))
            } else {
                None
            },
            text_color: if self.is_active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            border: Border {
                radius: RADIUS_MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(if self.is_active { HIGHLIGHT } else { ELEVATED })),
            text_color: TEXT_PRIMARY,
            border: Border {
                radius: RADIUS_MD.into(),
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

/// Solid accent (purple) FAB-style play button
pub struct AccentButton;

impl button::StyleSheet for AccentButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(ACCENT_PURPLE)),
            text_color: BASE,
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(Color {
                r: ACCENT_PURPLE.r + 0.07,
                g: ACCENT_PURPLE.g + 0.07,
                b: ACCENT_PURPLE.b + 0.04,
                a: 1.0,
            })),
            text_color: BASE,
            border: Border {
                radius: 50.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
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

/// Settings action button (Change Folder, Re-index)
pub struct SecondaryButton;

impl button::StyleSheet for SecondaryButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(with_alpha(ACCENT_PURPLE, 0.12))),
            text_color: ACCENT_PURPLE,
            border: Border {
                color: with_alpha(ACCENT_PURPLE, 0.25),
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(with_alpha(ACCENT_PURPLE, 0.20))),
            text_color: TEXT_PRIMARY,
            border: Border {
                color: ACCENT_PURPLE,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
    }
}

/// Swatch button (accent color picker in settings)
pub struct SwatchButton {
    pub color: Color,
    pub is_selected: bool,
}

impl button::StyleSheet for SwatchButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.color)),
            text_color: Color::TRANSPARENT,
            border: Border {
                color: if self.is_selected { TEXT_PRIMARY } else { Color::TRANSPARENT },
                width: if self.is_selected { 2.0 } else { 0.0 },
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(Background::Color(self.color)),
            text_color: Color::TRANSPARENT,
            border: Border {
                color: TEXT_SECONDARY,
                width: 2.0,
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.active(_style)
    }
}

/// Song row button
pub struct SongRowButton {
    pub is_playing: bool,
}

impl button::StyleSheet for SongRowButton {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: if self.is_playing {
                Some(Background::Color(HIGHLIGHT))
            } else {
                None
            },
            text_color: TEXT_PRIMARY,
            border: Border {
                radius: RADIUS_MD.into(),
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
                radius: RADIUS_MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> button::Appearance {
        self.hovered(_style)
    }
}

// ─── Text Input Style ─────────────────────────────────────────────────────────

pub struct SearchInput;

impl text_input::StyleSheet for SearchInput {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(ELEVATED),
            border: Border {
                color: BORDER_SUBTLE,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            icon_color: TEXT_MUTED,
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(ELEVATED),
            border: Border {
                color: ACCENT_PURPLE,
                width: 1.0,
                radius: RADIUS_SM.into(),
            },
            icon_color: TEXT_SECONDARY,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color { TEXT_MUTED }
    fn value_color(&self, _style: &Self::Style) -> Color { TEXT_PRIMARY }
    fn disabled_color(&self, _style: &Self::Style) -> Color { TEXT_MUTED }
    fn selection_color(&self, _style: &Self::Style) -> Color { with_alpha(ACCENT_PURPLE, 0.35) }
    fn hovered(&self, style: &Self::Style) -> text_input::Appearance { self.active(style) }
    fn disabled(&self, style: &Self::Style) -> text_input::Appearance { self.active(style) }
}

// ─── Accent Colors registry ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AccentColor {
    Purple,
    Rose,
    Cyan,
    Amber,
    Green,
}

impl AccentColor {
    pub fn color(&self) -> Color {
        match self {
            AccentColor::Purple => ACCENT_PURPLE,
            AccentColor::Rose   => ACCENT_ROSE,
            AccentColor::Cyan   => ACCENT_CYAN,
            AccentColor::Amber  => ACCENT_AMBER,
            AccentColor::Green  => ACCENT_GREEN,
        }
    }

    pub fn all() -> &'static [AccentColor] {
        &[
            AccentColor::Purple,
            AccentColor::Rose,
            AccentColor::Cyan,
            AccentColor::Amber,
            AccentColor::Green,
        ]
    }
}
