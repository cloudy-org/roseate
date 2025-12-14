use rayon::prelude::*;
use std::{f32::consts::PI, sync::{Arc, Mutex}};

use super::image::ImageSizeT;

// math :akko_shrug:
// SINNNNNNN, SIN CITY WASN'T MADE FOR YOU!!! ANGLES LIKEEEEE YOUUUU!
fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        return 1.0
    }

    (PI * x).sin() / (PI * x)
}

// Get Lanczos kernel for resampling
// Reference: https://en.wikipedia.org/wiki/Lanczos_resampling#Lanczos_kernel
fn lanczos_kernel(x: f32, a: f32) -> f32 {
    if x.abs() < a {
        sinc(x) * sinc(x / a)
    } else {
        0.0
    }
}

fn precomputed_lanczos(window_size: f32, scale_factor: f32) -> Vec<f32> {
    let max_distance = (window_size * scale_factor).ceil() as usize;
    let mut lookup = vec![0.0; max_distance + 1];
    for i in 0..=max_distance {
        let distance = i as f32 / scale_factor;
        lookup[i] = lanczos_kernel(distance, window_size);
    }
    lookup
}

pub(super) fn fast_downsample(
    pixels: &Vec<u8>,
    image_size: &ImageSizeT,
    target_size: (u32, u32),
    has_alpha: bool
) -> (Vec<u8>, ImageSizeT) {
    let window_size: f32 = 3.0; // the window size that determines the level 
    // of influence the kernel has on each original pixel. Larger values result in more smoothing 
    // but may also result in slower computation time so beware.

    let (target_width, target_height) = target_size;

    let scale_factor = (image_size.0 as f32 / target_width as f32)
        .max(image_size.1 as f32 / target_height as f32);

    let new_width = (image_size.0 as f32 / scale_factor) as u32;
    let new_height = (image_size.1 as f32 / scale_factor) as u32;

    let kernel_lookup = precomputed_lanczos(window_size, scale_factor);

    let index_times: u32 = match has_alpha {
        true => 4,
        false => 3,
    };

    let downsampled_pixels_buffer = Arc::new(
        Mutex::new(
            vec![0u8; (new_width * new_height * index_times) as usize]
        )
    );

    let rgb_index_range: Vec<usize> = (0..index_times as usize).collect();

    // TODO: do not use every single thread and have thread count configurable.

    // '(0..new_height).into_par_iter()' allocates each vertical line to a CPU thread.
    (0..new_height).into_par_iter().for_each(|y| {
        let original_vertical_pos = y as f32 * scale_factor;
        let mut local_downsampled_pixels_buffer = vec![0u8; (new_width * index_times) as usize];

        for x in 0..new_width {
            let original_horizontal_pos = x as f32 * scale_factor;

            let mut sum = 0.0;

            // basically --> "R,G,B", "R,G,B,A", etc etc
            let mut rgb_sum = vec![0.0; index_times as usize];

            let lanczos_window = window_size.ceil() as isize;

            // Here we iterate over the lanczos window which is a 
            // window that evenly surrounds the original pixel position.
            for vertical_offset in -lanczos_window..=lanczos_window {
                for horizontal_offset in -lanczos_window..=lanczos_window {
                    let relative_vertical_pos = (original_vertical_pos as isize + vertical_offset)
                        .clamp(0, (image_size.1 - 1) as isize);
                    let relative_horizontal_pos = (original_horizontal_pos as isize + horizontal_offset)
                        .clamp(0, (image_size.0 - 1) as isize);

                    // Each neighbouring pixel's influence is calculated based on it's 
                    // distance from the relative and original pixel position using the Lanczos kernel.
                    // 
                    // In this case below this we use a predetermined lanczos 
                    // (kernel_lookup) instead of calling the lanczos kernal each time.
                    let relative_horizontal_distance = (relative_horizontal_pos as f32 - original_horizontal_pos).abs() as usize;
                    let relative_vertical_distance = (relative_vertical_pos as f32 - original_vertical_pos).abs() as usize;

                    // Weights determine how much each original pixel contributes to the new resized pixel RGB colour.
                    let weight = kernel_lookup[relative_horizontal_distance] * kernel_lookup[relative_vertical_distance];
                    let index = (
                        relative_vertical_pos as usize * image_size.0 as usize + relative_horizontal_pos as usize
                    ) * index_times as usize;

                    for rgb_index in &rgb_index_range {
                        rgb_sum[*rgb_index] += pixels[index + rgb_index] as f32 * weight;
                    }

                    sum += weight;
                    // this has made me go insane!
                }
            }

            // work out the index of where the new pixels will lie (destination index).
            let destination_index: usize = (x * index_times) as usize;

            local_downsampled_pixels_buffer[destination_index..destination_index + index_times as usize].copy_from_slice(
                rgb_index_range.iter()
                    .map(|rgb_index| (rgb_sum[*rgb_index] / sum) as u8)
                    .collect::<Vec<u8>>()
                    .as_slice()
            );
        }

        // Copy thread-local downsampled pixels buffer into the global downsampled pixels buffer.
        let global_destination_index = (y * new_width * index_times) as usize;

        downsampled_pixels_buffer.lock().unwrap()[global_destination_index..global_destination_index + (new_width * index_times) as usize]
            .copy_from_slice(&local_downsampled_pixels_buffer);
    });

    (
        Arc::try_unwrap(downsampled_pixels_buffer)
            .expect("Arc unwrap of downsampled pixels failed!")
            .into_inner()
            .unwrap(),
        (
            new_width,
            new_height
        )
    )
}