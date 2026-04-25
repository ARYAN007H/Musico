use iced::font::Font;
use iced::Color;

pub struct Palette {
    pub base: Color,
    pub surface: Color,
    pub elevated: Color,
    pub highlight: Color,

    pub border_subtle: Color,
    pub border_accent: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    pub accent: Color,
    pub accent_rose: Color,
    pub accent_cyan: Color,

    pub art_tint: Color,
}

impl Palette {
    pub fn default_palette() -> Self {
        Self {
            base: color_from_hex("#0d0e14"),
            surface: color_from_hex("#13141d"),
            elevated: color_from_hex("#1a1b26"),
            highlight: color_from_hex("#1f2133"),

            border_subtle: Color {
                a: 0.6,
                ..color_from_hex("#2a2d3e")
            },
            border_accent: color_from_hex("#3d4163"),

            text_primary: color_from_hex("#e2e4f0"),
            text_secondary: color_from_hex("#8b8fa8"),
            text_muted: color_from_hex("#4a4d63"),

            accent: color_from_hex("#9d8cff"),
            accent_rose: color_from_hex("#ff8fa3"),
            accent_cyan: color_from_hex("#7dcfff"),

            art_tint: Color::TRANSPARENT,
        }
    }
}

// SF Pro is available system-wide on this machine
pub const FONT_DISPLAY: Font = Font::with_name("SF Pro Display");
pub const FONT_TEXT: Font = Font::with_name("SF Pro Text");
pub const FONT_ROUNDED: Font = Font::with_name("SF Pro Rounded");

// Sizes
pub const TEXT_HERO: f32 = 28.0;     // song title in now playing
pub const TEXT_TITLE: f32 = 18.0;    // section headings
pub const TEXT_BODY: f32 = 14.0;     // song rows, metadata
pub const TEXT_CAPTION: f32 = 12.0;  // timestamps, labels
pub const TEXT_MICRO: f32 = 10.0;    // badges, chips

// Spacing & Shape
pub const RADIUS_LG: f32 = 16.0;   // panels, main cards
pub const RADIUS_MD: f32 = 10.0;   // buttons, inputs
pub const RADIUS_SM: f32 = 6.0;    // tags, chips, badges
pub const SIDEBAR_WIDTH_FULL: f32 = 220.0;
pub const SIDEBAR_WIDTH_RAIL: f32 = 64.0;
pub const NOW_PLAYING_ART_MAX: f32 = 320.0;

fn color_from_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb8(r, g, b)
}

// Minimal implementation of iced::theme::Custom to use our palette
// We will use iced::Theme::custom("Musico", musico_palette())
pub fn musico_theme() -> iced::Theme {
    let p = Palette::default_palette();
    
    iced::Theme::custom(
        "Musico".to_string(),
        iced::theme::Palette {
            background: p.base,
            text: p.text_primary,
            primary: p.accent,
            success: color_from_hex("#9ece6a"),
            danger: color_from_hex("#f7768e"),
        }
    )
}
