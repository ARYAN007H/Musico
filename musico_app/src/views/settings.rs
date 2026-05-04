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
    on_check_update: Message,
    on_download_update: impl Fn(String) -> Message + 'a,
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

    // Re-index Section — Enhanced with animated progress
    let mut index_content = column![
        text("Library Index").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(12),
    ].spacing(8);

    if state.is_indexing {
        let (done, total) = state.index_progress;
        let progress = if total > 0 { done as f32 / total as f32 } else { 0.0 };
        let percent = (progress * 100.0) as u32;
        
        // Spinning indicator character based on progress
        let spinner_chars = ["◐", "◓", "◑", "◒"];
        let spinner = spinner_chars[done % spinner_chars.len()];

        // Status header with spinner and percentage
        index_content = index_content.push(
            row![
                text(spinner).size(20.0).style(accent),
                Space::with_width(10),
                text(format!("Indexing... {}%", percent))
                    .font(ctx.font_rounded)
                    .size(theme::TEXT_BODY)
                    .style(accent),
                Space::with_width(Length::Fill),
                text(format!("{} / {} songs", done, total))
                    .font(ctx.font_text)
                    .size(theme::TEXT_CAPTION)
                    .style(p.text_secondary),
            ].align_items(Alignment::Center)
        );

        index_content = index_content.push(Space::with_height(8));

        // Progress bar
        index_content = index_content.push(
            progress_bar(0.0..=1.0, progress)
                .height(Length::Fixed(6.0))
                .style(iced::theme::ProgressBar::Custom(Box::new(ProgressStyle(accent, p.surface))))
        );

        // Hint text
        index_content = index_content.push(
            text("Analyzing audio features for smart recommendations...")
                .font(ctx.font_text)
                .size(11.0)
                .style(p.text_muted)
        );
    } else {
        let lib_count = state.library.len();
        if lib_count > 0 {
            // Show song count with accent badge
            index_content = index_content.push(
                row![
                    container(
                        text(format!("{}", lib_count))
                            .font(ctx.font_rounded)
                            .size(theme::TEXT_BODY)
                            .style(accent)
                    )
                    .padding([4, 12])
                    .style(iced::theme::Container::Custom(Box::new(CountBadgeStyle(accent)))),
                    Space::with_width(10),
                    text("songs indexed")
                        .font(ctx.font_text)
                        .size(theme::TEXT_BODY)
                        .style(p.text_secondary),
                ].align_items(Alignment::Center)
            );
            index_content = index_content.push(Space::with_height(8));
        }
        index_content = index_content.push(
            button(
                row![
                    text("⟳").size(16.0).style(p.text_primary),
                    Space::with_width(8),
                    text("Re-index Library").font(ctx.font_text).style(p.text_primary),
                ].align_items(Alignment::Center)
            )
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

    // ─── Auto-Update Section ─────────────────────────────────────────────────
    let update_content = {
        use crate::state::UpdateStatus;
        let mut col = column![
            text("Updates").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
            Space::with_height(12),
        ].spacing(8);

        match &state.update_status {
            UpdateStatus::Idle => {
                col = col.push(
                    button(
                        text("Check for Updates").font(ctx.font_text).style(p.text_primary)
                    )
                    .on_press(on_check_update)
                    .padding([12, 20])
                    .style(iced::theme::Button::Custom(Box::new(PrimaryBtnStyle(p.elevated, p.highlight))))
                );
            }
            UpdateStatus::Checking => {
                col = col.push(
                    text("Checking for updates...").font(ctx.font_text).size(theme::TEXT_BODY).style(p.text_muted)
                );
            }
            UpdateStatus::Available { version, url } => {
                col = col.push(
                    text(format!("Update available: v{}", version))
                        .font(ctx.font_text)
                        .size(theme::TEXT_BODY)
                        .style(accent)
                );
                col = col.push(Space::with_height(8));
                col = col.push(
                    button(
                        text("Download & Install").font(ctx.font_text).style(p.text_primary)
                    )
                    .on_press(on_download_update(url.clone()))
                    .padding([12, 20])
                    .style(iced::theme::Button::Custom(Box::new(AccentBtnStyle(accent))))
                );
            }
            UpdateStatus::Downloading => {
                col = col.push(
                    text("Downloading update...").font(ctx.font_text).size(theme::TEXT_BODY).style(accent)
                );
                col = col.push(
                    progress_bar(0.0..=1.0, 0.5)
                        .height(Length::Fixed(6.0))
                        .style(iced::theme::ProgressBar::Custom(Box::new(ProgressStyle(accent, p.surface))))
                );
            }
            UpdateStatus::Ready => {
                col = col.push(
                    text("✓ Update installed! Restart Musico to apply.")
                        .font(ctx.font_text)
                        .size(theme::TEXT_BODY)
                        .style(Color::from_rgb8(158, 206, 106)) // green
                );
            }
            UpdateStatus::Error(msg) => {
                col = col.push(
                    text(msg)
                        .font(ctx.font_text)
                        .size(theme::TEXT_CAPTION)
                        .style(if msg.contains('✓') { Color::from_rgb8(158, 206, 106) } else { Color::from_rgb8(224, 108, 117) })
                );
            }
        }
        col
    };

    let update_section = container(update_content).padding(24).style(theme::glass_card).width(Length::Fill);
    content = content.push(update_section);

    // About Section
    let version = env!("CARGO_PKG_VERSION");
    let about_section = container(column![
        text("About").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(12),
        text(format!("Musico v{}", version)).font(ctx.font_text).style(p.text_secondary),
        text("Powered by Iced 0.12, Symphonia, and pure Rust.").font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(about_section);

    container(
        iced::widget::scrollable(content)
    ).width(Length::Fill).height(Length::Fill).into()
}

/// Builds the EQ + Audio settings section with concrete Message type.
/// Call this separately and push into a column alongside the generic settings.
pub fn audio_settings<'a>(state: &'a AppState) -> iced::Element<'a, crate::app::Message> {
    use crate::app::Message;
    use crate::state::NormalizationMode;
    use musico_playback::eq::{ALL_PRESETS, BAND_LABELS};

    let p = Palette::from_color_palette(&state.color_palette);
    let ctx = state.theme_ctx();
    let accent = state.art_tint;

    let mut content = column![].spacing(20);

    // ─── Equalizer Section ────────────────────────────────────────────────────
    let eq_status = if state.eq_enabled { "On" } else { "Off" };
    let mut eq_section = column![
        row![
            text("Equalizer").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
            Space::with_width(Length::Fill),
            button(
                text(eq_status).font(ctx.font_text).size(theme::TEXT_CAPTION)
                    .style(if state.eq_enabled { accent } else { p.text_muted })
            )
            .on_press(Message::ToggleEQ)
            .padding([6, 14])
            .style(iced::theme::Button::Custom(Box::new(PillToggleStyle {
                active: state.eq_enabled,
                accent,
                bg: p.elevated,
            }))),
        ].align_items(Alignment::Center),
        Space::with_height(12),
    ].spacing(4);

    // EQ Preset pills.
    let mut preset_row = row![].spacing(8);
    for preset in ALL_PRESETS.iter() {
        let is_selected = state.eq_preset_id == preset.id;
        let preset_id = preset.id.to_string();
        preset_row = preset_row.push(
            button(
                text(preset.name).font(ctx.font_text).size(11.0)
                    .style(if is_selected { accent } else { p.text_secondary })
            )
            .on_press(Message::SetEQPreset(preset_id))
            .padding([6, 12])
            .style(iced::theme::Button::Custom(Box::new(PillToggleStyle {
                active: is_selected,
                accent,
                bg: p.elevated,
            })))
        );
    }
    eq_section = eq_section.push(
        iced::widget::scrollable(preset_row)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Properties::default()
            ))
    );

    // EQ Band labels row.
    eq_section = eq_section.push(Space::with_height(8));
    let mut band_labels = row![].spacing(4);
    for (i, label) in BAND_LABELS.iter().enumerate() {
        let gain = state.eq_gains[i];
        let gain_text = format!("{:+.0}", gain);
        band_labels = band_labels.push(
            column![
                text(*label).font(ctx.font_text).size(10.0).style(p.text_muted),
                text(gain_text).font(ctx.font_rounded).size(11.0)
                    .style(if gain.abs() > 0.5 { accent } else { p.text_secondary }),
            ]
            .align_items(Alignment::Center)
            .width(Length::Fill)
        );
    }
    eq_section = eq_section.push(band_labels);

    content = content.push(
        container(eq_section).padding(24).style(theme::glass_card).width(Length::Fill)
    );

    // ─── Normalization Section ────────────────────────────────────────────────
    let mut norm_row = row![].spacing(8);
    for mode in &[NormalizationMode::Off, NormalizationMode::Track, NormalizationMode::Album] {
        let is_selected = state.normalization_mode == *mode;
        let m = *mode;
        norm_row = norm_row.push(
            button(
                text(mode.label()).font(ctx.font_text).size(13.0)
                    .style(if is_selected { accent } else { p.text_secondary })
            )
            .on_press(Message::SetNormalizationMode(m))
            .padding([8, 16])
            .style(iced::theme::Button::Custom(Box::new(PillToggleStyle {
                active: is_selected,
                accent,
                bg: p.elevated,
            })))
        );
    }

    let norm_section = container(column![
        text("Audio Normalization").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
        Space::with_height(8),
        text("Adjusts volume to match perceived loudness across tracks.")
            .font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
        Space::with_height(12),
        norm_row,
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(norm_section);

    // ─── Sleep Timer Section ─────────────────────────────────────────────────
    let mut timer_row = row![].spacing(8);
    for &(mins, label) in crate::timer::TIMER_PRESETS {
        let is_active = state.sleep_timer.as_ref()
            .map(|t| t.total_minutes() == mins)
            .unwrap_or(false);
        timer_row = timer_row.push(
            button(
                text(label).font(ctx.font_text).size(12.0)
                    .style(if is_active { accent } else { p.text_secondary })
            )
            .on_press(Message::SetSleepTimer(if is_active { 0 } else { mins }))
            .padding([8, 14])
            .style(iced::theme::Button::Custom(Box::new(PillToggleStyle {
                active: is_active,
                accent,
                bg: p.elevated,
            })))
        );
    }

    let timer_status = if let Some(timer) = &state.sleep_timer {
        format!("⏱ {} remaining", timer.remaining_display())
    } else {
        "No timer set".to_string()
    };

    let timer_section = container(column![
        row![
            text("Sleep Timer").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
            Space::with_width(Length::Fill),
            text(timer_status).font(ctx.font_text).size(theme::TEXT_CAPTION).style(
                if state.sleep_timer.is_some() { accent } else { p.text_muted }
            ),
        ].align_items(Alignment::Center),
        Space::with_height(12),
        timer_row,
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(timer_section);

    // ─── Crossfade Section ───────────────────────────────────────────────────
    use musico_playback::{CrossfadeCurve};

    let cf = &state.crossfade_config;
    let cf_status = if cf.enabled { "On" } else { "Off" };

    let mut curve_row = row![].spacing(8);
    for curve in &[CrossfadeCurve::Linear, CrossfadeCurve::EqualPower, CrossfadeCurve::Overlap] {
        let is_selected = cf.curve == *curve;
        let curve_id = curve.id().to_string();
        curve_row = curve_row.push(
            button(
                text(curve.label()).font(ctx.font_text).size(12.0)
                    .style(if is_selected { accent } else { p.text_secondary })
            )
            .on_press(Message::SetCrossfadeCurve(curve_id))
            .padding([6, 12])
            .style(iced::theme::Button::Custom(Box::new(PillToggleStyle {
                active: is_selected,
                accent,
                bg: p.elevated,
            })))
        );
    }

    let cf_section = container(column![
        row![
            text("Crossfade").font(ctx.font_rounded).size(theme::TEXT_TITLE).style(p.text_primary),
            Space::with_width(Length::Fill),
            button(
                text(cf_status).font(ctx.font_text).size(theme::TEXT_CAPTION)
                    .style(if cf.enabled { accent } else { p.text_muted })
            )
            .on_press(Message::ToggleCrossfade)
            .padding([6, 14])
            .style(iced::theme::Button::Custom(Box::new(PillToggleStyle {
                active: cf.enabled,
                accent,
                bg: p.elevated,
            }))),
        ].align_items(Alignment::Center),
        Space::with_height(8),
        text(format!("Duration: {:.1}s", cf.duration_secs))
            .font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_secondary),
        iced::widget::slider(0.5..=10.0, cf.duration_secs, Message::SetCrossfadeDuration)
            .width(Length::Fill)
            .style(iced::theme::Slider::Custom(Box::new(theme::VolumeSliderStyle(accent)))),
        Space::with_height(8),
        text("Curve").font(ctx.font_text).size(theme::TEXT_CAPTION).style(p.text_muted),
        curve_row,
    ]).padding(24).style(theme::glass_card).width(Length::Fill);

    content = content.push(cf_section);

    content.into()
}

struct PillToggleStyle {
    active: bool,
    accent: Color,
    bg: Color,
}
impl iced::widget::button::StyleSheet for PillToggleStyle {
    type Style = iced::Theme;
    fn active(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(if self.active {
                theme::with_alpha(self.accent, 0.15).into()
            } else {
                self.bg.into()
            }),
            border: iced::Border {
                color: if self.active { self.accent } else { Color::TRANSPARENT },
                width: if self.active { 1.5 } else { 0.0 },
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(theme::with_alpha(self.accent, 0.1).into()),
            border: iced::Border {
                color: if self.active { self.accent } else { theme::with_alpha(self.accent, 0.3) },
                width: 1.5,
                radius: 50.0.into(),
            },
            ..Default::default()
        }
    }
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

struct CountBadgeStyle(Color);
impl iced::widget::container::StyleSheet for CountBadgeStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(theme::with_alpha(self.0, 0.1).into()),
            border: iced::Border {
                color: theme::with_alpha(self.0, 0.25),
                width: 1.0,
                radius: 50.0.into(),
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

struct AccentBtnStyle(Color);
impl iced::widget::button::StyleSheet for AccentBtnStyle {
    type Style = iced::Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(self.0.into()),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: theme::RADIUS_MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let brighter = Color {
            r: (self.0.r * 1.15).min(1.0),
            g: (self.0.g * 1.15).min(1.0),
            b: (self.0.b * 1.15).min(1.0),
            a: self.0.a,
        };
        iced::widget::button::Appearance {
            background: Some(brighter.into()),
            text_color: Color::WHITE,
            border: iced::Border {
                radius: theme::RADIUS_MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
