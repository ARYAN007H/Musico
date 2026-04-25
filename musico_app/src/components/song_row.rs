use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};
use musico_recommender::SongRecord;
use crate::theme::{self, Palette};
use crate::components::seek_bar::format_time;

pub fn song_row<'a, Message: 'a + Clone>(
    song: &SongRecord,
    index: usize,
    is_playing: bool,
    on_play: impl Fn(SongRecord) -> Message,
    on_queue: Option<impl Fn(SongRecord) -> Message>,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    // In a real app we'd load the thumbnail asynchronously from cache.
    // For now we use a colored placeholder if no art is readily available in memory.
    let thumb = container(Space::new(Length::Fixed(48.0), Length::Fixed(48.0)))
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0))
        .style(iced::theme::Container::Custom(Box::new(ThumbStyle {
            radius: theme::RADIUS_SM,
            bg: p.surface,
        })));

    let title_color = if is_playing { p.accent } else { p.text_primary };

    let middle = column![
        text(&song.title).font(theme::FONT_TEXT).size(theme::TEXT_BODY).style(title_color),
        text(format!("{} · {}", song.artist, song.album))
            .font(theme::FONT_TEXT)
            .size(theme::TEXT_CAPTION)
            .style(p.text_muted)
    ]
    .spacing(4)
    .width(Length::Fill);

    let duration_text = text(format_time(song.duration_secs as f32))
        .font(theme::FONT_TEXT)
        .size(theme::TEXT_CAPTION)
        .style(p.text_muted);

    let mut right_col = row![duration_text].align_items(Alignment::Center).spacing(8);

    if let Some(queue_fn) = on_queue {
        let queue_btn = button(text("+").size(16).style(p.text_secondary))
            .on_press(queue_fn(song.clone()))
            .style(iced::theme::Button::Text);
        right_col = right_col.push(queue_btn);
    }

    let bg_color = if index % 2 == 0 {
        p.base
    } else {
        // Approximate 30% mix of elevated over base
        Color {
            r: p.base.r * 0.7 + p.elevated.r * 0.3,
            g: p.base.g * 0.7 + p.elevated.g * 0.3,
            b: p.base.b * 0.7 + p.elevated.b * 0.3,
            a: 1.0,
        }
    };

    let song_clone = song.clone();

    button(
        row![
            thumb,
            middle,
            right_col,
        ]
        .align_items(Alignment::Center)
        .spacing(12)
        .padding([8, 12])
    )
    .on_press(on_play(song_clone))
    .width(Length::Fill)
    .style(iced::theme::Button::Custom(Box::new(RowStyle {
        bg: bg_color,
        hover_bg: p.highlight,
    })))
    .into()
}

struct ThumbStyle {
    radius: f32,
    bg: Color,
}

impl iced::widget::container::StyleSheet for ThumbStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: self.radius.into(),
            },
            ..Default::default()
        }
    }
}

struct RowStyle {
    bg: Color,
    hover_bg: Color,
}

impl iced::widget::button::StyleSheet for RowStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
    
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.hover_bg.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    }
}
