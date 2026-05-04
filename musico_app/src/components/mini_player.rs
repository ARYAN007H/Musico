//! Compact mini-player bar — shown at the bottom of the window when the user
//! is not on the Now Playing view. Shows song info, playback controls, and a
//! seek bar in a single row. Enhanced with glassmorphism and top progress line.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};
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

    // ── Top progress line (thin accent bar showing position) ──────────────
    let progress_ratio = if state.duration_secs > 0.0 {
        (state.position_secs / state.duration_secs).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let progress_line = container(
        row![
            container(Space::new(Length::FillPortion((progress_ratio * 1000.0) as u16), Length::Fixed(3.0)))
                .style(iced::theme::Container::Custom(Box::new(ProgressLineStyle(accent)))),
            container(Space::new(Length::FillPortion(((1.0 - progress_ratio) * 1000.0) as u16), Length::Fixed(3.0)))
                .style(iced::theme::Container::Custom(Box::new(ProgressLineStyle(
                    theme::with_alpha(accent, 0.12)
                )))),
        ]
    )
    .width(Length::Fill);

    // ── Album art thumbnail ──────────────────────────────────────────────
    let art_widget: Element<'a, Message> = if let Some(bytes) = &song.cover_art {
        let handle = iced::widget::image::Handle::from_memory(bytes.clone());
        container(
            iced::widget::image(handle)
                .width(Length::Fixed(42.0))
                .height(Length::Fixed(42.0))
        )
        .style(|_: &iced::Theme| iced::widget::container::Appearance {
            border: Border { radius: 8.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
    } else {
        container(
            svg(svg::Handle::from_memory(icons::NOW_PLAYING))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0))
                .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(
                    theme::with_alpha(accent, 0.6)
                )))),
        )
        .width(Length::Fixed(42.0))
        .height(Length::Fixed(42.0))
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(MiniArtPlaceholderStyle(accent))))
        .into()
    };

    // ── Song info ────────────────────────────────────────────────────────
    let info = column![
        text(&song.title)
            .font(ctx.font_rounded)
            .size(13.0)
            .style(p.text_primary)
            .shaping(text::Shaping::Advanced),
        text(&song.artist)
            .font(ctx.font_text)
            .size(11.0)
            .style(p.text_muted)
            .shaping(text::Shaping::Advanced),
    ].spacing(2).width(Length::Fixed(200.0));

    // ── Transport controls ───────────────────────────────────────────────
    let play_icon = if is_playing { icons::PAUSE } else { icons::PLAY };
    let play_msg = if is_playing { Message::Pause } else { Message::Resume };

    let controls = row![
        button(svg_icon(icons::PREV, 16, p.text_primary))
            .on_press(Message::Previous)
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(8),
        Space::with_width(6),
        button(
            container(svg_icon(play_icon, 20, Color::WHITE))
                .width(Length::Fixed(36.0))
                .height(Length::Fixed(36.0))
                .center_x()
                .center_y()
        )
            .on_press(play_msg)
            .style(iced::theme::Button::Custom(Box::new(MiniPlayBtnStyle(accent))))
            .padding(0),
        Space::with_width(6),
        button(svg_icon(icons::NEXT, 16, p.text_primary))
            .on_press(Message::Next)
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(8),
    ].align_items(Alignment::Center);

    // ── Seek bar + time ──────────────────────────────────────────────────
    let seek = column![
        seek_bar(state.position_secs, state.duration_secs, accent, Message::Seek),
        row![
            text(format_time(state.position_secs)).font(ctx.font_rounded).size(9.0).style(p.text_muted),
            Space::with_width(Length::Fill),
            text(format_time(state.duration_secs)).font(ctx.font_rounded).size(9.0).style(p.text_muted),
        ]
    ].spacing(2).width(Length::Fill);

    // ── Navigate to now playing ──────────────────────────────────────────
    let go_to_np = button(
        svg_icon(icons::NOW_PLAYING, 16, p.text_secondary)
    )
    .on_press(Message::NavigateTo(crate::state::View::NowPlaying))
    .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
    .padding(8);

    let bar_content = row![
        art_widget,
        Space::with_width(12),
        info,
        Space::with_width(16),
        controls,
        Space::with_width(16),
        seek,
        Space::with_width(8),
        go_to_np,
    ]
    .align_items(Alignment::Center)
    .padding([10, 20]);

    let bar = container(
        column![
            progress_line,
            bar_content,
        ].spacing(0)
    )
    .width(Length::Fill)
    .style(iced::theme::Container::Custom(Box::new(MiniBarStyle(accent))));

    Some(bar.into())
}

// ─── Styles ──────────────────────────────────────────────────────────────────

struct ProgressLineStyle(Color);
impl iced::widget::container::StyleSheet for ProgressLineStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            ..Default::default()
        }
    }
}

struct MiniArtPlaceholderStyle(Color);
impl iced::widget::container::StyleSheet for MiniArtPlaceholderStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(theme::with_alpha(self.0, 0.1).into()),
            border: Border {
                radius: 8.0.into(),
                color: theme::with_alpha(self.0, 0.15),
                width: 1.0,
            },
            ..Default::default()
        }
    }
}

struct MiniBarStyle(Color);
impl iced::widget::container::StyleSheet for MiniBarStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(Color {
                a: 0.92,
                ..theme::BASE
            })),
            border: Border {
                color: theme::with_alpha(self.0, 0.12),
                width: 1.0,
                radius: 18.0.into(),
            },
            shadow: iced::Shadow {
                color: Color { a: 0.3, ..theme::BASE },
                offset: iced::Vector { x: 0.0, y: -4.0 },
                blur_radius: 20.0,
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
            background: Some(self.0.into()),
            border: Border { radius: 50.0.into(), ..Default::default() },
            shadow: iced::Shadow {
                color: Color { a: 0.3, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 2.0 },
                blur_radius: 8.0,
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        let brighter = Color {
            r: (self.0.r * 1.15).min(1.0),
            g: (self.0.g * 1.15).min(1.0),
            b: (self.0.b * 1.15).min(1.0),
            a: self.0.a,
        };
        iced::widget::button::Appearance {
            background: Some(brighter.into()),
            border: Border {
                color: theme::with_alpha(self.0, 0.3),
                width: 1.0,
                radius: 50.0.into(),
            },
            shadow: iced::Shadow {
                color: Color { a: 0.4, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 4.0 },
                blur_radius: 16.0,
            },
            ..Default::default()
        }
    }
}
