use iced::widget::{button, column, container, row, text, Space, scrollable};
use iced::{Alignment, Color, Element, Length};
use crate::state::AppState;
use crate::theme::{self, Palette};
use crate::app::Message;

pub fn stats_view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let p = Palette::from_color_palette(&state.color_palette);
    let ctx = state.theme_ctx();
    let accent = state.art_tint;

    let mut content = column![].spacing(24).padding(40);

    // Header
    content = content.push(
        row![
            text("Listening Stats")
                .font(ctx.font_display)
                .size(theme::TEXT_HERO)
                .style(p.text_primary),
            Space::with_width(Length::Fill),
            button(
                text("Refresh").font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_primary)
            )
            .on_press(Message::RefreshStats)
            .padding([8, 16])
            .style(iced::theme::Button::Custom(Box::new(PillBtnStyle(p.elevated, p.highlight)))),
        ].align_items(Alignment::Center)
    );

    if state.stats_loading {
        content = content.push(
            text("Loading statistics...").font(ctx.font_text).style(p.text_muted)
        );
    } else if let Some(stats) = &state.stats {
        // ─── Key Metrics Row ──────────────────────────────────────────────────

        let total_hours = stats.total_listened_secs as f32 / 3600.0;
        let completion_rate = if stats.total_plays > 0 {
            ((stats.total_plays - stats.total_skips) as f32 / stats.total_plays as f32) * 100.0
        } else {
            0.0
        };

        let plays_str = format!("{}", stats.total_plays);
        let hours_str = format!("{:.1}", total_hours);
        let rate_str = format!("{:.0}%", completion_rate);
        let streak_str = format!("{} days", stats.streak_days);

        let metrics = row![
            stat_card("Total Plays".to_string(), plays_str, accent, &p, &ctx),
            stat_card("Hours Listened".to_string(), hours_str, accent, &p, &ctx),
            stat_card("Completion Rate".to_string(), rate_str, accent, &p, &ctx),
            stat_card("Streak".to_string(), streak_str, accent, &p, &ctx),
        ].spacing(12);

        content = content.push(metrics);

        // ─── Top Songs ────────────────────────────────────────────────────────

        if !stats.top_songs.is_empty() {
            let mut top_section = column![
                text("Top Tracks").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
                Space::with_height(8),
            ].spacing(4);

            for (i, (song_id, count)) in stats.top_songs.iter().take(10).enumerate() {
                // Try to find the title from the library.
                let title = state.library.iter()
                    .find(|s| s.id == *song_id)
                    .map(|s| format!("{} — {}", s.title, s.artist))
                    .unwrap_or_else(|| song_id[..8.min(song_id.len())].to_string());

                let rank_color = match i {
                    0 => Color::from_rgb8(255, 215, 0), // gold
                    1 => Color::from_rgb8(192, 192, 192), // silver
                    2 => Color::from_rgb8(205, 127, 50), // bronze
                    _ => p.text_secondary,
                };

                top_section = top_section.push(
                    container(
                        row![
                            text(format!("#{}", i + 1))
                                .font(ctx.font_rounded)
                                .size(theme::TEXT_CAPTION)
                                .style(rank_color)
                                .width(Length::Fixed(32.0)),
                            text(&title)
                                .font(ctx.font_text)
                                .size(theme::TEXT_BODY)
                                .style(p.text_primary)
                                .width(Length::Fill),
                            text(format!("{} plays", count))
                                .font(ctx.font_text)
                                .size(theme::TEXT_CAPTION)
                                .style(p.text_muted),
                        ].align_items(Alignment::Center).padding([8, 12])
                    )
                    .style(if i % 2 == 0 { theme::glass_card } else { theme::glass_card })
                    .width(Length::Fill)
                );
            }

            content = content.push(
                container(top_section).padding(20).style(theme::glass_card).width(Length::Fill)
            );
        }

        // ─── Daily Activity Heatmap ───────────────────────────────────────────

        if !stats.daily_minutes.is_empty() {
            let max_mins = stats.daily_minutes.iter().map(|(_, m)| *m).max().unwrap_or(1);

            let mut heatmap_cols = row![].spacing(3);
            let days_to_show = stats.daily_minutes.len().min(52 * 7); // max ~1 year
            let start = if stats.daily_minutes.len() > days_to_show {
                stats.daily_minutes.len() - days_to_show
            } else {
                0
            };

            for (_date, mins) in &stats.daily_minutes[start..] {
                let intensity = (*mins as f32 / max_mins as f32).clamp(0.0, 1.0);
                let color = Color {
                    r: accent.r,
                    g: accent.g,
                    b: accent.b,
                    a: 0.1 + intensity * 0.9,
                };
                heatmap_cols = heatmap_cols.push(
                    container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0)))
                        .style(iced::theme::Container::Custom(Box::new(HeatCell(color))))
                );
            }

            content = content.push(
                container(column![
                    text("Daily Listening").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
                    Space::with_height(12),
                    scrollable(heatmap_cols).direction(iced::widget::scrollable::Direction::Horizontal(
                        iced::widget::scrollable::Properties::default()
                    )),
                ]).padding(20).style(theme::glass_card).width(Length::Fill)
            );
        }
    } else {
        content = content.push(
            text("No listening data yet. Play some music!")
                .font(ctx.font_text)
                .style(p.text_muted)
        );
    }

    container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
}

fn stat_card<'a>(
    label: String,
    value: String,
    accent: Color,
    p: &Palette,
    ctx: &theme::ThemeCtx,
) -> Element<'a, Message> {
    container(
        column![
            text(value)
                .font(ctx.font_display)
                .size(theme::TEXT_HERO)
                .style(accent),
            Space::with_height(4),
            text(label)
                .font(ctx.font_text)
                .size(theme::TEXT_CAPTION)
                .style(p.text_muted),
        ].align_items(Alignment::Center)
    )
    .padding([20, 16])
    .style(theme::glass_card)
    .width(Length::Fill)
    .center_x()
    .into()
}

struct PillBtnStyle(Color, Color);
impl iced::widget::button::StyleSheet for PillBtnStyle {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()),
            border: iced::Border { radius: 50.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.1.into()),
            border: iced::Border { radius: 50.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}

struct HeatCell(Color);
impl iced::widget::container::StyleSheet for HeatCell {
    type Style = iced::Theme;
    fn appearance(&self, _: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: iced::Border { radius: 2.0.into(), ..Default::default() },
            ..Default::default()
        }
    }
}
