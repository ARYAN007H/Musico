use iced::widget::{button, column, container, row, text, Space, scrollable};
use iced::{Alignment, Color, Element, Length};
use crate::state::AppState;
use crate::theme::{self, Palette};
use crate::components::seek_bar::{seek_bar, format_time};
use crate::components::art_canvas::art_canvas;
use crate::components::song_row::song_row;
use musico_recommender::SongRecord;

pub fn now_playing<'a, Message: 'a + Clone>(
    state: &AppState,
    on_toggle_play: Message,
    on_previous: Message,
    on_next: Message,
    on_seek: impl Fn(f32) -> Message + 'a,
    on_play_recommendation: impl Fn(SongRecord) -> Message,
    on_queue_recommendation: impl Fn(SongRecord) -> Message,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    // The whole view has a background tint effect based on state.art_tint
    // We achieve this with a container style.
    
    // Album Art
    let art_size = if state.window_width < 700.0 { 240.0 } else { theme::NOW_PLAYING_ART_MAX };
    
    // In a real implementation we would have the image handle in state.
    // For now we pass None to render the colored rectangle fallback.
    let art = art_canvas(None, art_size, theme::RADIUS_LG, state.art_tint);
    
    // Soft glow ring can be approximated by wrapping art in a container with padding and background
    let art_container = container(art)
        .padding(4)
        .style(iced::theme::Container::Custom(Box::new(GlowStyle(state.art_tint))));

    let mut content = column![].align_items(Alignment::Center).spacing(20);

    if let Some(song) = &state.current_song {
        let title = text(&song.title)
            .font(theme::FONT_DISPLAY)
            .size(theme::TEXT_HERO)
            .style(p.text_primary);

        let artist_album = text(format!("{} · {}", song.artist, song.album))
            .font(theme::FONT_TEXT)
            .size(theme::TEXT_BODY)
            .style(p.text_muted);

        let seek_container = column![
            seek_bar(state.position_secs, state.duration_secs, on_seek),
            row![
                text(format_time(state.position_secs)).size(theme::TEXT_CAPTION).style(p.text_secondary),
                Space::with_width(Length::Fill),
                text(format_time(state.duration_secs)).size(theme::TEXT_CAPTION).style(p.text_secondary),
            ]
        ].spacing(8);

        let is_playing = matches!(state.playback_status, musico_playback::PlaybackStatus::Playing);
        
        let controls = row![
            button(text("🔀").size(20).style(p.text_secondary)).style(iced::theme::Button::Text),
            Space::with_width(20),
            button(text("⏮").size(24).style(p.text_primary)).on_press(on_previous.clone()).style(iced::theme::Button::Text),
            Space::with_width(20),
            play_button(is_playing, state.art_tint, on_toggle_play.clone()),
            Space::with_width(20),
            button(text("⏭").size(24).style(p.text_primary)).on_press(on_next.clone()).style(iced::theme::Button::Text),
            Space::with_width(20),
            button(text("🔁").size(20).style(p.text_secondary)).style(iced::theme::Button::Text),
        ].align_items(Alignment::Center);

        content = content.push(art_container)
            .push(title)
            .push(artist_album)
            .push(seek_container)
            .push(controls);

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
    } else {
        content = content.push(
            text("Nothing is playing")
                .font(theme::FONT_DISPLAY)
                .size(theme::TEXT_HERO)
                .style(p.text_muted)
        );
    }

    let scrollable_content = scrollable(
        container(content)
            .width(Length::Fill)
            .padding(40)
            .center_x()
    );

    container(scrollable_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(NowPlayingBgStyle(state.art_tint, p.base))))
        .into()
}

fn play_button<'a, Message: 'a + Clone>(
    is_playing: bool,
    accent: iced::Color,
    on_press: Message,
) -> Element<'a, Message> {
    let icon = if is_playing { "⏸" } else { "▶" };
    
    button(
        container(text(icon).size(28).style(Color::WHITE))
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
            background: Some(iced::Color { a: 0.15, ..self.0 }.into()),
            border: iced::Border {
                radius: (theme::RADIUS_LG + 4.0).into(),
                ..Default::default()
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
            ..Default::default()
        }
    }
    
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let mut hover_color = self.0;
        // Approximate brightening by 10%
        hover_color.r = (hover_color.r * 1.1).clamp(0.0, 1.0);
        hover_color.g = (hover_color.g * 1.1).clamp(0.0, 1.0);
        hover_color.b = (hover_color.b * 1.1).clamp(0.0, 1.0);
        
        iced::widget::button::Appearance {
            background: Some(hover_color.into()), // Approximate scale on hover
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 39.0.into(),
            },
            ..Default::default()
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()), // Approximate scale down on press
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 34.0.into(),
            },
            ..Default::default()
        }
    }
}

struct NowPlayingBgStyle(iced::Color, iced::Color);
impl iced::widget::container::StyleSheet for NowPlayingBgStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        // We'll simulate a radial gradient or top-down gradient tint by just tinting the top area
        // In Iced 0.12, complex gradients might be tricky without iced_aw, so we use a solid tint
        // layered over the base, or simply an interpolated color.
        
        let mix_factor = 0.05; // 5% tint over base
        let r = self.1.r * (1.0 - mix_factor) + self.0.r * mix_factor;
        let g = self.1.g * (1.0 - mix_factor) + self.0.g * mix_factor;
        let b = self.1.b * (1.0 - mix_factor) + self.0.b * mix_factor;
        
        iced::widget::container::Appearance {
            background: Some(iced::Color::from_rgb(r, g, b).into()),
            ..Default::default()
        }
    }
}
