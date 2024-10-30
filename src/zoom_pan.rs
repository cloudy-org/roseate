use std::time::{Duration, Instant};

use rand::Rng;
use log::debug;
use eframe::egui::{Context, Key, Pos2, Response, Vec2};

/// Struct that controls the zoom and panning of the image.
pub struct ZoomPan {
    zoom_factor: f32,
    last_zoom_factor: f32,
    is_panning: bool,
    pan_offset: Vec2,
    drag_start: Option<Pos2>,
    reset_pan_offset: Option<ResetManager>,
    reset_scale_factor: Option<ResetManager>
}

struct ResetManager {
    timer: Instant,
    delay: Duration,
    animation_id: u32,
    in_animation: bool
}

impl ZoomPan {
    pub fn new() -> Self {
        Self {
            zoom_factor: 1.0,
            last_zoom_factor: 1.0,
            drag_start: None,
            is_panning: false,
            pan_offset: Vec2::ZERO,
            reset_pan_offset: None,
            reset_scale_factor: None
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        // Reset pan offset when scheduled and handle animation.
        self.pan_reset_update(ctx);

        // Reset zoom factor scale when scheduled and handle animation.
        self.scale_reset_update(ctx);

        // Schedule scale factor reset if the user zoom out too far.
        if self.zoom_factor < 1.0 {
            self.schedule_scale_reset(Duration::from_millis(500))
        }
    }

    pub fn handle_reset_input(&mut self, ctx: &Context) {
        // TODO: make the key customizable when we get a config.
        if ctx.input(|i| i.key_pressed(Key::R)) {
            self.schedule_pan_reset(Duration::ZERO);
            self.schedule_scale_reset(Duration::ZERO);
        }
    }

    // Method to handle zoom input (scrolling)
    pub fn handle_zoom_input(&mut self, ctx: &Context) {
        self.last_zoom_factor = self.zoom_factor;

        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);

        if scroll_delta != 0.0 {
            let zoom_delta = scroll_delta * self.zoom_factor * 0.004;

            // TODO: Make those clamped values customizable when we have configuration.
            self.zoom_factor = (self.zoom_factor + zoom_delta).clamp(0.1, 100.0);
        }
    }

    // Method to handle panning (dragging)
    pub fn handle_pan_input(&mut self, ctx: &Context, image_response: &Response, info_box_response: Option<&Response>) {
        let mut can_pan = false;

        // "&& self.is_panning" allows for the panning to continue even 
        // when the cursor is on top of another widget (i.e. the info box) 
        // BUT to start panning the cursor has to be directly inside the image.
        if ctx.input(|i| i.pointer.primary_down()) && self.is_panning {
            can_pan = true;
        } else if ctx.input(|i| i.pointer.primary_down()) && image_response.contains_pointer() {
            if let Some(info_box_response) = info_box_response {
                if info_box_response.is_pointer_button_down_on() {
                    can_pan = false;
                } else {
                    can_pan = true;
                }
            } else {
                can_pan = true;
            }
        }

        if can_pan {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                if self.is_panning {
                    let delta = pos - self.drag_start.unwrap();
                    self.pan_offset += delta;
                    self.drag_start = Some(pos);
                } else {
                    self.is_panning = true;
                    self.drag_start = Some(pos);
                }
            }
        } else {
            self.is_panning = false;
        }
    }

    pub fn get_transformation(&self, image_size: Vec2, image_position: Pos2) -> (Vec2, Pos2) {
        let scaled_size = image_size * self.zoom_factor;
        let image_position = image_position - scaled_size * 0.5 + self.pan_offset;

        (scaled_size, image_position)
    }

    pub fn is_pan_out_of_bounds(&mut self, image_size: Vec2) -> bool {
        // TODO: Make this customizable somehow when we add configuration.
        let bounds_to_not_exceed = (image_size / 1.5) * self.zoom_factor / 1.5;

        let pan_offset_x = self.pan_offset.x;
        let pan_offset_y = self.pan_offset.y;

        if pan_offset_x > bounds_to_not_exceed.x || pan_offset_y > bounds_to_not_exceed.y 
        || pan_offset_x < -bounds_to_not_exceed.x || pan_offset_y < -bounds_to_not_exceed.y {
            return true;
        }

        return false;
    }

    pub fn has_been_messed_with(&mut self) -> bool {
        if self.zoom_factor != 1.0 {
            true
        } else if self.pan_offset != Vec2::ZERO {
            true
        } else {
            false
        }
    }
}

impl ZoomPan {
    pub fn schedule_pan_reset(&mut self, delay: Duration) {
        if self.reset_pan_offset.is_none() {

            self.reset_pan_offset = Some(
                ResetManager {
                    timer: Instant::now(),
                    delay: delay,
                    animation_id: rand::thread_rng().gen::<u32>(),
                    in_animation: false
                }
            );

            debug!("Pan offset reset has been scheduled.");
        }
    }

    fn pan_reset_update(&mut self, ctx: &Context) {
        // We don't want to reset the pan offset while the user is still panning.
        if self.is_panning {
            return;
        }

        if let Some(reset_pan_offset) = self.reset_pan_offset.as_mut() {

            if reset_pan_offset.timer.elapsed() >= reset_pan_offset.delay {
                let mut pan_offset_x = 0.0 as f32;
                let mut pan_offset_y = 0.0 as f32;

                if !reset_pan_offset.in_animation {
                    pan_offset_x = self.pan_offset.x;
                    pan_offset_y = self.pan_offset.y;

                    reset_pan_offset.in_animation = true;
                }

                let pan_offset_animated = Vec2::new(
                    egui_animation::animate_eased(
                        ctx,
                        format!("pan_offset_x_{}", reset_pan_offset.animation_id),
                        pan_offset_x,
                        0.5,
                        simple_easing::cubic_in_out
                    ),
                    egui_animation::animate_eased(
                        ctx,
                        format!("pan_offset_y_{}", reset_pan_offset.animation_id),
                        pan_offset_y,
                        0.5,
                        simple_easing::cubic_in_out
                    )
                );

                self.pan_offset = pan_offset_animated;

                if self.pan_offset == Vec2::ZERO {
                    debug!("Pan offset resetting is done.");

                    self.reset_pan_offset = None;
                }
            }
        }
    }

    pub fn schedule_scale_reset(&mut self, delay: Duration) {
        if self.reset_scale_factor.is_none() {

            self.reset_scale_factor = Some(
                ResetManager {
                    timer: Instant::now(),
                    delay: delay,
                    animation_id: rand::thread_rng().gen::<u32>(),
                    in_animation: false
                }
            );

            debug!("Scale factor reset has been scheduled.");
        }
    }

    fn scale_reset_update(&mut self, ctx: &Context) {
        // We don't want to reset the pan offset while the user is still panning.
        if self.is_panning {
            return;
        }

        if let Some(reset_scale_factor) = self.reset_scale_factor.as_mut() {

            if reset_scale_factor.timer.elapsed() >= reset_scale_factor.delay {
                let mut scale_factor = 1.0 as f32;

                if !reset_scale_factor.in_animation {
                    scale_factor = self.zoom_factor;

                    reset_scale_factor.in_animation = true;
                }

                let scale_factor_animated = egui_animation::animate_eased(
                    ctx,
                    format!("scale_factor_{}", reset_scale_factor.animation_id),
                    scale_factor,
                    0.5,
                    simple_easing::cubic_in_out
                );

                self.zoom_factor = scale_factor_animated;

                if self.zoom_factor == 1.0 {
                    debug!("Scale factor resetting is done.");

                    self.reset_scale_factor = None;
                }
            }
        }
    }
}