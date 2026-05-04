use iced::widget::{button, column, container, row, text, svg, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};
use musico_recommender::SongRecord;
use crate::theme::{self, Palette};
use crate::components::seek_bar::format_time;
use crate::icons;

pub fn song_row<'a, Message: 'a + Clone>(
    song: &SongRecord,
    index: usize,
    is_playing: bool,
    on_play: impl Fn(SongRecord) -> Message,
    on_queue: Option<impl Fn(SongRecord) -> Message>,
    accent: Color,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    // ── Track number / playing indicator ──────────────────────────────────
    let indicator: Element<'a, Message> = if is_playing {
        // Animated equalizer bars representation (static, but visually distinct)
        container(
            column![
                container(Space::new(Length::Fixed(3.0), Length::Fixed(12.0)))
                    .style(iced::theme::Container::Custom(Box::new(EqBarStyle(accent)))),
                container(Space::new(Length::Fixed(3.0), Length::Fixed(8.0)))
                    .style(iced::theme::Container::Custom(Box::new(EqBarStyle(accent)))),
                container(Space::new(Length::Fixed(3.0), Length::Fixed(14.0)))
                    .style(iced::theme::Container::Custom(Box::new(EqBarStyle(accent)))),
            ]
            .spacing(2)
            .align_items(Alignment::End),
        )
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0))
        .center_x()
        .center_y()
        .into()
    } else {
        text(format!("{}", index + 1))
            .size(12.0)
            .style(theme::TEXT_MUTED)
            .width(Length::Fixed(18.0))
            .into()
    };

    // ── Thumbnail placeholder ────────────────────────────────────────────
    let thumb_bg = if is_playing {
        theme::with_alpha(accent, 0.15)
    } else {
        p.surface
    };

    let thumb_icon_color = if is_playing {
        accent
    } else {
        theme::with_alpha(theme::TEXT_MUTED, 0.4)
    };

    let thumb = container(
        svg(svg::Handle::from_memory(icons::LIBRARY))
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(thumb_icon_color)))),
    )
    .width(Length::Fixed(44.0))
    .height(Length::Fixed(44.0))
    .center_x()
    .center_y()
    .style(iced::theme::Container::Custom(Box::new(ThumbStyle {
        radius: 8.0,
        bg: thumb_bg,
        accent,
        is_playing,
    })));

    // ── Song info ────────────────────────────────────────────────────────
    let title_color = if is_playing { accent } else { p.text_primary };

    let middle = column![
        text(&song.title)
            .font(theme::FONT_TEXT)
            .size(theme::TEXT_BODY)
            .style(title_color)
            .shaping(text::Shaping::Advanced),
        text(format!("{} · {}", song.artist, song.album))
            .font(theme::FONT_TEXT)
            .size(theme::TEXT_CAPTION)
            .style(p.text_muted)
            .shaping(text::Shaping::Advanced),
    ]
    .spacing(3)
    .width(Length::Fill);

    // ── Duration ─────────────────────────────────────────────────────────
    let duration_text = text(format_time(song.duration_secs as f32))
        .font(theme::FONT_TEXT)
        .size(theme::TEXT_CAPTION)
        .style(if is_playing { theme::with_alpha(accent, 0.7) } else { p.text_muted });

    // ── Action buttons ───────────────────────────────────────────────────
    let mut actions = row![duration_text].align_items(Alignment::Center).spacing(6);

    if let Some(queue_fn) = on_queue {
        // Queue button with + icon
        let queue_btn = button(
            svg(svg::Handle::from_memory(icons::QUEUE))
                .width(Length::Fixed(14.0))
                .height(Length::Fixed(14.0))
                .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(p.text_secondary))))
        )
        .on_press(queue_fn(song.clone()))
        .style(iced::theme::Button::Custom(Box::new(ActionBtnStyle(accent))))
        .padding(6);
        actions = actions.push(queue_btn);
    }

    // ── Row background ───────────────────────────────────────────────────
    let bg_color = if is_playing {
        theme::with_alpha(accent, 0.06)
    } else if index % 2 == 0 {
        Color::TRANSPARENT
    } else {
        theme::with_alpha(p.elevated, 0.3)
    };

    let song_clone = song.clone();

    // ── Active left border for playing track ─────────────────────────────
    let row_content = row![
        indicator,
        Space::with_width(8),
        thumb,
        Space::with_width(12),
        middle,
        actions,
    ]
    .align_items(Alignment::Center)
    .padding([6, 12]);

    let inner: Element<'a, Message> = if is_playing {
        row![
            container(Space::new(Length::Fixed(3.0), Length::Fixed(44.0)))
                .style(iced::theme::Container::Custom(Box::new(AccentLeftBar(accent)))),
            container(row_content).width(Length::Fill),
        ]
        .align_items(Alignment::Center)
        .into()
    } else {
        row_content.into()
    };

    button(inner)
        .on_press(on_play(song_clone))
        .width(Length::Fill)
        .style(iced::theme::Button::Custom(Box::new(RowStyle {
            bg: bg_color,
            hover_bg: p.highlight,
            accent,
            is_playing,
        })))
        .into()
}

