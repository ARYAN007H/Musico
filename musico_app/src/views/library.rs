use iced::widget::{button, column, container, row, text, text_input, scrollable, Space};
use iced::{Alignment, Element, Length, Color};
use crate::state::{AppState, LibraryViewMode, SortField};
use crate::theme::{self, Palette};
use crate::components::song_row::song_row;
use crate::app::Message;
use musico_recommender::SongRecord;

pub fn library<'a>(
    state: &AppState,
    on_search: impl Fn(String) -> Message + 'a,
    on_clear_search: Message,
    on_toggle_view: Message,
    on_play: impl Fn(SongRecord) -> Message,
    on_queue: impl Fn(SongRecord) -> Message,
) -> Element<'a, Message> {
    let p = Palette::from_color_palette(&state.color_palette);
    let ctx = state.theme_ctx();
    let accent = state.art_tint;

    // Header with search and view toggle
    let search_input = text_input("Search your library...", &state.search_query)
        .on_input(on_search)
        .font(ctx.font_text)
        .size(14)
        .padding(10)
        .style(iced::theme::TextInput::Custom(Box::new(SearchInputStyle(p.surface, p.text_primary, p.text_muted))));

    let mut search_row = row![search_input].align_items(Alignment::Center).width(Length::Fill);

    if !state.search_query.is_empty() {
        search_row = search_row.push(
            button(text("✕").size(14).style(p.text_muted))
                .on_press(on_clear_search)
                .style(iced::theme::Button::Text)
                .padding(10)
        );
    }

    let search_container = container(search_row)
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(SearchContainerStyle(p.surface, ctx.radius_md))));

    let view_icon = match state.library_view_mode {
        LibraryViewMode::Grid => "☰",
        LibraryViewMode::List => "⊞",
    };

    let toggle_view_btn = button(text(view_icon).size(20).style(p.text_secondary))
        .on_press(on_toggle_view)
        .style(iced::theme::Button::Text)
        .padding(10);

    // Song count badge
    let songs = if state.search_query.is_empty() {
        &state.library
    } else {
        &state.filtered_library
    };

    let count_text = if state.search_query.is_empty() {
        format!("{} songs", songs.len())
    } else {
        format!("{} results", songs.len())
    };

    let count_badge = container(
        text(&count_text).font(ctx.font_rounded).size(11.0).style(p.text_muted)
    )
    .padding([4, 10])
    .style(iced::theme::Container::Custom(Box::new(BadgeStyle(accent))));

    let header = row![
        search_container,
        Space::with_width(10),
        count_badge,
        Space::with_width(10),
        toggle_view_btn,
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .padding([20, 40]);

    // ── Sort Controls ──────────────────────────────────────────────────────
    let sort_fields = [
        (SortField::Title, "Title"),
        (SortField::Artist, "Artist"),
        (SortField::Album, "Album"),
        (SortField::Duration, "Duration"),
    ];

    let mut sort_row = row![
        text("Sort by").font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
        Space::with_width(8),
    ].spacing(4).align_items(Alignment::Center);

    for (field, label) in &sort_fields {
        let is_active = state.sort_field == *field;
        let arrow = if is_active {
            if state.sort_ascending { " ↑" } else { " ↓" }
        } else { "" };

        sort_row = sort_row.push(
            button(
                text(format!("{}{}", label, arrow))
                    .font(ctx.font_text)
                    .size(11.0)
                    .style(if is_active { accent } else { p.text_secondary })
            )
            .on_press(Message::SetSortField(*field))
            .padding([4, 10])
            .style(iced::theme::Button::Custom(Box::new(SortPillStyle {
                active: is_active,
                accent,
                bg: p.elevated,
            })))
        );
    }

    let sort_container = container(sort_row)
        .width(Length::Fill)
        .padding([0, 40, 8, 40]);

    // Content
    let content: Element<'a, Message> = if songs.is_empty() {
        container(
            column![
                text("♫").size(48.0).style(theme::with_alpha(accent, 0.3)),
                Space::with_height(12),
                text(if state.search_query.is_empty() { "Library is empty" } else { "No results found" })
                    .font(ctx.font_display)
                    .size(theme::TEXT_TITLE)
                    .style(p.text_muted),
                Space::with_height(4),
                text(if state.search_query.is_empty() { "Add a music folder in Settings" } else { "Try a different search" })
                    .font(ctx.font_text)
                    .size(theme::TEXT_CAPTION)
                    .style(p.text_muted),
            ]
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    } else {
        match state.library_view_mode {
            LibraryViewMode::List => {
                let mut col = column![].spacing(2);
                let current_id = state.current_song.as_ref().map(|s| s.id.clone()).unwrap_or_default();

                for (i, song) in songs.iter().enumerate() {
                    let is_playing = song.id == current_id;
                    col = col.push(song_row(song, i, is_playing, &on_play, Some(&on_queue), accent));
                }

                container(scrollable(col.padding([0, 40, 40, 40]))).height(Length::Fill).into()
            }
            LibraryViewMode::Grid => {
                let columns = if state.window_width < 700.0 { 2 } else { 4 };
                let art_size = if state.window_width < 700.0 { 120.0 } else { 160.0 };
                
                let mut grid_col = column![].spacing(20);
                let mut current_row = row![].spacing(20);
                let mut items_in_row = 0;

                for song in songs {
                    current_row = current_row.push(grid_card(song, art_size, &on_play, accent, &ctx));
                    items_in_row += 1;

                    if items_in_row == columns {
                        grid_col = grid_col.push(current_row);
                        current_row = row![].spacing(20);
                        items_in_row = 0;
                    }
                }

                if items_in_row > 0 {
                    for _ in items_in_row..columns {
                        current_row = current_row.push(Space::new(Length::Fixed(art_size), Length::Fixed(0.0)));
                    }
                    grid_col = grid_col.push(current_row);
                }

                container(scrollable(grid_col.padding([0, 40, 40, 40]))).height(Length::Fill).into()
            }
        }
    };

    column![header, sort_container, content].into()
}

