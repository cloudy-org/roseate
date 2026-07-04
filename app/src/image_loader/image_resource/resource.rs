use cirrus_egui::notifier::Notifier;
use eframe::egui::{Context, TextureHandle, TextureOptions};
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
        texture_options: TextureOptions,
        notifier: &mut Notifier,
    ) -> Self {
        debug!("Copying image's '{}' pixels into RGBA egui texture...", decoded_image.info.colour_type);

        match &decoded_image.content {
            DecodedImageContent::Static(pixels) => {
                debug!("Transforming static image pixels to egui image texture for uploading to the GPU...");

                let texture = Self::decoded_image_pixels_to_egui_texture(
                    ctx,
                    decoded_image,
                    pixels,
                    texture_options,
                    notifier,
                );

                Self::Texture(texture)
            },
            DecodedImageContent::Animated(frames) => {
                debug!("Transforming animated image pixels to egui image texture for uploading to the GPU...");

                let mut textures: Vec<(TextureHandle, f32)> = Vec::new();

                for (pixels, delay) in frames {
                    textures.push(
                        (
                            Self::decoded_image_pixels_to_egui_texture(
                                ctx,
                                decoded_image,
                                pixels,
                                texture_options,
                                notifier,
                            ),
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
        texture_options: TextureOptions,
    ) -> Self {
        debug!("Image pixels will be directly consumed and uploaded to gpu to avoid memory spike...");

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

                let texture = Self::rgba8_pixels_direct_consume_into_egui_texture(
                    ctx,
                    decoded_image,
                    pixels,
                    texture_options
                );

                ImageResource::Texture(texture)
            },
            DecodedImageContent::Animated(frames) => {
                debug!("Handing animated image textures to egui's backend to upload to the GPU...");

                let mut textures: Vec<(TextureHandle, f32)> = Vec::new();

                for (pixels, delay) in frames {
                    let texture = Self::rgba8_pixels_direct_consume_into_egui_texture(
                        ctx,
                        decoded_image,
                        pixels,
                        texture_options
                    );

                    textures.push((texture, delay));
                }

                ImageResource::AnimatedTexture(textures)
            },
        }
    }
}