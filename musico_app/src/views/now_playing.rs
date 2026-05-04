use iced::widget::{button, column, container, row, slider, text, Space, scrollable};
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
    on_set_volume: impl Fn(f32) -> Message + 'a,
    on_toggle_like: Message,
    on_add_to_queue: impl Fn(SongRecord) -> Message,
    on_play_recommendation: impl Fn(SongRecord) -> Message,
    on_queue_recommendation: impl Fn(SongRecord) -> Message,
    on_toggle_shuffle: Message,
    on_toggle_repeat: Message,
) -> Element<'a, Message> {
    let p = Palette::from_color_palette(&state.color_palette);
    let accent = state.art_tint;
    let ctx = state.theme_ctx();

    if state.current_song.is_none() {
        return empty_state(&p, &ctx, accent);
    }

    // Album Art — scale to available space
    let art_size = (state.window_height * 0.38).clamp(180.0, 360.0);
    
    let art_handle = state.cached_art_handle.clone();
    
    let art = art_canvas(art_handle, art_size, ctx.radius_lg + 8.0, accent);
    
    // Deeper glow ring around album art
    let art_container = container(art)
        .padding(8)
        .style(iced::theme::Container::Custom(Box::new(GlowStyle(accent))));

    let mut content = column![].align_items(Alignment::Center).spacing(18);

    let (title_text, artist_text) = match &state.current_song {
        Some(song) => (song.title.clone(), song.artist.clone()),
        None => unreachable!(),
    };

    // Playing indicator next to title
    let is_playing = matches!(state.playback_status, musico_playback::PlaybackStatus::Playing);
    
    let title_row = if is_playing {
        row![
            text("♫").size(14.0).style(accent),
            Space::with_width(6),
            text(title_text).font(ctx.font_display).size(32.0).style(p.text_primary),
        ].align_items(Alignment::Center)
    } else {
        row![
            text(title_text).font(ctx.font_display).size(32.0).style(p.text_primary),
        ].align_items(Alignment::Center)
    };

    let title_col = column![
        title_row,
        text(artist_text).font(ctx.font_text).size(16.0).style(p.text_muted)
    ].spacing(6).align_items(Alignment::Center);

    let heart_color = if state.is_liked { accent } else { p.text_muted };

    let more_msg = {
        let song = state.current_song.as_ref().unwrap();
        state.library.iter()
            .find(|r| r.id == song.id)
            .map(|r| on_add_to_queue(r.clone()))
    };

    let mut more_btn = button(svg_icon(icons::MORE, 20, p.text_muted))
        .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
        .padding(12);
    if let Some(msg) = more_msg {
        more_btn = more_btn.on_press(msg);
    }

    let actions_row = row![
        button(svg_icon(icons::HEART, 20, heart_color))
            .on_press(on_toggle_like.clone())
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(12),
        more_btn,
    ].spacing(12);

    let meta_row = column![
        title_col,
        Space::with_height(6),
        actions_row
    ]
    .width(Length::Fill)
    .align_items(Alignment::Center);

    // Seek bar + timestamps
    let seek_container = column![
        seek_bar(state.position_secs, state.duration_secs, accent, on_seek),
        row![
            text(format_time(state.position_secs)).font(ctx.font_rounded).size(11.0).style(p.text_secondary),
            Space::with_width(Length::Fill),
            text(format_time(state.duration_secs)).font(ctx.font_rounded).size(11.0).style(p.text_secondary),
        ]
    ].spacing(5).width(Length::Fill);

    // Shuffle/repeat button colors based on state
    let shuffle_color = match state.shuffle_mode {
        ShuffleMode::Off => p.text_secondary,
        ShuffleMode::Shuffle => accent,
        ShuffleMode::SmartRadio => Color::from_rgb(0.431, 0.906, 0.718),
    };
    let repeat_color = match state.repeat_mode {
        RepeatMode::Off => p.text_secondary,
        _ => accent,
    };

    let controls = row![
        button(svg_icon(icons::SHUFFLE, 20, shuffle_color))
            .on_press(on_toggle_shuffle.clone())
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(12),
        Space::with_width(24),
        button(svg_icon(icons::PREV, 26, p.text_primary))
            .on_press(on_previous.clone())
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(12),
        Space::with_width(24),
        play_button(is_playing, accent, on_toggle_play.clone()),
        Space::with_width(24),
        button(svg_icon(icons::NEXT, 26, p.text_primary))
            .on_press(on_next.clone())
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
            .padding(12),
        Space::with_width(24),
        button(svg_icon(icons::REPEAT, 20, repeat_color))
            .on_press(on_toggle_repeat.clone())
            .style(iced::theme::Button::Custom(Box::new(theme::AccentTransportButton(accent))))
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
        [shuffle_label, repeat_label]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<&str>>()
            .join(" · ")
    };

    // Volume slider
    let vol_icon = if state.volume < 0.01 {
        icons::MUTE
    } else if state.volume < 0.5 {
        icons::VOL_LOW
    } else {
        icons::VOL_HIGH
    };

    let volume_row = row![
        svg_icon::<Message>(vol_icon, 16, p.text_muted),
        Space::with_width(8),
        slider(0.0..=1.0, state.volume, on_set_volume)
            .width(Length::Fixed(140.0))
            .style(iced::theme::Slider::Custom(Box::new(theme::VolumeSliderStyle(accent)))),
    ]
    .align_items(Alignment::Center)
    .spacing(0);

    content = content.push(art_container)
        .push(meta_row)
        .push(seek_container)
        .push(controls);

    if !mode_text.is_empty() {
        content = content.push(
            text(mode_text)
                .font(ctx.font_rounded)
                .size(theme::TEXT_CAPTION)
                .style(p.text_muted)
        );
    }

    content = content.push(volume_row);

    // ── Lyrics Panel ──────────────────────────────────────────────────────
    match &state.lyrics {
        crate::lyrics::Lyrics::Synced(lines) if state.show_lyrics && !lines.is_empty() => {
            let current_idx = crate::lyrics::current_line_index(lines, state.position_secs);

            let mut lyrics_col = column![
                Space::with_height(12),
                container(Space::with_height(1)).width(Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(DividerStyle(p.border_subtle)))),
                Space::with_height(12),
                text("LYRICS").font(ctx.font_rounded).size(theme::TEXT_CAPTION).style(p.text_muted),
                Space::with_height(8),
            ].spacing(2).width(Length::Fill).align_items(Alignment::Center);

            // Show a window of ~7 lines around the current line.
            let active = current_idx.unwrap_or(0);
            let start = active.saturating_sub(3);
            let end = (active + 4).min(lines.len());

            for i in start..end {
                let line = &lines[i];
                let is_active = Some(i) == current_idx;
                let line_color = if is_active {
                    accent
                } else {
                    theme::with_alpha(p.text_secondary, 0.5)
                };
                let font_size = if is_active { 18.0 } else { 14.0 };

                lyrics_col = lyrics_col.push(
                    text(&line.text)
                        .font(if is_active { ctx.font_display } else { ctx.font_text })
                        .size(font_size)
                        .style(line_color)
                );
            }

            content = content.push(
                container(lyrics_col).padding([8, 20]).width(Length::Fill)
            );
        }
        crate::lyrics::Lyrics::Unsynced(text_str) if state.show_lyrics => {
            content = content.push(
                container(
                    column![
                        Space::with_height(12),
                        container(Space::with_height(1)).width(Length::Fill)
                            .style(iced::theme::Container::Custom(Box::new(DividerStyle(p.border_subtle)))),
                        Space::with_height(12),
                        text("LYRICS").font(ctx.font_rounded).size(theme::TEXT_CAPTION).style(p.text_muted),
                        Space::with_height(8),
                        text(text_str.as_str())
                            .font(ctx.font_text)
                            .size(14.0)
                            .style(p.text_secondary),
                    ].width(Length::Fill).align_items(Alignment::Center)
                ).padding([8, 20]).width(Length::Fill)
            );
        }
        _ => {}
    }

    // Recommendations
    if !state.recommendations.is_empty() {
        let mut recs_col = column![
            Space::with_height(16),
            container(Space::with_height(1)).width(Length::Fill).style(iced::theme::Container::Custom(Box::new(DividerStyle(p.border_subtle)))),
            Space::with_height(16),
            text("UP NEXT").font(ctx.font_rounded).size(theme::TEXT_CAPTION).style(p.text_muted)
        ].spacing(10).width(Length::Fill);

        for (i, rec) in state.recommendations.iter().take(5).enumerate() {
            recs_col = recs_col.push(song_row(
                &rec.record,
                i,
                false,
                &on_play_recommendation,
                Some(&on_queue_recommendation),
                accent,
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
        .style(iced::theme::Container::Custom(Box::new(NowPlayingBgStyle(accent, theme::BASE))))
        .into()
}

// ─── Empty State ─────────────────────────────────────────────────────────────

fn empty_state<'a, Message: 'a + Clone>(p: &Palette, ctx: &theme::ThemeCtx, accent: Color) -> Element<'a, Message> {
    let icon = container(
        column![
            svg(svg::Handle::from_memory(icons::NOW_PLAYING))
                .width(Length::Fixed(48.0))
                .height(Length::Fixed(48.0))
                .style(iced::theme::Svg::Custom(Box::new(crate::theme::SvgStyle(
                    theme::with_alpha(accent, 0.5),
                )))),
        ].align_items(Alignment::Center)
    )
    .width(Length::Fixed(120.0))
    .height(Length::Fixed(120.0))
    .center_x()
    .center_y()
    .style(iced::theme::Container::Custom(Box::new(EmptyArtStyle(accent))));

    let content = column![
        Space::with_height(80),
        icon,
        Space::with_height(24),
        text("Nothing playing").font(ctx.font_display).size(24.0).style(p.text_primary),
        Space::with_height(6),
        text("Pick something from your library to get started")
            .font(ctx.font_text)
            .size(14.0)
            .style(p.text_muted),
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(NowPlayingBgStyle(accent, theme::BASE))))
        .into()
}

struct EmptyArtStyle(Color);
impl iced::widget::container::StyleSheet for EmptyArtStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(theme::with_alpha(self.0, 0.08).into()),
            border: iced::Border {
                radius: 24.0.into(),
                color: theme::with_alpha(self.0, 0.15),
                width: 1.0,
            },
            ..Default::default()
        }
    }
}

