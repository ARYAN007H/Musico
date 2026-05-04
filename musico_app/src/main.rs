mod app;
mod components;
mod config;
mod icons;
mod lyrics;
mod mpris;
mod scanner;
mod state;
mod theme;
mod timer;
mod updater;
mod views;

use iced::{Application, Settings};
use app::Musico;

fn main() -> iced::Result {
    env_logger::init();

    Musico::run(Settings {
        id: Some("musico".to_string()),
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 600.0),
            min_size: Some(iced::Size::new(380.0, 500.0)),
            decorations: true,
            transparent: false,
            ..Default::default()
        },
        default_font: theme::FONT_TEXT,
        ..Default::default()
    })
}
