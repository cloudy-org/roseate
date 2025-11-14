use std::time::Duration;

use cirrus_egui::v1::scheduler::{Scheduler};
use egui::{CursorIcon, Rect, Sense, Ui, Vec2};
use log::debug;

use crate::image::image::Image;

pub struct Viewport {
    pub zoom: f32,
    offset: Vec2,

    fit_to_window_animate_schedule: Scheduler,

    last_window_size: Vec2,
    last_fit_to_window_image_scale: f32,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,

            fit_to_window_animate_schedule: Self::schedule_fit_to_window_animation(),

            last_window_size: Vec2::ZERO,
            last_fit_to_window_image_scale: 1.0,
        }
    }

    fn schedule_fit_to_window_animation() -> Scheduler {
        debug!("The image has been scheduled to fit to window...");
        Scheduler::new(
            || {},
            Duration::from_secs_f32(0.3)
        )
    }

    fn calculate_fit_to_window_scale(&mut self, image_size: Vec2, fit_to_window: bool, animate_fit_to_window: bool) -> f32 {
        self.fit_to_window_animate_schedule.update();
        // should always be true if animation is 
        // disabled so the image scales instantly
        let can_fit_to_window = self.fit_to_window_animate_schedule.done || !animate_fit_to_window;

        if can_fit_to_window == false || fit_to_window == false || self.zoom != 1.0 {
            return self.last_fit_to_window_image_scale;
        }

        // we need the image size without padding to calculate 
        // what the image size scale (or zoom if you want to call it that) 
        // would be when scaled to fit the window size.
        let fit_to_window_image_scale = (self.last_window_size / image_size).min_elem().min(1.0);

        // println!("-> {}", fit_to_window_image_scale);

        self.last_fit_to_window_image_scale = fit_to_window_image_scale;

        fit_to_window_image_scale
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        image: &Image,
        egui_image: egui::Image,
        padding: f32,
        zoom_into_cursor: bool,
        fit_to_window: bool,
        animate_fit_to_window: bool
    ) {
        let window_size = ui.input(|i: &egui::InputState| i.viewport_rect()).size();

        // Schedule fit to window animation on window size 
        // change and reset that schedule if any more changes occur.
        if window_size != self.last_window_size {
            if animate_fit_to_window {
                self.fit_to_window_animate_schedule = Self::schedule_fit_to_window_animation();
            }

            println!("CHANGED!");

            // we keep track of the last known window size so we can 
            // determine when to schedule the fit to window animation.
            self.last_window_size = window_size;
        }

        let image_size = Vec2::new(
            image.image_size.0 as f32, image.image_size.1 as f32
        );
        let image_size_with_padding = image_size * padding;

        let fit_to_window_image_scale = self.calculate_fit_to_window_scale(image_size, fit_to_window, animate_fit_to_window);

        let fit_to_window_image_scale = match animate_fit_to_window {
            true => egui_animation::animate_eased(
                ui.ctx(),
                "fit_to_window_animation",
                fit_to_window_image_scale,
                1.5,
                simple_easing::cubic_in_out
            ),
            false => fit_to_window_image_scale
        };

        let (available_rect, response) = ui.allocate_exact_size(
            ui.available_size(),
            Sense::click_and_drag()
        );

        // The image size relative to viewport padding, zoom factor and fit to window size.
        let relative_image_size = (image_size_with_padding * self.zoom) * fit_to_window_image_scale;

        // Center the image in the center plus the offset for panning.
        // The "image_rect" controls entirely how the image should be painted in size and position.
        let image_rect = Rect::from_center_size(
            available_rect.center() + self.offset,
            relative_image_size,
        );

        // Respond to mouse zoom
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);

            if scroll.abs() > 0.0 {
                // Mouse position relative to screen coordinates.
                let mouse_position = match zoom_into_cursor {
                    true => ui.input(|i| i.pointer.latest_pos())
                        .unwrap_or(available_rect.center()),
                    // if configured to not zoom into cursor zoom into center of image instead
                    // TODO: test this!
                    false => available_rect.center()
                };

                let before_zoom = self.zoom;

                // TODO: configurable zoom speed (default is "0.005").
                let zoom_delta = (scroll * 0.005).exp(); // ".exp()" applies a smooth exponential zoom
                // TODO: configurable zoom factor limits, sensible values are currently in place but 
                // it would be FUNNY to zoom out of the entire galaxy and zoom in until maximum 32 bit 
                // unsigned floating point integer is reached (this is how it used to be before v1.0 alpha 17).
                self.zoom = (self.zoom * zoom_delta).clamp(0.01, 100.0);

                // Zoom into mouse cursor using offset.
                // TODO: fix zoom on cursor drifting ...you're not a nissan s15 silvia...
                let before_relative_mouse_position = (mouse_position - image_rect.center()) / before_zoom;
                let relative_mouse_position = (mouse_position - image_rect.center()) / self.zoom;

                self.offset += (relative_mouse_position - before_relative_mouse_position) * before_zoom;
            }
        }

        // Respond to image panning / grab
        if response.dragged() {
            let delta = response.drag_delta();
            self.offset += delta;

            // I kinda like the grabbing cursor. ツ
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        let egui_image = egui_image
            .corner_radius(10.0); // TODO: config to customize image corner radius.

        // Drawing the image to the viewport.
        egui_image.paint_at(ui, image_rect);
    }
}