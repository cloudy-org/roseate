use log::debug;

use crate::pixels::Pixels;

/// # Panics
/// This function panics if pixels are already u8.
pub fn squish_pixels_to_u8(pixels: &Pixels) -> Vec<u8> {
    debug!("Squishing '{pixels}' to u8 pixels...");

    match pixels {
        Pixels::U8(_) => panic!(
            "Squished pixels to u8 that were already \
                u8! Don't squish pixels to u8 that are already u8."
        ),
        Pixels::U16(pixels) => pixels
            .iter()
            .map(|&pixel| (pixel >> 8) as u8)
            .collect(),
        Pixels::F32(pixels) => pixels.iter()
            .map(|&pixel| (pixel.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect(),
    }
}