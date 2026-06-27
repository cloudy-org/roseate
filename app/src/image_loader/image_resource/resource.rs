use eframe::egui::{self, Context, TextureHandle, TextureOptions};
use log::debug;
use roseate_core::{colour_type::ImageColourType, decoded_image::{DecodedImage, DecodedImageContent}, pixels::Pixels};

#[derive(Clone)]
pub enum ImageResource {
    Texture(TextureHandle),
    AnimatedTexture(Vec<(TextureHandle, f32)>),
    // Vector(egui::Image<'static>)
}

impl ImageResource {
    pub fn from_decoded_image(
        ctx: &Context,
        decoded_image: &DecodedImage,
        texture_options: TextureOptions
    ) -> Self {
        debug!("Copying image's '{}' pixels into RGBA egui texture...", decoded_image.info.colour_type);

        match &decoded_image.content {
            DecodedImageContent::Static(pixels) => {
                debug!("Handing image texture to egui's backend to upload to the GPU...");

                let texture = Self::decoded_image_pixels_to_egui_texture(
                    ctx, decoded_image, &pixels, texture_options
                );

                Self::Texture(texture)
            },
            DecodedImageContent::Animated(frames) => {
                debug!("Handing animated image textures to egui's backend to upload to the GPU...");

                let mut textures: Vec<(TextureHandle, f32)> = Vec::new();

                for (pixels, delay) in frames {
                    textures.push(
                        (
                            Self::decoded_image_pixels_to_egui_texture(ctx, decoded_image, &pixels, texture_options),
                            *delay
                        )
                    );
                }

                Self::AnimatedTexture(textures)
            },
        }
    }

    pub fn from_rgba8_decoded_image_zero_copy(
        ctx: &Context,
        decoded_image: &mut DecodedImage,
        texture_options: TextureOptions
    ) -> Self {
        debug!("Image pixels will be directly consumed and uploaded to gpu to avoid memory spike...");

        let image_size = [decoded_image.size.0 as usize, decoded_image.size.1 as usize];

        assert!(
            matches!(decoded_image.info.colour_type, ImageColourType::Rgba8),
            "Wrong image resource from function was called, this is a logic error!"
        );

        let content = std::mem::replace(
            &mut decoded_image.content,
            // should Pixels have an empty enum??
            DecodedImageContent::Static(Pixels::U8(Vec::new())),
        );

        debug!("Zero-copying image's '{}' pixels into RGBA egui texture...", decoded_image.info.colour_type);

        match content {
            DecodedImageContent::Static(pixels) => {
                debug!("Handing image texture to egui's backend to upload to the GPU...");

                let colour_32_pixels = Self::rgba8_pixels_direct_consume_into_egui_color32(pixels);
                let colour_image = egui::ColorImage::new(image_size, colour_32_pixels);

                let texture = ctx.load_texture(
                    "static_image",
                    colour_image,
                    texture_options
                );

                ImageResource::Texture(texture)
            },
            DecodedImageContent::Animated(frames) => {
                debug!("Handing animated image textures to egui's backend to upload to the GPU...");

                let mut textures: Vec<(TextureHandle, f32)> = Vec::new();

                for (pixels, delay) in frames {
                    let colour_32_pixels = Self::rgba8_pixels_direct_consume_into_egui_color32(pixels);
                    let colour_image = egui::ColorImage::new(image_size, colour_32_pixels);

                    let texture = ctx.load_texture(
                        "static_image",
                        colour_image,
                        texture_options
                    );

                    textures.push((texture, delay));
                }

                ImageResource::AnimatedTexture(textures)
            },
        }
    }
}