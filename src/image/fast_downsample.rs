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

pub fn fast_downsample(
    pixels: Vec<u8>,
    image_size: &ImageSize,
    target_size: (u32, u32)
) -> (Vec<u8>, (u32, u32)) {
    let window_size: f32 = 3.0; // the window size that determines the level 
    // of influence the kernel has on each original pixel. Larger values result in more smoothing 
    // but may also result in slower computation time so beware.

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
                    let lanczos_x = lanczos_kernel(
                        (relative_horizontal_pos as f32 - original_horizontal_pos) / scale_factor,
                        window_size,
                    );
                    let lanczos_y = lanczos_kernel(
                        (relative_vertical_pos as f32 - original_vertical_pos) / scale_factor,
                        window_size,
                    );

                    // Weights determine how much each original pixel contributes to the new resized pixel RGB colour.
                    let weight = lanczos_x * lanczos_y;

                    let index = (
                        relative_vertical_pos as usize * image_size.width + relative_horizontal_pos as usize
                    ) * 3;

                    rgb_sum[0] += pixels[index] as f32 * weight; // red owo
                    rgb_sum[1] += pixels[index + 1] as f32 * weight; // green owo
                    rgb_sum[2] += pixels[index + 2] as f32 * weight; // blue owo
                    sum += weight;
                    // this has made me go insane!
                }
            }

            // work out the index of where the new pixels will lie (destination index).
            let destination_index: usize = ((y * new_width + x) * 3) as usize;

            let mut downsampled_pixels = downsampled_pixels.lock().unwrap();

            downsampled_pixels[destination_index..destination_index + 3].copy_from_slice(&[
                (rgb_sum[0] / sum) as u8,
                (rgb_sum[1] / sum) as u8,
                (rgb_sum[2] / sum) as u8,
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