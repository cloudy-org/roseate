use cirrus_egui::v1::notifier::Notifier;
use egui::{Color32, Context, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode};
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

            let optimizations = &self.image_optimizations;

            let can_free_memory_or_consume = optimizations.consume_pixels_during_gpu_upload && !optimizations.dynamic_sampling.is_some();

            if let Some(decoded_image) = image.decoded.lock().unwrap().as_mut() {
                notifier.set_loading(Some("Converting image to texture to be uploaded to the GPU..."));

                let texture_options = TextureOptions {
                    magnification: TextureFilter::Linear,
                    minification: TextureFilter::Linear,
                    wrap_mode: TextureWrapMode::ClampToEdge,
                    mipmap_mode: None,
                };

                let is_rgba = matches!(
                    decoded_image.colour_type,
                    ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F
                );

                self.resource = match can_free_memory_or_consume && is_rgba {
                    true => Some(
                        Self::rgba_zero_copy_decoded_image_to_resource(ctx, decoded_image, texture_options)
                    ),
                    false => Some(Self::decoded_image_to_resource(ctx, &decoded_image, texture_options)),
                };

                // Texture handle doesn't need forgetting like egui::Image 
                // as it's smart enough to free itself from memory.

                ctx.forget_all_images(); // we want to free the rose image in 
                // image selection menu and all other potential images from memory 
                // that we no longer require loaded.

                notifier.unset_loading();
            }

            if can_free_memory_or_consume {
                debug!("Freeing decoded image from memory...");
                *image.decoded.lock().unwrap() = None;
            }

            *self.load_image_texture.lock().unwrap() = false;
            self.image_loading = false;
        }
    }

    fn decoded_image_to_resource(
        ctx: &Context,
        decoded_image: &DecodedImage,
        texture_options: TextureOptions
    ) -> ImageResource {
        debug!("Copying image's '{}' pixels into RGBA egui texture...", decoded_image.colour_type);

        match &decoded_image.content {
            DecodedImageContent::Static(pixels) => {
                debug!("Handing image texture to egui's backend to upload to the GPU...");

                let texture = Self::decoded_image_pixels_to_egui_texture(
                    ctx, decoded_image, &pixels, texture_options
                );

                ImageResource::Texture(texture)
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

                ImageResource::AnimatedTexture(textures)
            },
        }
    }

    fn decoded_image_pixels_to_egui_texture(ctx: &Context, decoded_image: &DecodedImage, pixels: &Pixels, texture_options: TextureOptions) -> TextureHandle {
        let image_size = [decoded_image.size.0 as usize, decoded_image.size.1 as usize];

        let texture = ctx.load_texture(
            "static_image",
            match decoded_image.colour_type {
                ImageColourType::Grey8 | ImageColourType::Grey16 | ImageColourType::Grey32F | 
                ImageColourType::GreyA8 | ImageColourType::GreyA16 | ImageColourType::GreyA32F => {
                    egui::ColorImage::from_gray(image_size, &pixels)
                },
                ImageColourType::Rgb8 | ImageColourType::Rgb16 | ImageColourType::Rgb32F => {
                    egui::ColorImage::from_rgb(image_size, &pixels)
                },
                ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F => {
                    egui::ColorImage::from_rgba_unmultiplied(image_size, &pixels)
                },
            },
            texture_options
        );

        texture
    }

    fn rgba_zero_copy_decoded_image_to_resource(
        ctx: &Context,
        decoded_image: &mut DecodedImage,
        texture_options: TextureOptions
    ) -> ImageResource {
        debug!("Image pixels will be directly consumed and uploaded to gpu to avoid memory spike...");

        let image_size = [decoded_image.size.0 as usize, decoded_image.size.1 as usize];

        assert!(
            matches!(decoded_image.colour_type, ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F),
            "Wrong into egui texture function was called, this is a logic error!"
        );

        let content = std::mem::replace(
            &mut decoded_image.content,
            DecodedImageContent::Static(Vec::new()),
        );

        debug!("Zero-copying image's '{}' pixels into RGBA egui texture...", decoded_image.colour_type);

        match content {
            DecodedImageContent::Static(pixels) => {
                debug!("Handing image texture to egui's backend to upload to the GPU...");

                let colour_32_pixels = Self::rgba8_pixels_direct_consume_into_color32(pixels);
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
                    let colour_32_pixels = Self::rgba8_pixels_direct_consume_into_color32(pixels);
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

    pub fn rgba8_pixels_direct_consume_into_color32(mut pixels: Pixels) -> Vec<Color32> {
        assert!(pixels.len() % 4 == 0);

        let pointer = pixels.as_mut_ptr() as *mut Color32;
        let length = pixels.len() / 4;
        let capacity = pixels.capacity() / 4;

        std::mem::forget(pixels);

        let mut colour_32_vec: Vec<Color32> = unsafe { Vec::from_raw_parts(pointer, length, capacity) };

        // "Color32" wants premultiplied RGBA only and zero copying copies in 
        // unmultiplied RGBA so we need to convert it to a premultiplied RGBA colour.
        // 
        // Despite what you think, this will not lead to a duplication of memory 
        // (a new vec being assigned), the Rust compiler is smart enough here to 
        // perform an "in-place" optimization while iterating through the vec. :)
        // 
        // https://doc.rust-lang.org/stable/std/iter/trait.FromIterator.html#impl-FromIterator%3CT%3E-for-Vec%3CT%3E
        // 
        // I fucking love compilers! https://stackoverflow.com/a/78682795
        colour_32_vec = colour_32_vec.iter()
            .map(|colour| Color32::from_rgba_unmultiplied(
                colour.r(), colour.g(), colour.b(), colour.a()
            ))
            .collect();

        colour_32_vec.shrink_to_fit();
        colour_32_vec
    }
}