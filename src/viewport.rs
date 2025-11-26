use std::{hash::{DefaultHasher, Hash}, time::Duration};

use log::debug;
use std::hash::Hasher;
use cirrus_egui::v1::scheduler::{Scheduler};
use egui::{CursorIcon, Key, Rect, Sense, Ui, Vec2};

use crate::image::image::Image;

pub struct Viewport {
    pub zoom: f32,
    offset: Vec2,
    is_busy: bool,

    reset_zoom: Option<f32>,
    reset_offset: Option<Vec2>,
    // we use these booleans to check if we are currently 
    // in the animation of resetting zoom or offset in our 
    // update loop.
    zoom_is_resetting: bool,
    offset_is_resetting: bool,

    // we use a scheduler for fit to window 
    // animation so we can have a nice delay effect.
    fit_to_window_animate_schedule: Scheduler,

    last_window_size: Vec2,
    last_fit_to_window_image_scale: f32,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,
            is_busy: false,

            reset_zoom: None,
            reset_offset: None,
            zoom_is_resetting: false,
            offset_is_resetting: false,

            fit_to_window_animate_schedule: Self::get_fit_to_window_animation_schedule(),

            last_window_size: Vec2::ZERO,
            last_fit_to_window_image_scale: 1.0,
        }
    }

    fn get_fit_to_window_animation_schedule() -> Scheduler {
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
        animate_fit_to_window: bool,
        animate_reset: bool
    ) {
        let window_size = ui.input(|i: &egui::InputState| i.viewport_rect()).size();

        self.pan_and_zoom_reset_update(ui, window_size, animate_reset);

        // Schedule fit to window animation on window size 
        // change and reset that schedule if any more changes occur.
        if window_size != self.last_window_size {
            if animate_fit_to_window {
                self.fit_to_window_animate_schedule = Self::get_fit_to_window_animation_schedule();
            }

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
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if response.hovered() {
            let center_of_image = image_rect.center();

            if scroll.abs() > 0.0 {
                // Mouse position relative to screen coordinates.
                let mouse_position = match zoom_into_cursor {
                    true => ui.input(|i| i.pointer.latest_pos())
                        .unwrap_or(center_of_image),
                    // if configured to not zoom into cursor zoom into center of image instead
                    false => center_of_image
                };

                let before_zoom = self.zoom;

                // TODO: configurable zoom speed (default is "0.005").
                let zoom_delta = (scroll * 0.005).exp(); // ".exp()" applies a smooth exponential zoom
                // TODO: configurable zoom factor limits, sensible values are currently in place but 
                // it would be FUNNY to zoom out of the entire galaxy and zoom in until maximum 32 bit 
                // unsigned floating point integer is reached (this is how it used to be before v1.0 alpha 17).
                self.zoom = (self.zoom * zoom_delta).clamp(0.01, 100.0);

                // Zoom into mouse cursor using offset.
                let before_relative_mouse_position = (mouse_position - center_of_image) / before_zoom;
                let relative_mouse_position = (mouse_position - center_of_image) / self.zoom;

                self.offset += (relative_mouse_position - before_relative_mouse_position) * self.zoom;
            }
        }

        // Respond to image panning / grab
        if response.dragged() {
            let delta = response.drag_delta();
            self.offset += delta;

            // I kinda like the grabbing cursor. ツ
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        // the viewport is busy if the user is interacting with it (scrolling, zooming, etc).
        self.is_busy = response.dragged() || response.hovered() && scroll.abs() > 0.0;

        let egui_image = egui_image
            .corner_radius(10.0); // TODO: config to customize image corner radius.

        // Drawing the image to the viewport.
        egui_image.paint_at(ui, image_rect);
    }

    fn pan_and_zoom_reset_update(&mut self, ui: &Ui, window_size: Vec2, animate_reset: bool) {
        if ui.ctx().input(|i| i.key_pressed(Key::R)) {
            debug!("Force resetting zoom and pan...");

            self.reset_zoom = Some(self.zoom);
            self.reset_offset = Some(self.offset);
        }

        let pan_bounds_to_not_exceed = (window_size / 1.9) * self.zoom;

        let is_out_of_bounds = self.offset.x > pan_bounds_to_not_exceed.x || 
            self.offset.y > pan_bounds_to_not_exceed.y || 
            self.offset.x < -pan_bounds_to_not_exceed.x || 
            self.offset.y < -pan_bounds_to_not_exceed.y || 
            self.zoom < 1.0;

        if is_out_of_bounds {
            debug!("You panned or zoomed out of bounds, resetting offset and zoom...");

            if !self.zoom_is_resetting { self.reset_zoom = Some(self.zoom) };
            if !self.offset_is_resetting { self.reset_offset = Some(self.offset) };
        }

        if self.is_busy {
            return;
        }

        if let Some(offset_before_reset) = self.reset_offset {
            // we only set self.offset_is_resetting to true at the end
            // so if it's false we know this is the first pass.
            let first_pass = !self.offset_is_resetting;

            self.offset = match animate_reset {
                true => Vec2::new(
                    Self::animate_to(
                        ui,
                        "reset_offset_x_animation",
                        offset_before_reset.x,
                        0.0,
                        0.5,
                        first_pass
                    ),
                    Self::animate_to(
                        ui,
                        "reset_offset_y_animation",
                        offset_before_reset.y,
                        0.0,
                        0.5,
                        first_pass
                    ),
                ),
                false => Vec2::ZERO
            };

            self.offset_is_resetting = true;

            if self.offset == Vec2::ZERO {
                self.reset_offset = None;
                self.offset_is_resetting = false;

                debug!("Pan reset done!");
            }
        }

        if let Some(zoom_before_reset) = self.reset_zoom {
            self.zoom = match animate_reset {
                true => Self::animate_to(
                    ui,
                    "reset_zoom_animation",
                    zoom_before_reset,
                    1.0,
                    0.5,
                    // we only set self.zoom_is_resetting to true at the end
                    // so if it's false we know this is the first pass.
                    !self.zoom_is_resetting
                ),
                false => 1.0
            };

            self.zoom_is_resetting = true;

            if self.zoom == 1.0 {
                self.reset_zoom = None;
                self.zoom_is_resetting = false;

                debug!("Zoom reset done!");
            }
        }
    }

    fn animate_to(ui: &Ui, animation_id: &str, current: f32, destination: f32, animation_time: f32, is_first_pass: bool) -> f32 {
        let mut hasher = DefaultHasher::new();
        animation_id.hash(&mut hasher);
        (current as i32).hash(&mut hasher);

        let animated_value = egui_animation::animate_eased(
            ui.ctx(),
            hasher.finish(),
            // we can only animate forward values so we use 
            // 0 here to represent current value and 1 to 
            // represent destination.
            match is_first_pass {
                true => 0.0, // current (or value before 'animate_to', e.g: 'zoom_before_reset')
                false => 1.0, // destination (e.g: 'self.zoom = 1.0')
            },
            animation_time,
            simple_easing::cubic_in_out
        );

        let animated_current = current + (destination - current) * animated_value;

        // if animated_current == destination {
        //     // TODO: switch to some type of method to only 
        //     // clear this specific animation and nothing else 
        //     // as this may cause problems.
        //     // 
        //     // Like 'clear_animation("animation_id")'.
        //     ui.ctx().clear_animations();

        //     debug!("Animations cleared due to '{}'!", animation_id);
        // }

        animated_current
    }
}