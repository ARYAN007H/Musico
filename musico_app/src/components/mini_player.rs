use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length};
use musico_playback::SongInfo;
use crate::theme::{self, Palette};

pub fn mini_player<'a, Message: 'a + Clone>(
    song: Option<&SongInfo>,
    is_playing: bool,
    on_toggle_play: Message,
    on_next: Message,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    if let Some(s) = song {
        let title = text(&s.title)
            .font(theme::FONT_TEXT)
            .size(theme::TEXT_BODY)
            .style(p.text_primary);

        let artist = text(&s.artist)
            .font(theme::FONT_TEXT)
            .size(theme::TEXT_CAPTION)
            .style(p.text_muted);

        let info_col = column![title, artist].spacing(2);

        let play_icon = if is_playing { "⏸" } else { "▶" };
        let play_btn = button(text(play_icon).size(20))
            .on_press(on_toggle_play)
            .style(iced::theme::Button::Text);

        let next_btn = button(text("⏭").size(20))
            .on_press(on_next)
            .style(iced::theme::Button::Text);

        container(
            row![
                info_col,
                Space::with_width(Length::Fill),
                play_btn,
                Space::with_width(16),
                next_btn,
            ]
            .align_items(Alignment::Center)
            .padding([8, 16])
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(MiniPlayerStyle {
            bg: p.surface,
            border: p.border_subtle,
        })))
        .into()
    } else {
        container(Space::with_height(Length::Fixed(60.0))).into()
    }
}

struct MiniPlayerStyle {
    bg: iced::Color,
    border: iced::Color,
}

impl iced::widget::container::StyleSheet for MiniPlayerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: self.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }
    }
}
