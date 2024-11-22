use rayon::prelude::*;
use imagesize::ImageSize;
use std::sync::{Arc, Mutex};

pub fn fast_downsample(pixels:Vec<u8>, image_size: &ImageSize, target_size: (u32, u32)) -> (Vec<u8>, (u32, u32)) {
    let (target_width, target_height) = target_size;

    let scale_factor = (image_size.width as f32 / target_width as f32)
        .max(image_size.height as f32 / target_height as f32);

    let new_width = (image_size.width as f32 / scale_factor) as u32;
    let new_height = (image_size.height as f32 / scale_factor) as u32;

    let downsampled_pixels = Arc::new(
        Mutex::new(
            vec![0u8; (new_width * new_height * 3) as usize]
        )
    );

    // '(0..new_height).into_par_iter()' allocates each vertical line to a CPU thread.
    (0..new_height).into_par_iter().for_each(|y| {
        let original_vertical_pos = y as f32 * scale_factor;

        for x in 0..new_width {
            let original_horizontal_pos = x as f32 * scale_factor;
            let mut rgb_sum = [0u16; 3]; // basically --> "R, G, B"

            let square_block_size: usize = 2;

            // Here we basically take a 2x2 square block (4 pixels) from the source image so we can
            // average their colour values to downscale that to one pixel in the downsampled image.
            for vertical_offset in 0..square_block_size {
                for horizontal_offset in 0..square_block_size {
                    let relative_vertical_pos = (original_vertical_pos as usize + vertical_offset)
                        .min(image_size.height - 1);
                    let relative_horizontal_pos = (original_horizontal_pos as usize + horizontal_offset)
                        .min(image_size.width - 1);

                    let index = (
                        relative_vertical_pos * image_size.width + relative_horizontal_pos
                    ) * 3;

                    rgb_sum[0] += pixels[index] as u16; // red owo
                    rgb_sum[1] += pixels[index + 1] as u16; // green owo
                    rgb_sum[2] += pixels[index + 2] as u16; // blue owo
                    // this has made me go insane!
                }
            }

            // work out the index of where the new pixels will lie (destination index).
            let destination_index: usize = ((y * new_width + x) * 3) as usize;

            let mut downsampled_pixels = downsampled_pixels.lock().unwrap();

            // compute the average colour values
            let square_block_pixel_count = u16::pow(square_block_size as u16, 2);

            downsampled_pixels[destination_index..destination_index + 3].copy_from_slice(&[
                (rgb_sum[0] / square_block_pixel_count) as u8,
                (rgb_sum[1] / square_block_pixel_count) as u8,
                (rgb_sum[2] / square_block_pixel_count) as u8,
            ]);
        }
    });

    (
        Arc::try_unwrap(downsampled_pixels)
            .expect("Arc unwrap of downsampled pixels failed!")
            .into_inner()
            .unwrap(),
        (
            new_width, 
            new_height
        )
    )
}