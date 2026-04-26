use iced::mouse;
use iced::widget::canvas::{self, Cache, Canvas, Event, Geometry, Path, Program, Stroke};
use iced::mouse::Cursor;
use iced::{Element, Length, Point, Rectangle, Theme};
use std::time::Duration;
use crate::theme::Palette;

pub struct SeekBar<'a, Message> {
    position_secs: f32,
    duration_secs: f32,
    on_seek: Box<dyn Fn(f32) -> Message + 'a>,
}

impl<'a, Message> SeekBar<'a, Message> {
    pub fn new(
        position_secs: f32,
        duration_secs: f32,
        on_seek: impl Fn(f32) -> Message + 'a,
    ) -> Self {
        Self {
            position_secs,
            duration_secs,
            on_seek: Box::new(on_seek),
        }
    }
}

pub fn format_time(secs: f32) -> String {
    let duration = Duration::from_secs_f32(secs);
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[derive(Default)]
pub struct SeekBarState {
    cache: Cache,
    is_dragging: bool,
    hover_x: Option<f32>,
}

impl<'a, Message> Program<Message> for SeekBar<'a, Message> {
    type State = SeekBarState;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let cursor_position = cursor.position_in(bounds);
        
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    if let Some(pos) = cursor_position {
                        state.hover_x = Some(pos.x);
                        if state.is_dragging {
                            state.cache.clear();
                            // Update drag visual
                        }
                    } else {
                        if !state.is_dragging {
                            state.hover_x = None;
                        }
                    }
                    state.cache.clear();
                    (canvas::event::Status::Ignored, None)
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if cursor_position.is_some() {
                        state.is_dragging = true;
                        state.cache.clear();
                        (canvas::event::Status::Captured, None)
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.is_dragging {
                        state.is_dragging = false;
                        state.cache.clear();
                        if let Some(x) = state.hover_x {
                            let percent = (x / bounds.width).clamp(0.0, 1.0);
                            let new_pos = percent * self.duration_secs;
                            (canvas::event::Status::Captured, Some((self.on_seek)(new_pos)))
                        } else {
                            (canvas::event::Status::Ignored, None)
                        }
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                }
                _ => (canvas::event::Status::Ignored, None),
            },
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let p = Palette::default_palette();
            
            let track_height = 3.0;
            let y_center = bounds.height / 2.0;
            
            // Draw background track
            let track_bg = Path::line(
                Point::new(0.0, y_center),
                Point::new(bounds.width, y_center),
            );
            let track_bg_color = iced::Color::from_rgb8(0x1e, 0x20, 0x33);
            frame.stroke(
                &track_bg,
                Stroke::default()
                    .with_color(track_bg_color)
                    .with_width(track_height),
            );

            // Calculate fill percentage based on drag or actual position
            let percent = if state.is_dragging {
                if let Some(x) = state.hover_x {
                    (x / bounds.width).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            } else {
                if self.duration_secs > 0.0 {
                    (self.position_secs / self.duration_secs).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            };
            
            let fill_width = bounds.width * percent;

            // Draw filled track
            let track_fg = Path::line(
                Point::new(0.0, y_center),
                Point::new(fill_width, y_center),
            );
            frame.stroke(
                &track_fg,
                Stroke::default()
                    .with_color(p.accent)
                    .with_width(track_height),
            );

            // Always draw thumb but make it more prominent when hovering
            let is_active = state.hover_x.is_some() || state.is_dragging;
            let thumb_radius = if is_active { 5.5 } else { 4.0 };
            
            let thumb_path = Path::circle(Point::new(fill_width, y_center), thumb_radius);
            frame.fill(&thumb_path, iced::Color::WHITE);
            frame.stroke(&thumb_path, Stroke::default().with_color(p.accent).with_width(2.0));
        });

        vec![geometry]
    }
}

// We wrap the Canvas in an Element to make it easy to use
pub fn seek_bar<'a, Message: 'a>(
    position_secs: f32,
    duration_secs: f32,
    on_seek: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    Canvas::new(SeekBar::new(position_secs, duration_secs, on_seek))
        .width(Length::Fill)
        .height(Length::Fixed(20.0))
        .into()
}
