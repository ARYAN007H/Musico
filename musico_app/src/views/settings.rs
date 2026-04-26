use iced::widget::{button, column, container, progress_bar, row, text, Space};
use iced::{Alignment, Color, Element, Length};
use crate::state::AppState;
use crate::theme::{self, Palette};

pub fn settings<'a, Message: 'a + Clone>(
    state: &AppState,
    on_change_folder: Message,
    on_scan: Message,
    on_set_accent: impl Fn(Color) -> Message + 'a,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    let mut content = column![].spacing(32).padding(40);

    // Header
    content = content.push(
        text("Settings")
            .font(theme::FONT_DISPLAY)
            .size(theme::TEXT_HERO)
            .style(p.text_primary)
    );

    // Music Folder Section
    let folder_path = match &state.music_folder {
        Some(path) => path.to_string_lossy().to_string(),
        None => "Not selected".to_string(),
    };

    let folder_section = container(column![
        text("Music Folder").font(theme::FONT_ROUNDED).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(16),
        row![
            container(text(&folder_path).style(p.text_secondary))
                .padding(12)
                .width(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(InputBgStyle(p.surface)))),
            Space::with_width(12),
            button(text("Change Folder").style(p.text_primary))
                .on_press(on_change_folder)
                .padding([12, 20])
                .style(iced::theme::Button::Custom(Box::new(PrimaryBtnStyle(p.elevated, p.highlight))))
        ].align_items(Alignment::Center)
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(folder_section);

    // Re-index Section
    let mut index_content = column![
        text("Library Index").font(theme::FONT_ROUNDED).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(16),
    ].spacing(8);

    if state.is_indexing {
        let (done, total) = state.index_progress;
        let progress = if total > 0 { done as f32 / total as f32 } else { 0.0 };
        
        index_content = index_content.push(
            row![
                progress_bar(0.0..=1.0, progress)
                    .height(Length::Fixed(8.0))
                    .style(iced::theme::ProgressBar::Custom(Box::new(ProgressStyle(state.art_tint, p.surface)))),
                Space::with_width(12),
                text(format!("{}/{}", done, total)).size(theme::TEXT_CAPTION).style(p.text_muted)
            ].align_items(Alignment::Center)
        );
    } else {
        index_content = index_content.push(
            button(text("Re-index Library").style(p.text_primary))
                .on_press(on_scan)
                .padding([12, 20])
                .style(iced::theme::Button::Custom(Box::new(PrimaryBtnStyle(p.elevated, p.highlight))))
        );
    }

    let index_section = container(index_content).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(index_section);

    // Accent Color Section
    let colors = [
        ("#9d8cff", "Purple"),
        ("#ff8fa3", "Rose"),
        ("#7dcfff", "Cyan"),
        ("#ffb378", "Amber"),
        ("#9ece6a", "Green"),
    ];

    let mut swatches = row![].spacing(12);
    for (hex, _name) in colors {
        let color = color_from_hex(hex);
        let is_selected = state.art_tint == color;
        
        let swatch = button(Space::new(Length::Fixed(32.0), Length::Fixed(32.0)))
            .on_press(on_set_accent(color))
            .style(iced::theme::Button::Custom(Box::new(SwatchStyle(color, is_selected))));
            
        swatches = swatches.push(swatch);
    }

    let accent_section = container(column![
        text("Accent Color").font(theme::FONT_ROUNDED).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(16),
        swatches
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(accent_section);

    // About Section
    let about_section = container(column![
        text("About").font(theme::FONT_ROUNDED).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(16),
        text("Musico v0.1.0").font(theme::FONT_TEXT).style(p.text_secondary),
        text("Powered by Iced 0.12, Symphonia, and pure Rust.").font(theme::FONT_TEXT).size(theme::TEXT_CAPTION).style(p.text_muted),
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(about_section);

    container(content).width(Length::Fill).height(Length::Fill).into()
}

fn color_from_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_rgb8(r, g, b)
}

struct InputBgStyle(Color);
impl iced::widget::container::StyleSheet for InputBgStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: theme::RADIUS_MD.into(),
            },
            ..Default::default()
        }
    }
}

struct PrimaryBtnStyle(Color, Color);
impl iced::widget::button::StyleSheet for PrimaryBtnStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: theme::RADIUS_MD.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.1.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: theme::RADIUS_MD.into(),
            },
            ..Default::default()
        }
    }
}

struct ProgressStyle(Color, Color);
impl iced::widget::progress_bar::StyleSheet for ProgressStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::progress_bar::Appearance {
        iced::widget::progress_bar::Appearance {
            background: self.1.into(),
            bar: self.0.into(),
            border_radius: 4.0.into(),
        }
    }
}

struct SwatchStyle(Color, bool);
impl iced::widget::button::StyleSheet for SwatchStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: Color::WHITE,
                width: if self.1 { 2.0 } else { 0.0 },
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: Color { a: 0.5, ..Color::WHITE },
                width: 2.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}
