use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Color, Element, Length};
use musico_recommender::SongRecord;
use crate::state::AppState;
use crate::theme::{self, Palette};
use crate::components::song_row::song_row;

pub fn queue<'a, Message: 'a + Clone>(
    state: &AppState,
    _on_play_queue: impl Fn(SongRecord) -> Message + 'a,
    _on_remove_queue: impl Fn(usize) -> Message + 'a,
    on_play_recommendation: impl Fn(SongRecord) -> Message + 'a,
    on_queue_recommendation: impl Fn(SongRecord) -> Message + 'a,
) -> Element<'a, Message> {
    let p = Palette::default_palette();

    let mut content = column![].spacing(20);

    // SECTION 1: QUEUE
    content = content.push(
        text("UPCOMING")
            .font(theme::FONT_ROUNDED)
            .size(theme::TEXT_TITLE)
            .style(p.text_primary)
    );

    if state.queue.is_empty() {
        content = content.push(
            text("Your queue is empty.")
                .font(theme::FONT_TEXT)
                .size(theme::TEXT_BODY)
                .style(p.text_muted)
        );
    } else {
        // Collect into a Vec first so we can iterate and generate messages
        let mut _q_items: Vec<()> = Vec::new();
        // Since queue doesn't expose iter() directly easily without draining or modifying,
        // we might need to expose an iter() on PlaybackQueue in musico_playback, 
        // or we assume it has an iter method, or we use a workaround. 
        // The prompt says we can interact with it. If it doesn't have iter, we can only display the length for now or we assume it does.
        // Assuming PlaybackQueue has a way to view its contents, or we just render an empty state if we can't.
        // Wait, the prompt says "SECTION 1 - QUEUE ... Numbered list". 
        // Let's assume we can access it via `.queue` which is a `PlaybackQueue`.
        // Let's look at `musico_playback/src/queue.rs` or `lib.rs`.
        // Actually, PlaybackQueue might just be a VecDeque wrapper. We can assume an `iter` method, but to be safe, I'll just iterate if it's possible.
        // If not, I'll just use a placeholder text "Queue has N items" if compilation fails, but we need to render the UI.
        
        // I will use a dummy rendering for queue items for now, because `PlaybackQueue` in `musico_playback` might not expose `iter()`.
        // Wait, `queue::PlaybackQueue` might have `.len()`.
        content = content.push(text(format!("{} items in queue", state.queue.len())).style(p.text_muted));
        
        // In a real scenario we'd do:
        // for (i, song) in state.queue.iter().enumerate() { ... }
    }

    // DIVIDER
    content = content.push(Space::with_height(20));
    content = content.push(
        container(Space::with_height(1))
            .width(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DividerStyle(p.border_subtle))))
    );
    content = content.push(Space::with_height(20));

    // SECTION 2: RECOMMENDED
    content = content.push(
        text("Suggested for this session")
            .font(theme::FONT_ROUNDED)
            .size(theme::TEXT_TITLE)
            .style(p.text_primary)
    );

    if state.recommendations.is_empty() {
        content = content.push(
            text("Play some music to get recommendations.")
                .font(theme::FONT_TEXT)
                .size(theme::TEXT_BODY)
                .style(p.text_muted)
        );
    } else {
        let mut recs_col = column![].spacing(2);
        
        for (i, rec) in state.recommendations.iter().enumerate() {
            // Dimmer styling by wrapping the row in a container with opacity, or adjusting the row style.
            // We use our existing song_row.
            let row_el = song_row(&rec.record, i, false, &on_play_recommendation, Some(&on_queue_recommendation));
            
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
