use iced::widget::{button, column, container, row, text, scrollable, Space};
use iced::{Alignment, Color, Element, Length};
use crate::state::AppState;
use crate::theme::{self, Palette};
use crate::app::Message;
use crate::components::song_row::song_row;
use musico_recommender::SmartPlaylist;

pub fn playlists_view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let p = Palette::from_color_palette(&state.color_palette);
    let ctx = state.theme_ctx();
    let accent = state.art_tint;

    let mut content = column![].spacing(20).padding(40);

    // ── Header ─────────────────────────────────────────────────────────────
    content = content.push(
        row![
            text("Playlists")
                .font(ctx.font_display)
                .size(theme::TEXT_HERO)
                .style(p.text_primary),
            Space::with_width(Length::Fill),
            button(
                text("+ New").font(ctx.font_text).size(theme::TEXT_CAPTION).style(accent)
            )
            .on_press(Message::CreatePlaylist)
            .padding([8, 16])
            .style(iced::theme::Button::Custom(Box::new(PillStyle {
                bg: theme::with_alpha(accent, 0.12),
                hover: theme::with_alpha(accent, 0.2),
                border_color: accent,
            }))),
        ].align_items(Alignment::Center)
    );

    // ── Playlist Cards ─────────────────────────────────────────────────────
    if state.playlists.is_empty() {
        content = content.push(
            container(
                column![
                    text("♫").size(48.0).style(theme::with_alpha(accent, 0.3)),
                    Space::with_height(12),
                    text("No playlists yet")
                        .font(ctx.font_display)
                        .size(theme::TEXT_TITLE)
                        .style(p.text_muted),
                    Space::with_height(4),
                    text("Create a smart playlist to get started")
                        .font(ctx.font_text)
                        .size(theme::TEXT_CAPTION)
                        .style(p.text_muted),
                ].align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .center_x()
            .padding(60)
        );
    } else {
        for (i, playlist) in state.playlists.iter().enumerate() {
            let is_selected = state.active_playlist_idx == Some(i);
            let resolved_count = playlist.resolve(&state.library).len();

            let card = container(
                row![
                    // Playlist icon
                    container(
                        text("♬").size(24.0).style(if is_selected { accent } else {
                            theme::with_alpha(p.text_secondary, 0.5)
                        })
                    )
                    .width(Length::Fixed(48.0))
                    .height(Length::Fixed(48.0))
                    .center_x()
                    .center_y()
                    .style(iced::theme::Container::Custom(Box::new(PlaylistIconStyle {
                        accent,
                        active: is_selected,
                    }))),
                    Space::with_width(16),
                    // Title + info
                    column![
                        text(&playlist.name)
                            .font(ctx.font_rounded)
                            .size(theme::TEXT_BODY)
                            .style(if is_selected { accent } else { p.text_primary }),
                        text(format!("{} songs · {} rules", resolved_count, playlist.rules.len()))
                            .font(ctx.font_text)
                            .size(theme::TEXT_CAPTION)
                            .style(p.text_muted),
                    ].spacing(2),
                    Space::with_width(Length::Fill),
                    // Export button
                    button(
                        text("M3U").font(ctx.font_rounded).size(10.0).style(p.text_muted)
                    )
                    .on_press(Message::ExportPlaylist(i))
                    .padding([4, 10])
                    .style(iced::theme::Button::Custom(Box::new(PillStyle {
                        bg: p.elevated,
                        hover: p.highlight,
                        border_color: Color::TRANSPARENT,
                    }))),
                    Space::with_width(8),
                    // Delete button
                    button(
                        text("✕").size(14.0).style(p.text_muted)
                    )
                    .on_press(Message::DeletePlaylist(i))
                    .padding([4, 8])
                    .style(iced::theme::Button::Text),
                ]
                .align_items(Alignment::Center)
                .padding([12, 16])
            )
            .style(if is_selected {
                iced::theme::Container::Custom(Box::new(ActiveCardStyle(accent)))
            } else {
                iced::theme::Container::Custom(Box::new(InactiveCardStyle))
            })
            .width(Length::Fill);

            let pl_clone = playlist.clone();
            content = content.push(
                button(card)
                    .on_press(Message::SelectPlaylist(i))
                    .padding(0)
                    .style(iced::theme::Button::Custom(Box::new(TransparentBtn)))
            );

            // Show resolved songs for the active playlist.
            if is_selected {
                let resolved = pl_clone.resolve(&state.library);
                if resolved.is_empty() {
                    content = content.push(
                        text("  No matching songs").font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted)
                    );
                } else {
                    let current_id = state.current_song.as_ref().map(|s| s.id.clone()).unwrap_or_default();
                    let mut song_list = column![].spacing(2).padding([0, 0, 0, 16]);
                    for (j, song) in resolved.iter().take(50).enumerate() {
                        let is_playing = song.id == current_id;
                        song_list = song_list.push(song_row(
                            song, j, is_playing,
                            &|s: musico_recommender::SongRecord| Message::PlaySong(s),
                            Some(&|s: musico_recommender::SongRecord| Message::AddToQueue(s)),
                            accent,
                        ));
                    }
                    if resolved.len() > 50 {
                        song_list = song_list.push(
                            text(format!("  ... and {} more", resolved.len() - 50))
                                .font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted)
                        );
                    }
                    content = content.push(song_list);
                }
            }
        }
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

// ─── Styles ──────────────────────────────────────────────────────────────────

struct PillStyle {
    bg: Color,
    hover: Color,
    border_color: Color,
}
impl iced::widget::button::StyleSheet for PillStyle {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: self.border_color,
                width: if self.border_color == Color::TRANSPARENT { 0.0 } else { 1.0 },
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.hover.into()),
            border: iced::Border {
                color: self.border_color,
                width: if self.border_color == Color::TRANSPARENT { 0.0 } else { 1.5 },
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
}

struct PlaylistIconStyle { accent: Color, active: bool }
impl iced::widget::container::StyleSheet for PlaylistIconStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(theme::with_alpha(
                self.accent,
                if self.active { 0.15 } else { 0.06 },
            ).into()),
            border: iced::Border { radius: 12.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

struct ActiveCardStyle(Color);
impl iced::widget::container::StyleSheet for ActiveCardStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(theme::with_alpha(self.0, 0.08).into()),
            border: iced::Border {
                color: theme::with_alpha(self.0, 0.3),
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

struct InactiveCardStyle;
impl iced::widget::container::StyleSheet for InactiveCardStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Color { a: 0.06, ..Color::WHITE }.into()),
            border: iced::Border {
                color: Color { a: 0.08, ..Color::WHITE },
                width: 1.0,
                radius: 16.0.into(),
            },
            ..Default::default()
        }
    }
}

struct TransparentBtn;
impl iced::widget::button::StyleSheet for TransparentBtn {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: None,
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: None,
            ..Default::default()
        }
    }
}
