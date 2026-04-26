use iced::widget::{button, column, container, row, text, text_input, scrollable, Space};
use iced::{Alignment, Element, Length, Color};
use crate::state::{AppState, LibraryViewMode};
use crate::theme::{self, Palette};
use crate::components::song_row::song_row;
use musico_recommender::SongRecord;

pub fn library<'a, Message: 'a + Clone>(
    state: &AppState,
    on_search: impl Fn(String) -> Message + 'a,
    on_clear_search: Message,
    on_toggle_view: Message,
    on_play: impl Fn(SongRecord) -> Message,
    on_queue: impl Fn(SongRecord) -> Message,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    // Header with search and view toggle
    let search_input = text_input("Search your library...", &state.search_query)
        .on_input(on_search)
        .font(theme::FONT_TEXT)
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
        .style(iced::theme::Container::Custom(Box::new(SearchContainerStyle(p.surface, theme::RADIUS_MD))));

    let view_icon = match state.library_view_mode {
        LibraryViewMode::Grid => "☰",
        LibraryViewMode::List => "⊞",
    };

    let toggle_view_btn = button(text(view_icon).size(20).style(p.text_secondary))
        .on_press(on_toggle_view)
        .style(iced::theme::Button::Text)
        .padding(10);

    let header = row![
        search_container,
        Space::with_width(16),
        toggle_view_btn,
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .padding([20, 40]);

    // Content
    let songs = if state.search_query.is_empty() {
        &state.library
    } else {
        &state.filtered_library
    };

    let content: Element<'a, Message> = if songs.is_empty() {
        container(
            text(if state.search_query.is_empty() { "Library is empty." } else { "No results found." })
                .font(theme::FONT_TEXT)
                .size(theme::TEXT_TITLE)
                .style(p.text_muted)
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
                    col = col.push(song_row(song, i, is_playing, &on_play, Some(&on_queue)));
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
                    current_row = current_row.push(grid_card(song, art_size, &on_play));
                    items_in_row += 1;

                    if items_in_row == columns {
                        grid_col = grid_col.push(current_row);
                        current_row = row![].spacing(20);
                        items_in_row = 0;
                    }
                }

                if items_in_row > 0 {
                    // Fill remaining space to keep alignment
                    for _ in items_in_row..columns {
                        current_row = current_row.push(Space::new(Length::Fixed(art_size), Length::Fixed(0.0)));
                    }
                    grid_col = grid_col.push(current_row);
                }

                container(scrollable(grid_col.padding([0, 40, 40, 40]))).height(Length::Fill).into()
            }
        }
    };

    column![header, content].into()
}

fn grid_card<'a, Message: 'a + Clone>(
    song: &SongRecord,
    art_size: f32,
    on_play: &impl Fn(SongRecord) -> Message,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    let art_placeholder = container(Space::new(Length::Fixed(art_size), Length::Fixed(art_size)))
        .width(Length::Fixed(art_size))
        .height(Length::Fixed(art_size))
        .style(iced::theme::Container::Custom(Box::new(GridArtStyle {
            radius: 10.0,
            bg: p.surface,
        })));

    let title = text(&song.title).font(theme::FONT_TEXT).size(theme::TEXT_BODY).style(p.text_primary);
    let artist = text(&song.artist).font(theme::FONT_TEXT).size(theme::TEXT_CAPTION).style(p.text_muted);

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
    })))
    .padding(8)
    .into()
}

// Custom Styles

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

struct GridCardStyle { bg: Color, hover_bg: Color }
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
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        }
    }
}