// ─── Play Button ─────────────────────────────────────────────────────────────

fn play_button<'a, Message: 'a + Clone>(
    is_playing: bool,
    accent: iced::Color,
    on_press: Message,
) -> Element<'a, Message> {
    let icon = if is_playing { icons::PAUSE } else { icons::PLAY };
    
    button(
        container(svg_icon(icon, 30, Color::WHITE))
            .width(Length::Fixed(80.0))
            .height(Length::Fixed(80.0))
            .center_x()
            .center_y()
    )
    .on_press(on_press)
    .style(iced::theme::Button::Custom(Box::new(PlayButtonStyle(accent))))
    .into()
}

// ─── Styles ──────────────────────────────────────────────────────────────────

struct GlowStyle(iced::Color);
impl iced::widget::container::StyleSheet for GlowStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Color { a: 0.20, ..self.0 }.into()),
            border: iced::Border {
                radius: 32.0.into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.5, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 24.0 },
                blur_radius: 100.0,
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
                radius: 40.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.5, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 6.0 },
                blur_radius: 24.0,
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
                radius: 40.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.6, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 8.0 },
                blur_radius: 32.0,
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
                radius: 40.0.into(),
            },
            ..Default::default()
        }
    }
}

struct NowPlayingBgStyle(iced::Color, iced::Color);
impl iced::widget::container::StyleSheet for NowPlayingBgStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        // Deeper ambient blend: accent color suffused into the base for
        // a warm, immersive "spotlight" effect
        let mix = 0.14;
        let r = self.1.r * (1.0 - mix) + self.0.r * mix;
        let g = self.1.g * (1.0 - mix) + self.0.g * mix;
        let b = self.1.b * (1.0 - mix) + self.0.b * mix;
        
        iced::widget::container::Appearance {
            background: Some(iced::Color::from_rgb(r, g, b).into()),
            border: iced::Border {
                color: theme::with_alpha(self.0, 0.08),
                width: 1.0,
                radius: 24.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.2, ..self.0 },
                offset: iced::Vector { x: 0.0, y: 0.0 },
                blur_radius: 60.0,
            },
            ..Default::default()
        }
    }
}
