use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};
use musico_recommender::SongRecord;
use crate::state::AppState;
use crate::theme::{self, Palette};
use crate::components::song_row::song_row;
use crate::components::seek_bar::format_time;

pub fn queue<'a, Message: 'a + Clone>(
    state: &AppState,
    _on_play_queue: impl Fn(SongRecord) -> Message + 'a,
    on_remove_queue: impl Fn(usize) -> Message + 'a,
    on_play_recommendation: impl Fn(SongRecord) -> Message + 'a,
    on_queue_recommendation: impl Fn(SongRecord) -> Message + 'a,
) -> Element<'a, Message> {
    let p = Palette::from_color_palette(&state.color_palette);
    let ctx = state.theme_ctx();
    let accent = state.art_tint;

    let mut content = column![].spacing(20);

    // SECTION 1: QUEUE
    content = content.push(
        text("UPCOMING")
            .font(ctx.font_rounded)
            .size(theme::TEXT_TITLE)
            .style(p.text_primary)
    );

    if state.queue.is_empty() {
        // Rich empty state with icon
        let empty_icon = container(
            iced::widget::svg(iced::widget::svg::Handle::from_memory(crate::icons::QUEUE))
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0))
                .style(iced::theme::Svg::Custom(Box::new(theme::SvgStyle(
                    theme::with_alpha(accent, 0.4),
                ))))
        )
        .width(Length::Fixed(72.0))
        .height(Length::Fixed(72.0))
        .center_x()
        .center_y()
        .style(iced::theme::Container::Custom(Box::new(EmptyQueueIconStyle(accent))));

        content = content.push(
            container(
                column![
                    empty_icon,
                    Space::with_height(12),
                    text("Queue is empty").font(ctx.font_text).size(theme::TEXT_BODY).style(p.text_primary),
                    Space::with_height(4),
                    text("Add songs from Library or let Smart Radio fill it for you")
                        .font(ctx.font_text)
                        .size(theme::TEXT_CAPTION)
                        .style(p.text_muted),
                ]
                .align_items(Alignment::Center)
                .spacing(0)
            )
            .width(Length::Fill)
            .center_x()
            .padding([24, 0])
        );
    } else {
        let mut q_col = column![].spacing(2);
        for (i, song) in state.queue.iter().enumerate() {
            let remove_btn = button(text("✕").size(14).style(p.text_muted))
                .on_press(on_remove_queue(i))
                .style(iced::theme::Button::Text)
                .padding(8);

            let song_row = row![
                container(
                    text(format!("{}", i + 1))
                        .font(ctx.font_rounded)
                        .size(theme::TEXT_CAPTION)
                        .style(p.text_muted)
                ).width(Length::Fixed(28.0)).center_x(),
                column![
                    text(&song.title).font(ctx.font_text).size(theme::TEXT_BODY).style(p.text_primary),
                    text(&song.artist).font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
                ].spacing(2).width(Length::Fill),
                text(format_time(song.duration_secs)).font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
                remove_btn,
            ]
            .align_items(Alignment::Center)
            .spacing(10)
            .padding([8, 12]);

            q_col = q_col.push(
                container(song_row)
                    .width(Length::Fill)
                    .style(iced::theme::Container::Custom(Box::new(QueueRowStyle(
                        if i % 2 == 0 { Color::TRANSPARENT } else {
                            Color {
                                r: p.base.r * 0.7 + p.elevated.r * 0.3,
                                g: p.base.g * 0.7 + p.elevated.g * 0.3,
                                b: p.base.b * 0.7 + p.elevated.b * 0.3,
                                a: 1.0,
                            }
                        }
                    ))))
            );
        }
        content = content.push(q_col);
    }

    // DIVIDER
    content = content.push(Space::with_height(16));
    content = content.push(
        container(Space::with_height(1))
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DividerStyle(p.border_subtle))))
    );
    content = content.push(Space::with_height(16));

    // SECTION 2: RECOMMENDED
    content = content.push(
        text("Suggested for this session")
            .font(ctx.font_rounded)
            .size(theme::TEXT_TITLE)
            .style(p.text_primary)
    );

    if state.recommendations.is_empty() {
        content = content.push(
            container(
                column![
                    text("🎵").size(24.0).style(theme::with_alpha(accent, 0.3)),
                    Space::with_height(6),
                    text("Play some music to get recommendations")
                        .font(ctx.font_text)
                        .size(theme::TEXT_BODY)
                        .style(p.text_muted),
                ]
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .center_x()
            .padding([16, 0])
        );
    } else {
        let mut recs_col = column![].spacing(2);
        
        for (i, rec) in state.recommendations.iter().enumerate() {
            let row_el = song_row(&rec.record, i, false, &on_play_recommendation, Some(&on_queue_recommendation), accent);
            
            // Similarity indicator dot
            let dot_color = if rec.final_score > 0.8 {
                Color::from_rgb8(158, 206, 106) // Green
            } else if rec.final_score > 0.5 {
                Color::from_rgb8(224, 175, 104) // Yellow
            } else {
                p.text_muted
            };

            let dot = container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                .style(iced::theme::Container::Custom(Box::new(DotStyle(dot_color))));

            let item = row![
                container(dot).padding([0, 16, 0, 8]),
                container(row_el).width(Length::Fill)
            ].align_items(Alignment::Center);

            recs_col = recs_col.push(item);
        }
        
        content = content.push(recs_col);
    }

    container(scrollable(content.padding(40)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

struct QueueRowStyle(Color);
impl iced::widget::container::StyleSheet for QueueRowStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

struct DividerStyle(Color);
impl iced::widget::container::StyleSheet for DividerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            ..Default::default()
        }
    }
}

struct DotStyle(Color);
impl iced::widget::container::StyleSheet for DotStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}

struct EmptyQueueIconStyle(Color);
impl iced::widget::container::StyleSheet for EmptyQueueIconStyle {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Background::Color(theme::with_alpha(self.0, 0.08))),
            border: Border {
                radius: 20.0.into(),
                color: theme::with_alpha(self.0, 0.15),
                width: 1.0,
            },
            ..Default::default()
        }
    }
}

struct BrowseBtnStyle(Color);
impl iced::widget::button::StyleSheet for BrowseBtnStyle {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.0, 0.1).into()),
            border: Border {
                radius: 50.0.into(),
                color: theme::with_alpha(self.0, 0.25),
                width: 1.0,
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.0, 0.18).into()),
            border: Border {
                radius: 50.0.into(),
                color: theme::with_alpha(self.0, 0.35),
                width: 1.0,
            },
            ..Default::default()
        }
    }
}
