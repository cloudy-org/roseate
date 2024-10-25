use std::time::{Duration, Instant};

use eframe::egui::Rect;
use imagesize::ImageSize;

/// Struct that handles the image auto resizing with window size.
pub struct WindowScaling {
    scale_factor: f32,
    resize_to_window_timer: Option<Instant>,
}

impl WindowScaling {
    pub fn new() -> Self {
        Self {
            scale_factor: 1.0,
            resize_to_window_timer: Some(Instant::now())
        }
    }

    /// Resizes the image to the window size after a short delay 
    /// or later in the update loop (hence being named 'schedule_').
    pub fn schedule_image_scale_to_window_size(&mut self) {
        self.resize_to_window_timer = Some(Instant::now());
    }

    pub fn update(&mut self, window_rect: &Rect, actual_image_size: &ImageSize) {
        if let Some(timer) = self.resize_to_window_timer {
            // If the timer has expired (no new resize events)
            if timer.elapsed() >= Duration::from_millis(300) {
                // Reset the timer
                self.resize_to_window_timer = None;

                let scale_x = window_rect.width() / actual_image_size.width as f32;
                let scale_y = window_rect.height() / actual_image_size.height as f32;

                let scale_factor = scale_x.min(scale_y); // Scale uniformly.

                // Make sure scale_factor doesn't exceed the original size (1).
                self.scale_factor = scale_factor.min(1.0);
            }
        }
    }

    pub fn get_scaled_image_size(&self, actual_image_size: ImageSize) -> (f32, f32) {
        (
            actual_image_size.width as f32 * self.scale_factor,
            actual_image_size.height as f32 * self.scale_factor
        )
    }
}