use std::time::{Duration, Instant};

use eframe::egui::{Rect, Vec2};
use imagesize::ImageSize;
use log::debug;

use crate::config::config::Config;

/// Struct that handles the image auto resizing with window size.
pub struct WindowScaling {
    scale_factor: f32,
    resize_to_window_timer: Option<Instant>,
    padding: f32
}

impl WindowScaling {
    pub fn new(config: &Config) -> Self {
        Self {
            scale_factor: 1.0,
            resize_to_window_timer: Some(Instant::now()),
            padding: config.ui.viewport.padding
        }
    }

    /// Resizes the image to the window size after a short delay 
    /// or later in the update loop (hence being named 'schedule_').
    pub fn schedule_scale_image_to_window_size(&mut self) {
        debug!("The image has been scheduled to resize to the window size.");
        self.resize_to_window_timer = Some(Instant::now());
    }

    pub fn update(&mut self, window_rect: &Rect, actual_image_size: &ImageSize) {
        if let Some(timer) = self.resize_to_window_timer {
            // If the timer has expired (no new resize events)
            if timer.elapsed() >= Duration::from_millis(300) {
                // Reset the timer
                self.resize_to_window_timer = None;

                let actual_padding = 1.00 - (self.padding.clamp(0.0, 50.0) / 100.0);

                // padding between the image and the edge of the window.
                let scale_x = window_rect.width() / actual_image_size.width as f32 * actual_padding;
                let scale_y = window_rect.height() / actual_image_size.height as f32 * actual_padding;

                let scale_factor = scale_x.min(scale_y); // Scale uniformly.

                // Make sure scale_factor doesn't exceed the original size (1).
                self.scale_factor = scale_factor.min(1.0);
            }
        }
    }

    pub fn relative_image_size(&self, image_size: Vec2) -> Vec2 {
        Vec2::new(
            image_size.x as f32 * self.scale_factor,
            image_size.y as f32 * self.scale_factor
        )
    }
}