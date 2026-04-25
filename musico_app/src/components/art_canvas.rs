use iced::widget::{container, image, Space};
use iced::{Color, Element, Length};
use ::image::{DynamicImage, GenericImageView};
use palette::{FromColor, Hsl, IntoColor, Srgb};

pub fn extract_dominant_color(img: &DynamicImage) -> Color {
    // Sample a 16x16 downscaled version
    let thumbnail = img.resize_exact(16, 16, ::image::imageops::FilterType::Nearest);
    
    let mut sum_r = 0.0;
    let mut sum_g = 0.0;
    let mut sum_b = 0.0;
    let mut count = 0.0;

    for (_, _, pixel) in thumbnail.pixels() {
        // Skip fully transparent pixels
        if pixel[3] < 255 { continue; }
        
        let srgb = Srgb::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        ).into_linear();
        
        sum_r += srgb.red;
        sum_g += srgb.green;
        sum_b += srgb.blue;
        count += 1.0;
    }

    if count == 0.0 {
        return Color::from_rgb(0.5, 0.5, 0.5);
    }

    let avg_linear = palette::LinSrgb::new(sum_r / count, sum_g / count, sum_b / count);
    let avg_srgb: Srgb = avg_linear.into_color();
    
    // Boost saturation slightly using HSL
    let mut hsl: Hsl = Hsl::from_color(avg_srgb);
    hsl.saturation = (hsl.saturation * 1.2).clamp(0.0, 1.0);
    
    let srgb: Srgb = hsl.into_color();
    
    Color::from_rgb(srgb.red, srgb.green, srgb.blue)
}

pub fn art_canvas<'a, Message: 'a>(
    handle: Option<image::Handle>,
    size: f32,
    radius: f32,
    fallback_color: Color,
) -> Element<'a, Message> {
    if let Some(h) = handle {
        container(
            image(h)
                .width(Length::Fixed(size))
                .height(Length::Fixed(size))
        )
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(iced::theme::Container::Custom(Box::new(ArtStyle {
            radius,
            bg: Color::TRANSPARENT,
        })))
        .into()
    } else {
        container(Space::new(Length::Fixed(size), Length::Fixed(size)))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .style(iced::theme::Container::Custom(Box::new(ArtStyle {
                radius,
                bg: fallback_color,
            })))
            .into()
    }
}

struct ArtStyle {
    radius: f32,
    bg: Color,
}

impl iced::widget::container::StyleSheet for ArtStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(self.bg.into()),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: self.radius.into(),
            },
            ..Default::default()
        }
    }
}
