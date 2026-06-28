use eframe::egui::{self, Color32, Context, TextureHandle, TextureOptions};
use roseate_core::{colour_type::ImageColourType, decoded_image::{DecodedImage}, pixels::Pixels};

use crate::image_loader::image_resource::ImageResource;


impl ImageResource {
    pub(super) fn decoded_image_pixels_to_egui_texture(ctx: &Context, decoded_image: &DecodedImage, pixels: &Pixels, texture_options: TextureOptions) -> TextureHandle {
        let image_size = [decoded_image.size.0 as usize, decoded_image.size.1 as usize];

        let texture = ctx.load_texture(
            "static_image",
            match pixels {
                Pixels::U8(pixels) => Self::u8_pixels_into_egui_color_image(pixels, image_size, decoded_image.info.colour_type),
                // NOTE: U16 and F32 images will display completely washed out.
                // 
                // Currently we do not handle any kind of HDR data and u16 
                // and f32 images just get linearly converted to u8 or clamped.
                // 
                // Also i'm currently still learning about bit depth and everything else around it. ~ Goldy
                Pixels::U16(pixels) => {
                    let u8_pixels: Vec<u8> = pixels
                        .iter()
                        .map(|&pixel| (pixel / 257) as u8)
                        .collect();

                    Self::u8_pixels_into_egui_color_image(
                        &u8_pixels,
                        image_size,
                        decoded_image.info.colour_type
                    )
                },
                Pixels::F32(pixels) => {
                    let u8_pixels: Vec<u8> = pixels
                        .iter()
                        .map(|&pixel| (pixel.clamp(0.0, 1.0) * 255.0).round() as u8)
                        .collect();

                    Self::u8_pixels_into_egui_color_image(
                        &u8_pixels,
                        image_size,
                        decoded_image.info.colour_type
                    )
                }
            },
            texture_options
        );

        texture
    }

    pub(super) fn rgba8_pixels_direct_consume_into_egui_color32(mut pixels: Pixels) -> Vec<Color32> {
        assert!(pixels.len() % 4 == 0);

        let length = pixels.len() / 4;
        let capacity = match &pixels {
            Pixels::U8(pixels) => pixels.capacity() / 4,
            Pixels::U16(_) => panic!("u16 pixels are not allowed in `rgba8_pixels_direct_consume_into_color32`!"),
            Pixels::F32(_) => panic!("f32 pixels are not allowed in `rgba8_pixels_direct_consume_into_color32`!"),
        };

        let pointer = pixels.as_mut_ptr() as *mut Color32;

        match pixels {
            Pixels::U8(pixels) => std::mem::forget(pixels),
            Pixels::U16(_) => unreachable!(),
            Pixels::F32(_) => unreachable!(),
        }

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

    fn u8_pixels_into_egui_color_image(raw_vec_pixels: &Vec<u8>, image_size: [usize; 2], colour_type: ImageColourType) -> egui::ColorImage {
        match colour_type {
            ImageColourType::Grey8 | ImageColourType::Grey16 | ImageColourType::Grey32F | 
            ImageColourType::GreyA8 | ImageColourType::GreyA16 | ImageColourType::GreyA32F => {
                egui::ColorImage::from_gray(image_size, raw_vec_pixels)
            },
            ImageColourType::Rgb8 | ImageColourType::Rgb16 | ImageColourType::Rgb32F => {
                egui::ColorImage::from_rgb(image_size, raw_vec_pixels)
            },
            ImageColourType::Rgba8 | ImageColourType::Rgba16 | ImageColourType::Rgba32F => {
                egui::ColorImage::from_rgba_unmultiplied(image_size, raw_vec_pixels)
            },
        }
    }
}