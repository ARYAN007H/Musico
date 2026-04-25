// components/sidebar.rs — Musico sidebar navigation
// Icons drawn as inline SVG paths. Active item has a left-edge accent bar.

use iced::{
    Alignment, Background, Border, Color, Element, Length,
    widget::{button, column, container, row, svg, text, vertical_space, Space},
};
use iced::widget::container::Appearance;

use crate::{
    app::Message,
    state::{AppState, View},
    theme::{self, *},
};

// ─── Icon SVG handles (embed at compile time) ─────────────────────────────────
// Place these SVG files in musico_app/assets/icons/
// Each is a 20x20 single-path icon.

fn icon_now_playing() -> svg::Handle {
    svg::Handle::from_memory(include_bytes!("../../assets/icons/disc.svg").as_slice())
}
fn icon_library() -> svg::Handle {
    svg::Handle::from_memory(include_bytes!("../../assets/icons/library.svg").as_slice())
}
fn icon_queue() -> svg::Handle {
    svg::Handle::from_memory(include_bytes!("../../assets/icons/queue.svg").as_slice())
}
fn icon_settings() -> svg::Handle {
    svg::Handle::from_memory(include_bytes!("../../assets/icons/settings.svg").as_slice())
}
fn icon_musico() -> svg::Handle {
    svg::Handle::from_memory(include_bytes!("../../assets/icons/musico_logo.svg").as_slice())
}

// ─── Nav item component ───────────────────────────────────────────────────────

fn nav_item<'a>(
    label: &'a str,
    icon: svg::Handle,
    target: View,
    current: View,
) -> Element<'a, Message> {
    let is_active = current == target;

    // Left accent bar (3px wide purple line on active)
    let accent_bar = container(Space::new(Length::Fixed(3.0), Length::Fixed(18.0)))
        .style(move |_theme: &iced::Theme| {
            container::Appearance {
                background: if is_active {
                    Some(Background::Color(ACCENT_PURPLE))
                } else {
                    None
                },
                border: Border {
                    radius: [0.0, 3.0, 3.0, 0.0].into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    // Icon tinted based on active state
    let icon_widget = svg(icon)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(
            if is_active { ACCENT_PURPLE } else { TEXT_SECONDARY }
        ))));

    let label_widget = text(label)
        .size(SIZE_LABEL)
        .style(if is_active { TEXT_PRIMARY } else { TEXT_SECONDARY });

    let inner = row![icon_widget, label_widget]
        .spacing(10)
        .align_items(Alignment::Center)
        .padding([9, 12, 9, 12])
        .width(Length::Fill);

    let btn = button(inner)
        .on_press(Message::NavigateTo(target))
        .style(iced::theme::Button::Custom(Box::new(NavButton { is_active })))
        .width(Length::Fill);

    // Wrap in a row so the accent bar sits flush-left outside the button radius
    row![
        accent_bar,
        container(btn).padding([0, 4]).width(Length::Fill),
    ]
    .align_items(Alignment::Center)
    .into()
}

// ─── Now-playing mini card (bottom of sidebar above settings) ─────────────────

fn now_playing_mini<'a>(state: &'a AppState) -> Element<'a, Message> {
    if let Some(song) = &state.current_song {
        let art = match &song.cover_art {
            Some(bytes) => {
                let b: Vec<u8> = bytes.clone();
                let handle = iced::widget::image::Handle::from_memory(b);
                container(
                    iced::widget::image(handle)
                        .width(Length::Fixed(34.0))
                        .height(Length::Fixed(34.0)),
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
                container(
                    svg(icon_now_playing())
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                        .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(ACCENT_PURPLE)))),
                )
                .width(Length::Fixed(34.0))
                .height(Length::Fixed(34.0))
                .style(|_theme: &iced::Theme| container::Appearance {
                    background: Some(Background::Color(with_alpha(ACCENT_PURPLE, 0.12))),
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
    } else {
        Space::new(Length::Fixed(0.0), Length::Fixed(0.0)).into()
    }
}

// ─── Logo section ─────────────────────────────────────────────────────────────

fn logo_section<'a>() -> Element<'a, Message> {
    let logo_icon = container(
        svg(icon_musico())
            .width(Length::Fixed(18.0))
            .height(Length::Fixed(18.0))
            .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(ACCENT_PURPLE)))),
    )
    .width(Length::Fixed(32.0))
    .height(Length::Fixed(32.0))
    .style(|_theme: &iced::Theme| container::Appearance {
        background: Some(Background::Color(ELEVATED)),
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);

    container(
        row![
            logo_icon,
            text("Musico")
                .size(18.0)
                .style(TEXT_PRIMARY)
                .font(iced::Font {
                    weight: iced::font::Weight::Semibold,
                    ..iced::Font::DEFAULT
                }),
        ]
        .spacing(10)
        .align_items(Alignment::Center),
    )
    .padding([22, 20, 18, 20])
    .into()
}

// ─── Public: build the full sidebar ──────────────────────────────────────────

pub fn sidebar<'a>(state: &'a AppState) -> Element<'a, Message> {
    let current = state.active_view;

    let nav_section = column![
        nav_item("Now Playing", icon_now_playing(), View::NowPlaying, current),
        nav_item("Library",     icon_library(),     View::Library,    current),
        nav_item("Queue",       icon_queue(),        View::Queue,      current),
    ]
    .spacing(2)
    .padding([4, 10]);

    let bottom_section = column![
        now_playing_mini(state),
        container(Space::new(Length::Fill, Length::Fixed(1.0)))
            .style(|_theme: &iced::Theme| container::Appearance {
                background: Some(Background::Color(BORDER_SUBTLE)),
                ..Default::default()
            })
            .width(Length::Fill)
            .padding([0, 8]),
        nav_item("Settings", icon_settings(), View::Settings, current),
    ]
    .spacing(10)
    .padding([10, 0]);

    container(
        column![
            logo_section(),
            nav_section,
            vertical_space(),
            bottom_section,
        ]
        .height(Length::Fill),
    )
    .style(theme::sidebar_container)
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .into()
}
