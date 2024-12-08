use rayon::prelude::*;
use imagesize::ImageSize;
use std::{f32::consts::PI, sync::{Arc, Mutex}};

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

pub fn fast_downsample(
    pixels: Vec<u8>,
    image_size: &ImageSize,
    target_size: (u32, u32),
    has_alpha: bool
) -> (Vec<u8>, (u32, u32)) {
    let window_size: f32 = 3.0; // the window size that determines the level 
    // of influence the kernel has on each original pixel. Larger values result in more smoothing 
    // but may also result in slower computation time so beware.

    let (target_width, target_height) = target_size;

    let scale_factor = (image_size.width as f32 / target_width as f32)
        .max(image_size.height as f32 / target_height as f32);

    let new_width = (image_size.width as f32 / scale_factor) as u32;
    let new_height = (image_size.height as f32 / scale_factor) as u32;

    let kernel_lookup = precomputed_lanczos(window_size, scale_factor);

    let downsampled_pixels_buffer = Arc::new(
        Mutex::new(
            vec![0u8; (new_width * new_height * 3) as usize]
        )
    );

    let index_times = match has_alpha {
        true => 4,
        false => 3,
    };

    // '(0..new_height).into_par_iter()' allocates each vertical line to a CPU thread.
    (0..new_height).into_par_iter().for_each(|y| {
        let original_vertical_pos = y as f32 * scale_factor;
        let mut local_downsampled_pixels_buffer = vec![0u8; (new_width * 3) as usize];

        for x in 0..new_width {
            let original_horizontal_pos = x as f32 * scale_factor;

            let mut sum = 0.0;
            let mut rgb_sum = [0.0; 3]; // basically --> "R, G, B"

            let lanczos_window = window_size.ceil() as isize;

            // Here we iterate over the lanczos window which is a 
            // window that evenly surrounds the original pixel position.
            for vertical_offset in -lanczos_window..=lanczos_window {
                for horizontal_offset in -lanczos_window..=lanczos_window {
                    let relative_vertical_pos = (original_vertical_pos as isize + vertical_offset)
                        .clamp(0, (image_size.height - 1) as isize);
                    let relative_horizontal_pos = (original_horizontal_pos as isize + horizontal_offset)
                        .clamp(0, (image_size.width - 1) as isize);

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
                        relative_vertical_pos as usize * image_size.width + relative_horizontal_pos as usize
                    ) * index_times;

                    rgb_sum[0] += pixels[index] as f32 * weight; // red owo
                    rgb_sum[1] += pixels[index + 1] as f32 * weight; // green owo
                    rgb_sum[2] += pixels[index + 2] as f32 * weight; // blue owo
                    sum += weight;
                    // this has made me go insane!
                }
            }
            
            // work out the index of where the new pixels will lie (destination index).
            let destination_index: usize = (x * 3) as usize;

            local_downsampled_pixels_buffer[destination_index..destination_index + 3].copy_from_slice(&[
                (rgb_sum[0] / sum) as u8,
                (rgb_sum[1] / sum) as u8,
                (rgb_sum[2] / sum) as u8,
            ]);
        }

        // Copy thread-local downsampled pixels buffer into the global downsampled pixels buffer.
        let global_destination_index = (y * new_width * 3) as usize;

        downsampled_pixels_buffer.lock().unwrap()[global_destination_index..global_destination_index + (new_width * 3) as usize]
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