fn grid_card<'a, Message: 'a + Clone>(
    song: &SongRecord,
    art_size: f32,
    on_play: &impl Fn(SongRecord) -> Message,
    accent: Color,
    ctx: &theme::ThemeCtx,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    let art_placeholder = container(
        iced::widget::svg(iced::widget::svg::Handle::from_memory(crate::icons::LIBRARY))
            .width(Length::Fixed(art_size * 0.3))
            .height(Length::Fixed(art_size * 0.3))
            .style(iced::theme::Svg::Custom(Box::new(crate::theme::SvgStyle(theme::with_alpha(accent, 0.4)))))
    )
    .width(Length::Fixed(art_size))
    .height(Length::Fixed(art_size))
    .center_x()
    .center_y()
    .style(iced::theme::Container::Custom(Box::new(GridArtStyle {
        radius: ctx.radius_md,
        bg: theme::with_alpha(p.elevated, 0.5),
    })));

    let title = text(&song.title).font(ctx.font_text).size(theme::TEXT_BODY).style(p.text_primary);
    let artist = text(&song.artist).font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted);

    let song_clone = song.clone();

    button(
        column![
            art_placeholder,
            Space::with_height(8),
            title,
            artist,
        ]
        .width(Length::Fixed(art_size))
    )
    .on_press(on_play(song_clone))
    .style(iced::theme::Button::Custom(Box::new(GridCardStyle {
        bg: Color::TRANSPARENT,
        hover_bg: p.elevated,
        accent,
    })))
    .padding(8)
    .into()
}

// ─── Custom Styles ───────────────────────────────────────────────────────────

#[allow(dead_code)]
struct SearchInputStyle(Color, Color, Color);
impl iced::widget::text_input::StyleSheet for SearchInputStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(Color::TRANSPARENT),
            icon_color: self.2,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
        }
    }
    fn focused(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(style)
    }
    fn placeholder_color(&self, _style: &Self::Style) -> Color { self.2 }
    fn value_color(&self, _style: &Self::Style) -> Color { self.1 }
    fn disabled_color(&self, _style: &Self::Style) -> Color { self.2 }
    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color { a: 0.5, ..self.1 }
    }
    fn disabled(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(style)
    }
    fn hovered(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(style)
    }
}

struct SearchContainerStyle(Color, f32);
impl iced::widget::container::StyleSheet for SearchContainerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: self.1.into(),
            },
            ..Default::default()
        }
    }
}

struct BadgeStyle(Color);
impl iced::widget::container::StyleSheet for BadgeStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(theme::with_alpha(self.0, 0.08).into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
}

struct GridArtStyle { radius: f32, bg: Color }
impl iced::widget::container::StyleSheet for GridArtStyle {
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

struct GridCardStyle { bg: Color, hover_bg: Color, accent: Color }
impl iced::widget::button::StyleSheet for GridCardStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.hover_bg.into()),
            border: iced::Border {
                color: theme::with_alpha(self.accent, 0.2),
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.2, ..theme::BASE },
                offset: iced::Vector { x: 0.0, y: 4.0 },
                blur_radius: 12.0,
            },
            ..Default::default()
        }
    }
}

struct SortPillStyle { active: bool, accent: Color, bg: Color }
impl iced::widget::button::StyleSheet for SortPillStyle {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(if self.active {
                theme::with_alpha(self.accent, 0.12).into()
            } else {
                self.bg.into()
            }),
            border: iced::Border {
                color: if self.active { theme::with_alpha(self.accent, 0.3) } else { Color::TRANSPARENT },
                width: if self.active { 1.0 } else { 0.0 },
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.accent, 0.15).into()),
            border: iced::Border {
                color: theme::with_alpha(self.accent, 0.3),
                width: 1.0,
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
}
