use iced::widget::{button, column, container, row, text, Space, scrollable};
use iced::{Alignment, Color, Element, Length};
use crate::state::{AppState, ShuffleMode, RepeatMode};
use crate::theme::{self, Palette};
use crate::components::seek_bar::{seek_bar, format_time};
use crate::components::art_canvas::art_canvas;
use crate::components::song_row::song_row;
use musico_recommender::SongRecord;
use crate::icons;
use iced::widget::svg;

fn svg_icon<'a, Message: 'a + Clone>(bytes: &'static [u8], size: u16, color: iced::Color) -> Element<'a, Message> {
    svg(svg::Handle::from_memory(bytes))
        .width(Length::Fixed(size as f32))
        .height(Length::Fixed(size as f32))
        .style(iced::theme::Svg::Custom(Box::new(crate::theme::SvgStyle(color))))
        .into()
}

pub fn now_playing<'a, Message: 'a + Clone>(
    state: &AppState,
    on_toggle_play: Message,
    on_previous: Message,
    on_next: Message,
    on_seek: impl Fn(f32) -> Message + 'a,
    on_play_recommendation: impl Fn(SongRecord) -> Message,
    on_queue_recommendation: impl Fn(SongRecord) -> Message,
    on_toggle_shuffle: Message,
    on_toggle_repeat: Message,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    // Album Art — scale to available space
    let art_size = (state.window_height * 0.38).clamp(180.0, 360.0);
    
    // Build cover art widget — use actual image if available.
    let art_handle = state.current_song.as_ref()
        .and_then(|s| s.cover_art.as_ref())
        .map(|bytes| iced::widget::image::Handle::from_memory(bytes.clone()));
    
    let art = art_canvas(art_handle, art_size, 24.0, state.art_tint);
    
    // Ambient glow ring around album art
    let art_container = container(art)
        .padding(8)
        .style(iced::theme::Container::Custom(Box::new(GlowStyle(state.art_tint))));

    let mut content = column![].align_items(Alignment::Center).spacing(20);

    let (title_text, artist_text) = match &state.current_song {
        Some(song) => (song.title.clone(), song.artist.clone()),
        None => ("Not Playing".to_string(), "Select a track from your library".to_string()),
    };

    let title_col = column![
        text(title_text).font(theme::FONT_DISPLAY).size(28.0).style(p.text_primary),
        text(artist_text).font(theme::FONT_TEXT).size(16.0).style(p.text_muted)
    ].spacing(6).align_items(Alignment::Center);

    let actions_row = row![
        button(svg_icon(icons::HEART, 20, p.text_muted)).style(iced::theme::Button::Custom(Box::new(theme::TransportButton))).padding(12),
        button(svg_icon(icons::MORE, 20, p.text_muted)).style(iced::theme::Button::Custom(Box::new(theme::TransportButton))).padding(12),
    ].spacing(12);

    let meta_row = column![
        title_col,
        Space::with_height(8),
        actions_row
    ]
    .width(Length::Fill)
    .align_items(Alignment::Center);

    // Seek bar + timestamps
    let seek_container = if state.current_song.is_some() {
        column![
            seek_bar(state.position_secs, state.duration_secs, on_seek),
            row![
                text(format_time(state.position_secs)).font(theme::FONT_ROUNDED).size(11.0).style(p.text_secondary),
                Space::with_width(Length::Fill),
                text(format_time(state.duration_secs)).font(theme::FONT_ROUNDED).size(11.0).style(p.text_secondary),
            ]
        ].spacing(5).width(Length::Fill)
    } else {
        column![].width(Length::Fill)
    };

    let is_playing = matches!(state.playback_status, musico_playback::PlaybackStatus::Playing);
    
    // Shuffle/repeat button colors based on state
    let shuffle_color = match state.shuffle_mode {
        ShuffleMode::Off => p.text_secondary,
        ShuffleMode::Shuffle => p.accent,
        ShuffleMode::SmartRadio => Color::from_rgb(0.431, 0.906, 0.718), // Green for smart radio
    };
    let repeat_color = match state.repeat_mode {
        RepeatMode::Off => p.text_secondary,
        _ => p.accent,
    };

    let controls = row![
        button(svg_icon(icons::SHUFFLE, 20, shuffle_color))
            .on_press(on_toggle_shuffle.clone())
            .style(iced::theme::Button::Custom(Box::new(theme::TransportButton)))
            .padding(12),
        Space::with_width(32),
        button(svg_icon(icons::PREV, 26, p.text_primary)).on_press(on_previous.clone()).style(iced::theme::Button::Custom(Box::new(theme::TransportButton))).padding(12),
        Space::with_width(32),
        play_button(is_playing, state.art_tint, on_toggle_play.clone()),
        Space::with_width(32),
        button(svg_icon(icons::NEXT, 26, p.text_primary)).on_press(on_next.clone()).style(iced::theme::Button::Custom(Box::new(theme::TransportButton))).padding(12),
        Space::with_width(32),
        button(svg_icon(icons::REPEAT, 20, repeat_color))
            .on_press(on_toggle_repeat.clone())
            .style(iced::theme::Button::Custom(Box::new(theme::TransportButton)))
            .padding(12),
    ].align_items(Alignment::Center);

    // Shuffle/repeat mode indicator
    let mode_text = {
        let shuffle_label = match state.shuffle_mode {
            ShuffleMode::Off => "",
            ShuffleMode::Shuffle => "Shuffle",
            ShuffleMode::SmartRadio => "Smart Radio ✦",
        };
        let repeat_label = match state.repeat_mode {
            RepeatMode::Off => "",
            RepeatMode::One => "Repeat One",
            RepeatMode::All => "Repeat All",
        };
        let combined = [shuffle_label, repeat_label]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<&str>>()
            .join(" · ");
        combined
    };

    content = content.push(art_container)
        .push(meta_row)
        .push(seek_container)
        .push(controls);

    if !mode_text.is_empty() {
        content = content.push(
            text(mode_text)
                .font(theme::FONT_ROUNDED)
                .size(theme::TEXT_CAPTION)
                .style(p.text_muted)
        );
    }

    // Recommendations
    if !state.recommendations.is_empty() {
        let mut recs_col = column![
            Space::with_height(20),
            container(Space::with_height(1)).width(Length::Fill).style(iced::theme::Container::Custom(Box::new(DividerStyle(p.border_subtle)))),
            Space::with_height(20),
            text("UP NEXT").font(theme::FONT_ROUNDED).size(theme::TEXT_CAPTION).style(p.text_muted)
        ].spacing(10).width(Length::Fill);

        for (i, rec) in state.recommendations.iter().take(5).enumerate() {
            recs_col = recs_col.push(song_row(
                &rec.record,
                i,
                false, // is_playing
                &on_play_recommendation,
                Some(&on_queue_recommendation)
            ));
        }

        content = content.push(recs_col);
    }

    let scrollable_content = scrollable(
        container(content)
            .width(Length::Fill)
            .max_width(700.0)
            .padding(40)
            .center_x()
    );

    container(scrollable_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .style(iced::theme::Container::Custom(Box::new(NowPlayingBgStyle(state.art_tint, p.base))))
        .into()
}

fn play_button<'a, Message: 'a + Clone>(
    is_playing: bool,
    accent: iced::Color,
    on_press: Message,
) -> Element<'a, Message> {
    let icon = if is_playing { icons::PAUSE } else { icons::PLAY };
    
    button(
        container(svg_icon(icon, 28, Color::WHITE))
            .width(Length::Fixed(72.0))
            .height(Length::Fixed(72.0))
            .center_x()
            .center_y()
    )
    .on_press(on_press)
    .style(iced::theme::Button::Custom(Box::new(PlayButtonStyle(accent))))
    .into()
}

struct GlowStyle(iced::Color);
impl iced::widget::container::StyleSheet for GlowStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Color { a: 0.25, ..self.0 }.into()),
            border: iced::Border {
                radius: (24.0 + 8.0).into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.4, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 0.0 },
                blur_radius: 60.0,
            },
            ..Default::default()
        }
    }
}

