use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length};
use crate::state::{AppState, View};
use crate::theme::{self, Palette};

// In a real app we'd load an icon font, but we'll use Unicode here for zero-dependency native primitives.
const ICON_HOME: &str = "🏠";
const ICON_LIBRARY: &str = "📚";
const ICON_QUEUE: &str = "≡";
const ICON_SETTINGS: &str = "⚙";

pub fn sidebar<'a, Message: 'a + Clone>(
    state: &AppState,
    navigate_to: impl Fn(View) -> Message + 'a,
    toggle_sidebar: Message,
) -> Element<'a, Message> {
    let p = Palette::default_palette();
    
    let width = if state.sidebar_collapsed {
        theme::SIDEBAR_WIDTH_RAIL
    } else {
        theme::SIDEBAR_WIDTH_FULL
    };

    let logo_text = if state.sidebar_collapsed { "M" } else { "Musico" };

    let logo: Element<'_, Message> = container(
        text(logo_text)
            .font(theme::FONT_DISPLAY)
            .size(24)
            .style(iced::theme::Text::Color(p.text_primary)),
    )
    .width(Length::Fill)
    .padding(if state.sidebar_collapsed { [20, 0, 20, 0] } else { [20, 0, 20, 20] })
    .center_x()
    .into();

    let toggle_btn: Element<'_, Message> = button(
        text(if state.sidebar_collapsed { "→" } else { "←" })
            .size(16)
            .style(iced::theme::Text::Color(p.text_muted))
    )
    .on_press(toggle_sidebar)
    .style(iced::theme::Button::Text)
    .into();

    let logo_row: Element<'_, Message> = if state.sidebar_collapsed {
        column![logo, toggle_btn].align_items(Alignment::Center).into()
    } else {
        row![
            logo,
            Space::with_width(Length::Fill),
            container(toggle_btn).padding([0, 10, 0, 0])
        ]
        .align_items(Alignment::Center)
        .into()
    };

    let nav_items = column![
        nav_item(ICON_HOME, "Home", View::NowPlaying, state, &navigate_to),
        nav_item(ICON_LIBRARY, "Library", View::Library, state, &navigate_to),
        nav_item(ICON_QUEUE, "Queue", View::Queue, state, &navigate_to),
    ]
    .spacing(4);

    let settings_item = nav_item(ICON_SETTINGS, "Settings", View::Settings, state, &navigate_to);

    container(
        column![
            logo_row,
            Space::with_height(20),
            nav_items,
            Space::with_height(Length::Fill),
            settings_item,
        ]
    )
    .width(width)
    .height(Length::Fill)
    .style(iced::theme::Container::Transparent) // Base style can be customised with custom container styles if needed
    .into()
}

fn nav_item<'a, Message: 'a + Clone>(
    icon: &'static str,
    label: &'static str,
    view: View,
    state: &AppState,
    on_press: &impl Fn(View) -> Message,
) -> Element<'a, Message> {
    let p = Palette::default_palette();
    let is_active = state.active_view == view;
    let is_collapsed = state.sidebar_collapsed;

    // We can simulate the active indicator border by using a Row with a colored thin container
    let indicator_color = if is_active { state.art_tint } else { Color::TRANSPARENT };
    
    let indicator: Element<'_, Message> = container(Space::with_width(3))
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(IndicatorStyle(indicator_color))))
        .into();

    let icon_text = text(icon).size(20);
    let label_text = text(label).font(theme::FONT_TEXT).size(theme::TEXT_BODY);

    let content: Element<'_, Message> = if is_collapsed {
        container(icon_text).width(Length::Fill).center_x().into()
    } else {
        container(row![Space::with_width(12), icon_text, Space::with_width(12), label_text].align_items(Alignment::Center))
            .width(Length::Fill)
            .into()
    };

    let bg_color = if is_active { p.highlight } else { Color::TRANSPARENT };
    let text_color = if is_active { p.text_primary } else { p.text_secondary };

    let btn = button(
        row![
            indicator,
            container(content).padding([10, 0])
        ]
        .height(40)
    )
    .on_press(on_press(view))
    .width(Length::Fill)
    .style(iced::theme::Button::Custom(Box::new(NavItemStyle {
        bg: bg_color,
        text: text_color,
        hover_bg: p.elevated,
        hover_text: p.text_primary,
    })));

    // In a real app we'd wrap in tooltip if collapsed, but tooltip is in iced core, not always easy without iced_aw depending on version
    // Iced 0.12 has `iced::widget::tooltip`.
    if is_collapsed {
        iced::widget::tooltip(
            btn,
            label,
            iced::widget::tooltip::Position::Right,
        )
        .style(iced::theme::Container::Box)
        .into()
    } else {
        btn.into()
    }
}

// Custom styles for Iced 0.12

struct IndicatorStyle(iced::Color);
impl iced::widget::container::StyleSheet for IndicatorStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            ..Default::default()
        }
    }
}

struct NavItemStyle {
    bg: iced::Color,
    text: iced::Color,
    hover_bg: iced::Color,
    hover_text: iced::Color,
}

impl iced::widget::button::StyleSheet for NavItemStyle {
    type Style = iced::Theme;
    
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.bg.into()),
            text_color: self.text,
            ..Default::default()
        }
    }
    
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.hover_bg.into()),
            text_color: self.hover_text,
            ..Default::default()
        }
    }
}
