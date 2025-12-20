use cirrus_egui::v1::notifier::Notifier;
use egui::{Context, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode};
use log::debug;
use roseate_core::{colour_type::ImageColourType, decoded_image::{DecodedImage, DecodedImageContent, Pixels}};

use crate::{image_handler::ImageHandler};

#[derive(Clone)]
pub enum ImageResource {
    Texture(TextureHandle),
    AnimatedTexture(Vec<(TextureHandle, f32)>),
    // Vector(egui::Image<'static>)
}

impl ImageHandler {
    pub(super) fn load_resource_update(&mut self, ctx: &Context, notifier: &mut Notifier) {
        if let Some(image) = &self.image {
            let reload_texture = match self.load_image_texture.try_lock() {
                Ok(load_image_texture_mutex) => *load_image_texture_mutex,
                Err(_) => false,
            };

            if reload_texture == false {
                return;
            }

            if let Some(decoded_image) = image.decoded.lock().unwrap().as_ref() {
                notifier.set_loading(Some("Converting image to texture to be uploaded to the GPU..."));

                match &decoded_image.content {
                    DecodedImageContent::Static(pixels) => {
                        debug!("Handing image texture to egui's backend to upload to the GPU...");
 
                        let texture = Self::decoded_image_pixels_to_egui_texture(
                            ctx, decoded_image, pixels
                        );

                        self.resource = Some(ImageResource::Texture(texture));
                    },
                    DecodedImageContent::Animated(frames) => {
                        debug!("Handing animated image textures to egui's backend to upload to the GPU...");

                        let mut textures: Vec<(TextureHandle, f32)> = Vec::new();

                        for (pixels, delay) in frames {
                            textures.push(
                                (
                                    Self::decoded_image_pixels_to_egui_texture(ctx, decoded_image, pixels),
                                    *delay
                                )
                            );
                        }

                        self.resource = Some(ImageResource::AnimatedTexture(textures))
                    },
                };

                // Texture handle doesn't need forgetting like egui::Image 
                // as it's smart enough to free itself from memory.

                ctx.forget_all_images(); // we want to free the rose image in 
                // image selection menu and all other potential images from memory 
                // that we no longer require loaded.

                notifier.unset_loading();
            }

            let optimizations = &self.image_optimizations;

            if optimizations.free_memory_after_gpu_upload && !optimizations.dynamic_sampling.is_some() {
                *image.decoded.lock().unwrap() = None;
            }

            *self.load_image_texture.lock().unwrap() = false;
            self.image_loading = false;
        }
    }

    fn decoded_image_pixels_to_egui_texture(ctx: &Context, decoded_image: &DecodedImage, pixels: &Pixels) -> TextureHandle {
        let texture_options = TextureOptions {
            magnification: TextureFilter::Linear,
            minification: TextureFilter::Linear,
            wrap_mode: TextureWrapMode::ClampToEdge,
            mipmap_mode: None,
        };

        let image_size = [decoded_image.size.0 as usize, decoded_image.size.1 as usize];

        ctx.load_texture(
            "static_image",
            match decoded_image.colour_type {
                ImageColourType::Grey8 | ImageColourType::Grey16 | ImageColourType::Grey32F | 
                ImageColourType::GreyA8 | ImageColourType::GreyA16 | ImageColourType::GreyA32F => {
                    debug!("Loading image as grey scale egui texture...");
                    egui::ColorImage::from_gray(
                        image_size, pixels
                    )
                },
                ImageColourType::Rgb8 | ImageColourType::Rgb16 | ImageColourType::Rgb32F => {
                    debug!("Loading image as rgb egui texture...");
                    egui::ColorImage::from_rgb(
                        image_size, pixels
                    )
                },
                ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F => {
                    debug!("Loading image as rgba egui texture...");
                    egui::ColorImage::from_rgba_unmultiplied(
                        image_size, pixels
                    )
                },
            },
            texture_options
        )
    }
}