struct DividerStyle(iced::Color);
impl iced::widget::container::StyleSheet for DividerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            ..Default::default()
        }
    }
}

struct PlayButtonStyle(iced::Color);
impl iced::widget::button::StyleSheet for PlayButtonStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 36.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.5, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 4.0 },
                blur_radius: 20.0,
            },
            ..Default::default()
        }
    }
    
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let mut hover_color = self.0;
        hover_color.r = (hover_color.r * 1.15).clamp(0.0, 1.0);
        hover_color.g = (hover_color.g * 1.15).clamp(0.0, 1.0);
        hover_color.b = (hover_color.b * 1.05).clamp(0.0, 1.0);
        
        iced::widget::button::Appearance {
            background: Some(hover_color.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 36.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.6, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 6.0 },
                blur_radius: 28.0,
            },
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let mut press_color = self.0;
        press_color.r = (press_color.r * 0.9).clamp(0.0, 1.0);
        press_color.g = (press_color.g * 0.9).clamp(0.0, 1.0);
        press_color.b = (press_color.b * 0.9).clamp(0.0, 1.0);

        iced::widget::button::Appearance {
            background: Some(press_color.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 36.0.into(),
            },
            ..Default::default()
        }
    }
}

struct NowPlayingBgStyle(iced::Color, iced::Color);
impl iced::widget::container::StyleSheet for NowPlayingBgStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let mix_factor = 0.08; // 8% tint over base for subtle ambient color
        let r = self.1.r * (1.0 - mix_factor) + self.0.r * mix_factor;
        let g = self.1.g * (1.0 - mix_factor) + self.0.g * mix_factor;
        let b = self.1.b * (1.0 - mix_factor) + self.0.b * mix_factor;
        
        iced::widget::container::Appearance {
            background: Some(iced::Color::from_rgb(r, g, b).into()),
            border: iced::Border {
                color: theme::BORDER_SUBTLE,
                width: 1.0,
                radius: 24.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.3, ..theme::BASE },
                offset: iced::Vector { x: 0.0, y: 10.0 },
                blur_radius: 30.0,
            },
            ..Default::default()
        }
    }
}
