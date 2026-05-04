//! Compact mini-player bar — shown at the bottom of the window when the user
//! is not on the Now Playing view. Shows song info, playback controls, and a
//! seek bar in a single row.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};
use crate::state::AppState;
use crate::theme::{self, Palette};
use crate::app::Message;
use crate::components::seek_bar::{seek_bar, format_time};
use crate::icons;
use iced::widget::svg;

fn svg_icon<'a>(bytes: &'static [u8], size: u16, color: Color) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(bytes))
        .width(Length::Fixed(size as f32))
        .height(Length::Fixed(size as f32))
        .style(iced::theme::Svg::Custom(Box::new(crate::theme::SvgStyle(color))))
        .into()
}

/// Build the mini-player bar. Returns `None` if no song is playing.
pub fn mini_player_bar<'a>(state: &AppState) -> Option<Element<'a, Message>> {
    let song = state.current_song.as_ref()?;

    let p = Palette::from_color_palette(&state.color_palette);
    let ctx = state.theme_ctx();
    let accent = state.art_tint;
    let is_playing = matches!(state.playback_status, musico_playback::PlaybackStatus::Playing);

    // Song info (left side)
    let info = row![
        column![
            text(&song.title)
                .font(ctx.font_rounded)
                .size(13.0)
                .style(p.text_primary),
            text(&song.artist)
                .font(ctx.font_text)
                .size(11.0)
                .style(p.text_muted),
        ].spacing(2).width(Length::Fixed(180.0)),
    ].align_items(Alignment::Center);

    // Transport controls (center)
    let play_icon = if is_playing { icons::PAUSE } else { icons::PLAY };
    let play_msg = if is_playing { Message::Pause } else { Message::Resume };

    let controls = row![
        button(svg_icon(icons::PREV, 16, p.text_primary))
            .on_press(Message::Previous)
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(8),
        Space::with_width(8),
        button(svg_icon(play_icon, 22, p.text_primary))
            .on_press(play_msg)
            .style(iced::theme::Button::Custom(Box::new(MiniPlayBtnStyle(accent))))
            .padding(8),
        Space::with_width(8),
        button(svg_icon(icons::NEXT, 16, p.text_primary))
            .on_press(Message::Next)
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(8),
    ].align_items(Alignment::Center);

    // Seek bar + time (right side)
    let seek = column![
        seek_bar(state.position_secs, state.duration_secs, accent, Message::Seek),
        row![
            text(format_time(state.position_secs)).font(ctx.font_rounded).size(9.0).style(p.text_muted),
            Space::with_width(Length::Fill),
            text(format_time(state.duration_secs)).font(ctx.font_rounded).size(9.0).style(p.text_muted),
        ]
    ].spacing(2).width(Length::Fill);

    // Navigate to now playing on click
    let go_to_np = button(
        text("↗").size(16.0).style(p.text_muted)
    )
    .on_press(Message::NavigateTo(crate::state::View::NowPlaying))
    .style(iced::theme::Button::Text)
    .padding(8);

    let bar = container(
        row![
            info,
            Space::with_width(16),
            controls,
            Space::with_width(16),
            seek,
            Space::with_width(8),
            go_to_np,
        ]
        .align_items(Alignment::Center)
        .padding([8, 20])
    )
    .width(Length::Fill)
    .style(iced::theme::Container::Custom(Box::new(MiniBarStyle(accent))));

    Some(bar.into())
}

// ─── Styles ──────────────────────────────────────────────────────────────────

struct MiniBarStyle(Color);
impl iced::widget::container::StyleSheet for MiniBarStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Color { a: 0.85, ..theme::BASE }.into()),
            border: iced::Border {
                color: theme::with_alpha(self.0, 0.15),
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

struct MiniPlayBtnStyle(Color);
impl iced::widget::button::StyleSheet for MiniPlayBtnStyle {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.0, 0.15).into()),
            border: iced::Border { radius: 50.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.0, 0.25).into()),
            border: iced::Border {
                color: theme::with_alpha(self.0, 0.3),
                width: 1.0,
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
}