// ─── Styles ──────────────────────────────────────────────────────────────────

struct EqBarStyle(Color);
impl iced::widget::container::StyleSheet for EqBarStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: Border { radius: 1.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

struct AccentLeftBar(Color);
impl iced::widget::container::StyleSheet for AccentLeftBar {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: Border { radius: 2.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

struct ThumbStyle {
    radius: f32,
    bg: Color,
    accent: Color,
    is_playing: bool,
}

impl iced::widget::container::StyleSheet for ThumbStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.bg.into()),
            border: Border {
                color: if self.is_playing {
                    theme::with_alpha(self.accent, 0.3)
                } else {
                    Color::TRANSPARENT
                },
                width: if self.is_playing { 1.0 } else { 0.0 },
                radius: self.radius.into(),
            },
            shadow: if self.is_playing {
                iced::Shadow {
                    color: Color { a: 0.2, ..self.accent },
                    offset: iced::Vector { x: 0.0, y: 2.0 },
                    blur_radius: 8.0,
                }
            } else {
                iced::Shadow::default()
            },
            ..Default::default()
        }
    }
}

struct ActionBtnStyle(Color);
impl iced::widget::button::StyleSheet for ActionBtnStyle {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: None,
            border: Border { radius: 6.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.0, 0.1).into()),
            border: Border {
                radius: 6.0.into(),
                color: theme::with_alpha(self.0, 0.2),
                width: 1.0,
            },
            ..Default::default()
        }
    }
}

struct RowStyle {
    bg: Color,
    hover_bg: Color,
    accent: Color,
    is_playing: bool,
}

impl iced::widget::button::StyleSheet for RowStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.bg.into()),
            border: Border {
                color: if self.is_playing {
                    theme::with_alpha(self.accent, 0.15)
                } else {
                    Color::TRANSPARENT
                },
                width: if self.is_playing { 1.0 } else { 0.0 },
                radius: 10.0.into(),
            },
            shadow: if self.is_playing {
                iced::Shadow {
                    color: Color { a: 0.15, ..self.accent },
                    offset: iced::Vector { x: 0.0, y: 2.0 },
                    blur_radius: 12.0,
                }
            } else {
                iced::Shadow::default()
            },
            ..Default::default()
        }
    }
    
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(if self.is_playing {
                theme::with_alpha(self.accent, 0.1).into()
            } else {
                self.hover_bg.into()
            }),
            border: Border {
                color: theme::with_alpha(self.accent, 0.3),
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: iced::Shadow {
                color: Color { a: 0.1, ..theme::BASE },
                offset: iced::Vector { x: 0.0, y: 2.0 },
                blur_radius: 8.0,
            },
            ..Default::default()
        }
    }
}
