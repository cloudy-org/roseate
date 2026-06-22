use std::{hash::{DefaultHasher, Hash, Hasher}};

use cirrus_egui::notifier::Notifier;
use eframe::egui::{Context, TextureFilter, TextureOptions, TextureWrapMode};
use log::debug;
use roseate_core::{colour_type::ImageColourType, image_info::info::ImageInfo};

use crate::{image::Image, image_loader::{ImageLoader, image_resource::ImageResource}, image_selector::ImageSelector};

pub struct UploadedImage {
    pub image: Image,
    pub resource: ImageResource,
    pub image_info: ImageInfo,
    pub image_hash: u64,
}

impl ImageLoader {
    pub fn upload(&mut self, ctx: &Context, image_selector: &ImageSelector, notifier: &Notifier) -> Option<&UploadedImage> {
        match image_selector.get_image() {
            Some(image) => {
                let load_image_to_gpu = match self.load_image_to_gpu.try_lock() {
                    Ok(load_image_texture_mutex) => *load_image_texture_mutex,
                    Err(_) => false,
                };

                if load_image_to_gpu {
                    let can_free_memory_or_consume = self.image_optimizations.consume_pixels_during_gpu_upload;

                    if let Some(decoded_image) = image.decoded.lock().unwrap().as_mut() {
                        notifier.set_loading(Some("Converting image to texture to be uploaded to the GPU..."));

                        let texture_options = TextureOptions {
                            magnification: TextureFilter::Linear,
                            minification: TextureFilter::Linear,
                            wrap_mode: TextureWrapMode::ClampToEdge,
                            mipmap_mode: None,
                        };

                        let is_rgba = matches!(
                            decoded_image.info.colour_type,
                            ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F
                        );

                        self.uploaded_image = Some(
                            UploadedImage {
                                image: image.clone(),
                                resource: match can_free_memory_or_consume && is_rgba {
                                    true => ImageResource::from_rgba_decoded_image_zero_copy(ctx, decoded_image, texture_options),
                                    false => ImageResource::from_decoded_image(ctx, &decoded_image, texture_options),
                                },
                                image_info: decoded_image.info.clone(),
                                image_hash: {
                                    let mut hasher = DefaultHasher::new();

                                    image.hash(&mut hasher);

                                    hasher.finish()
                                }
                            }
                        );

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

                    *self.load_image_to_gpu.lock().unwrap() = false;
                    self.image_loading = false;
                }

                self.uploaded_image.as_ref()
            },
            None => None,
        }
    }

}