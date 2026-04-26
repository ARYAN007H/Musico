use iced::widget::{button, column, container, progress_bar, row, text, Space};
use iced::{Alignment, Color, Element, Length};
use crate::state::AppState;
use crate::theme::{self, ColorPalette, FontMode, Palette, ALL_PALETTES, ALL_FONT_MODES};

pub fn settings<'a, Message: 'a + Clone>(
    state: &AppState,
    on_pick_folder: Message,
    on_scan: Message,
    on_set_palette: impl Fn(ColorPalette) -> Message + 'a,
    on_set_font_mode: impl Fn(FontMode) -> Message + 'a,
) -> Element<'a, Message> {
    let p = Palette::from_color_palette(&state.color_palette);
    let ctx = state.theme_ctx();
    let accent = state.art_tint;

    let mut content = column![].spacing(28).padding(40);

    // Header
    content = content.push(
        text("Settings")
            .font(ctx.font_display)
            .size(theme::TEXT_HERO)
            .style(p.text_primary)
    );

    // Music Folder Section
    let folder_path = match &state.music_folder {
        Some(path) => path.to_string_lossy().to_string(),
        None => "Not selected".to_string(),
    };

    let folder_section = container(column![
        text("Music Folder").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(12),
        row![
            container(text(&folder_path).font(ctx.font_text).style(p.text_secondary))
                .padding(12)
                .width(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(InputBgStyle(p.surface)))),
            Space::with_width(12),
            button(text("Change Folder").font(ctx.font_text).style(p.text_primary))
                .on_press(on_pick_folder)
                .padding([12, 20])
                .style(iced::theme::Button::Custom(Box::new(PrimaryBtnStyle(p.elevated, p.highlight))))
        ].align_items(Alignment::Center)
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(folder_section);

    // Re-index Section
    let mut index_content = column![
        text("Library Index").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(12),
    ].spacing(8);

    if state.is_indexing {
        let (done, total) = state.index_progress;
        let progress = if total > 0 { done as f32 / total as f32 } else { 0.0 };
        
        index_content = index_content.push(
            row![
                progress_bar(0.0..=1.0, progress)
                    .height(Length::Fixed(8.0))
                    .style(iced::theme::ProgressBar::Custom(Box::new(ProgressStyle(accent, p.surface)))),
                Space::with_width(12),
                text(format!("{}/{}", done, total)).font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted)
            ].align_items(Alignment::Center)
        );
    } else {
        let lib_count = state.library.len();
        if lib_count > 0 {
            index_content = index_content.push(
                text(format!("{} songs indexed", lib_count))
                    .font(ctx.font_text)
                    .size(theme::TEXT_CAPTION)
                    .style(p.text_muted)
            );
            index_content = index_content.push(Space::with_height(6));
        }
        index_content = index_content.push(
            button(text("Re-index Library").font(ctx.font_text).style(p.text_primary))
                .on_press(on_scan)
                .padding([12, 20])
                .style(iced::theme::Button::Custom(Box::new(PrimaryBtnStyle(p.elevated, p.highlight))))
        );
    }

    let index_section = container(index_content).padding(24).style(theme::glass_card).width(Length::Fill);
    content = content.push(index_section);

    // ─── Color Palette Section ───────────────────────────────────────────────
    let mut palette_row = row![].spacing(12);

    for palette in ALL_PALETTES.iter() {
        let is_selected = state.color_palette.id == palette.id;
        let pal = *palette;

        let gradient_pill = container(
            Space::new(Length::Fixed(60.0), Length::Fixed(28.0))
        )
        .style(iced::theme::Container::Custom(Box::new(PaletteGradientStyle {
            left: palette.primary,
            right: palette.secondary,
        })));

        let card_content = column![
            gradient_pill,
            Space::with_height(6),
            text(palette.name).font(ctx.font_rounded).size(11.0).style(
                if is_selected { accent } else { p.text_secondary }
            )
        ]
        .align_items(Alignment::Center)
        .spacing(2);

        let card = button(
            container(card_content).padding([10, 12]).center_x()
        )
        .on_press(on_set_palette(pal))
        .style(iced::theme::Button::Custom(Box::new(PaletteCardStyle {
            is_selected,
            accent: palette.primary,
            bg: p.elevated,
        })));

        palette_row = palette_row.push(card);
    }

    let palette_section = container(column![
        text("Color Palette").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(14),
        palette_row,
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(palette_section);

    // ─── Font Mode Section ───────────────────────────────────────────────────
    let mut font_row = row![].spacing(10);

    for mode in ALL_FONT_MODES.iter() {
        let is_selected = state.font_mode == *mode;
        let m = *mode;

        let label = button(
            container(
                text(mode.label())
                    .font(mode.display_font())
                    .size(13.0)
                    .style(if is_selected { accent } else { p.text_secondary })
            )
            .padding([10, 18])
            .center_x()
        )
        .on_press(on_set_font_mode(m))
        .style(iced::theme::Button::Custom(Box::new(FontModeStyle {
            is_selected,
            accent,
            bg: p.elevated,
        })));

        font_row = font_row.push(label);
    }

    let font_section = container(column![
        text("Font Mode").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(14),
        font_row,
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(font_section);

    // Keyboard Shortcuts Section
    let shortcuts_section = container(column![
        text("Keyboard Shortcuts").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(12),
        shortcut_row("Space", "Play / Pause", &p, &ctx),
        shortcut_row("← / →", "Seek ±5 seconds", &p, &ctx),
        shortcut_row("↑ / ↓", "Volume ±5%", &p, &ctx),
        shortcut_row("N / P", "Next / Previous", &p, &ctx),
        shortcut_row("S", "Cycle shuffle mode", &p, &ctx),
        shortcut_row("R", "Cycle repeat mode", &p, &ctx),
        shortcut_row("Esc", "Clear search / Now Playing", &p, &ctx),
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(shortcuts_section);

    // About Section
    let about_section = container(column![
        text("About").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(12),
        text("Musico v0.1.0").font(ctx.font_text).style(p.text_secondary),
        text("Powered by Iced 0.12, Symphonia, and pure Rust.").font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(about_section);

    container(
        iced::widget::scrollable(content)
    ).width(Length::Fill).height(Length::Fill).into()
}

fn shortcut_row<'a, Message: 'a>(key: &str, desc: &str, p: &Palette, ctx: &theme::ThemeCtx) -> Element<'a, Message> {
    row![
        container(text(key).font(ctx.font_rounded).size(theme::TEXT_CAPTION).style(p.text_primary))
            .padding([4, 10])
            .style(iced::theme::Container::Custom(Box::new(InputBgStyle(p.surface)))),
        Space::with_width(12),
        text(desc).font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
    ]
    .spacing(4)
    .align_items(Alignment::Center)
    .padding([4, 0])
    .into()
}

// ─── Styles ──────────────────────────────────────────────────────────────────

struct InputBgStyle(Color);
impl iced::widget::container::StyleSheet for InputBgStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: theme::RADIUS_MD.into(),
            },
            ..Default::default()
        }
    }
}

struct PrimaryBtnStyle(Color, Color);
impl iced::widget::button::StyleSheet for PrimaryBtnStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: theme::RADIUS_MD.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.1.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: theme::RADIUS_MD.into(),
            },
            ..Default::default()
        }
    }
}

