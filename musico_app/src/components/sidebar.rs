// components/sidebar.rs — Musico sidebar navigation
// Responsive: icon-only rail in compact mode, full labels in standard+.

use iced::{
    Alignment, Background, Border, Element, Length,
    widget::{button, column, container, row, svg, text, vertical_space, Space},
};

use crate::{
    app::Message,
    state::{AppState, View},
    theme::{self, *},
};

use crate::icons;

fn icon_now_playing() -> svg::Handle {
    svg::Handle::from_memory(icons::NOW_PLAYING)
}
fn icon_library() -> svg::Handle {
    svg::Handle::from_memory(icons::LIBRARY)
}
fn icon_queue() -> svg::Handle {
    svg::Handle::from_memory(icons::QUEUE)
}
fn icon_settings() -> svg::Handle {
    svg::Handle::from_memory(icons::SETTINGS)
}
fn icon_musico() -> svg::Handle {
    svg::Handle::from_memory(icons::NOW_PLAYING)
}

// ─── Nav item component ───────────────────────────────────────────────────────

fn nav_item<'a>(
    label: &'a str,
    icon: svg::Handle,
    target: View,
    current: View,
    compact: bool,
    accent: iced::Color,
) -> Element<'a, Message> {
    let is_active = current == target;

    let icon_color = if is_active { accent } else { TEXT_SECONDARY };
    let icon_widget = svg(icon)
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(18.0))
        .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(icon_color))));

    let inner: Element<'a, Message> = if compact {
        // Icon-only mode — centered icon, no label
        container(icon_widget)
            .width(Length::Fill)
            .center_x()
            .padding([12, 0])
            .into()
    } else {
        // Full mode — icon + label + accent left bar
        let label_widget = text(label)
            .size(SIZE_LABEL)
            .style(if is_active { accent } else { TEXT_SECONDARY });

        let content = row![icon_widget, label_widget]
            .spacing(10)
            .align_items(Alignment::Center)
            .padding([10, 14])
            .width(Length::Fill);

        if is_active {
            // Active item: accent left bar + tinted bg
            row![
                container(Space::new(Length::Fixed(3.0), Length::Fill))
                    .style(iced::theme::Container::Custom(Box::new(AccentBarStyle(accent)))),
                container(content).width(Length::Fill),
            ].into()
        } else {
            content.into()
        }
    };

    let btn = button(inner)
        .on_press(Message::NavigateTo(target))
        .style(iced::theme::Button::Custom(Box::new(NavButton { is_active, accent })))
        .width(Length::Fill);

    container(btn).padding([0, if compact { 4 } else { 6 }]).width(Length::Fill).into()
}

// ─── Accent left bar style ───────────────────────────────────────────────────

struct AccentBarStyle(iced::Color);
impl iced::widget::container::StyleSheet for AccentBarStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(self.0)),
            border: Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

// ─── Now-playing mini card (bottom of sidebar above settings) ─────────────────

fn now_playing_mini<'a>(state: &'a AppState, compact: bool) -> Element<'a, Message> {
    if let Some(song) = &state.current_song {
        let accent = state.art_tint;
        
        let art = match &song.cover_art {
            Some(bytes) => {
                let b: Vec<u8> = bytes.clone();
                let handle = iced::widget::image::Handle::from_memory(b);
                let img_size = if compact { 28.0 } else { 34.0 };
                container(
                    iced::widget::image(handle)
                        .width(Length::Fixed(img_size))
                        .height(Length::Fixed(img_size)),
                )
                .style(|_theme: &iced::Theme| container::Appearance {
                    border: Border {
                        radius: 7.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            }
            None => {
                let img_size = if compact { 28.0 } else { 34.0 };
                container(
                    svg(icon_now_playing())
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(accent)))),
                )
                .width(Length::Fixed(img_size))
                .height(Length::Fixed(img_size))
                .style(|_theme: &iced::Theme| container::Appearance {
                    background: Some(Background::Color(with_alpha(PALETTE_NEBULA.primary, 0.12))),
                    border: Border {
                        radius: 7.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
            }
        };

        if compact {
            // Compact: just the art thumbnail, centered
            container(art)
                .width(Length::Fill)
                .center_x()
                .padding(6)
                .style(theme::elevated_card)
                .into()
        } else {
            let info = column![
                text(&song.title)
                    .size(SIZE_CAPTION)
                    .style(TEXT_PRIMARY)
                    .shaping(text::Shaping::Advanced),
                text(&song.artist)
                    .size(11.0)
                    .style(TEXT_SECONDARY)
                    .shaping(text::Shaping::Advanced),
            ]
            .spacing(1)
            .width(Length::Fill);

            container(
                row![art, info]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .padding([10, 12]),
            )
            .style(theme::elevated_card)
            .width(Length::Fill)
            .into()
        }
    } else {
        Space::new(Length::Fixed(0.0), Length::Fixed(0.0)).into()
    }
}

// ─── Logo section ─────────────────────────────────────────────────────────────

fn logo_section<'a>(compact: bool, accent: iced::Color) -> Element<'a, Message> {
    let logo_icon = container(
        svg(icon_musico())
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0))
            .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(accent)))),
    )
    .width(Length::Fixed(30.0))
    .height(Length::Fixed(30.0))
    .style(move |_theme: &iced::Theme| container::Appearance {
        background: Some(Background::Color(with_alpha(accent, 0.12))),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    if compact {
        container(logo_icon)
            .width(Length::Fill)
            .center_x()
            .padding([18, 0, 14, 0])
            .into()
    } else {
        container(
            row![
                logo_icon,
                text("Musico")
                    .size(17.0)
                    .style(TEXT_PRIMARY)
                    .font(iced::Font {
                        weight: iced::font::Weight::Semibold,
                        ..iced::Font::DEFAULT
                    }),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        )
        .padding([20, 16, 16, 16])
        .into()
    }
}

// ─── Public: build the full sidebar ──────────────────────────────────────────

pub fn sidebar<'a>(state: &'a AppState) -> Element<'a, Message> {
    let current = state.active_view;
    let compact = state.is_compact();
    let accent = state.art_tint;
    let sw = theme::sidebar_width(state.window_width);

    let nav_section = column![
        nav_item("Now Playing", icon_now_playing(), View::NowPlaying, current, compact, accent),
        nav_item("Library",     icon_library(),     View::Library,    current, compact, accent),
        nav_item("Queue",       icon_queue(),       View::Queue,      current, compact, accent),
    ]
    .spacing(2)
    .padding([4, if compact { 4 } else { 6 }]);

    let bottom_section = column![
        now_playing_mini(state, compact),
        container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .style(|_theme: &iced::Theme| container::Appearance {
                background: Some(Background::Color(BORDER_SUBTLE)),
                ..Default::default()
            })
            .width(Length::Fill)
            .padding([0, if compact { 4 } else { 8 }]),
        nav_item("Settings", icon_settings(), View::Settings, current, compact, accent),
    ]
    .spacing(8)
    .padding([8, 0]);

    container(
        column![
            logo_section(compact, accent),
            nav_section,
            vertical_space(),
            bottom_section,
        ]
        .height(Length::Fill),
    )
    .style(theme::floating_panel)
    .width(Length::Fixed(sw))
    .height(Length::Fill)
    .into()
}