struct ProgressStyle(Color, Color);
impl iced::widget::progress_bar::StyleSheet for ProgressStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::progress_bar::Appearance {
        iced::widget::progress_bar::Appearance {
            background: self.1.into(),
            bar: self.0.into(),
            border_radius: 4.0.into(),
        }
    }
}

struct PaletteGradientStyle {
    left: Color,
    right: Color,
}
impl iced::widget::container::StyleSheet for PaletteGradientStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        // Approximate gradient with average color (iced doesn't support linear gradients in containers)
        let avg = Color {
            r: (self.left.r + self.right.r) * 0.5,
            g: (self.left.g + self.right.g) * 0.5,
            b: (self.left.b + self.right.b) * 0.5,
            a: 1.0,
        };
        iced::widget::container::Appearance {
            background: Some(avg.into()),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

struct PaletteCardStyle {
    is_selected: bool,
    accent: Color,
    bg: Color,
}
impl iced::widget::button::StyleSheet for PaletteCardStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: if self.is_selected { self.accent } else { iced::Color::TRANSPARENT },
                width: if self.is_selected { 2.0 } else { 0.0 },
                radius: 14.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::HIGHLIGHT.into()),
            border: iced::Border {
                color: if self.is_selected { self.accent } else { theme::with_alpha(self.accent, 0.4) },
                width: if self.is_selected { 2.0 } else { 1.0 },
                radius: 14.0.into(),
            },
            ..Default::default()
        }
    }
}

struct FontModeStyle {
    is_selected: bool,
    accent: Color,
    bg: Color,
}
impl iced::widget::button::StyleSheet for FontModeStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(if self.is_selected {
                theme::with_alpha(self.accent, 0.12).into()
            } else {
                self.bg.into()
            }),
            border: iced::Border {
                color: if self.is_selected { self.accent } else { iced::Color::TRANSPARENT },
                width: if self.is_selected { 1.5 } else { 0.0 },
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.accent, 0.1).into()),
            border: iced::Border {
                color: if self.is_selected { self.accent } else { theme::with_alpha(self.accent, 0.3) },
                width: 1.5,
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
}
