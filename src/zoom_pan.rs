use std::time::{Duration, Instant};

use eframe::egui::{Context, Pos2, Response, Vec2};
use log::debug;
use rand::Rng;

/// Struct that controls the zoom and panning of the image.
pub struct ZoomPan {
    zoom_factor: f32,
    last_zoom_factor: f32,
    is_panning: bool,
    pan_offset: Vec2,
    pan_offset_before_reset: Option<Vec2>,
    drag_start: Option<Pos2>,
    reset_pan_offset_timer: Option<Instant>,
    reset_pan_animation_id: u32
}

impl ZoomPan {
    pub fn new() -> Self {
        Self {
            zoom_factor: 1.0,
            last_zoom_factor: 1.0,
            drag_start: None,
            is_panning: false,
            pan_offset: Vec2::ZERO,
            pan_offset_before_reset: None,
            reset_pan_offset_timer: None,
            reset_pan_animation_id: 0
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        // Reset pan offset when scheduled.
        self.pan_reset_update(ctx);
    }

    // Method to handle zoom input (scrolling)
    pub fn handle_zoom_input(&mut self, ctx: &Context) {
        self.last_zoom_factor = self.zoom_factor;

        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);

        if scroll_delta != 0.0 {
            let zoom_delta = scroll_delta * self.zoom_factor * 0.004;

            // TODO: Make those clamped values customizable when we have configuration.
            self.zoom_factor = (self.zoom_factor + zoom_delta).clamp(0.5, 100.0);
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
    fn pan_reset_update(&mut self, ctx: &Context) {
        if let Some(timer) = self.reset_pan_offset_timer {
            if timer.elapsed() >= Duration::from_millis(300) {
                let mut pan_offset_x = 0.0 as f32;
                let mut pan_offset_y = 0.0 as f32;

                if self.pan_offset_before_reset.is_none() {
                    self.pan_offset_before_reset = Some(self.pan_offset);

                    pan_offset_x = self.pan_offset.x;
                    pan_offset_y = self.pan_offset.y;
                }

                let pan_offset_animated = Vec2::new(
                    egui_animation::animate_eased(
                        ctx, 
                        format!("pan_offset_x_{}", self.reset_pan_animation_id), 
                        pan_offset_x, 
                        0.5, 
                        simple_easing::cubic_in_out
                    ),
                    egui_animation::animate_eased(
                        ctx, 
                        format!("pan_offset_y_{}", self.reset_pan_animation_id), 
                        pan_offset_y, 
                        0.5, 
                        simple_easing::cubic_in_out
                    )
                );

                self.pan_offset = pan_offset_animated;

                if self.pan_offset == Vec2::ZERO {
                    debug!("Pan offset resetting is done.");
                    self.reset_pan_offset_timer = None;
                    self.pan_offset_before_reset = None;
                }
            }
        }
    }

    pub fn schedule_pan_reset(&mut self) {
        if self.reset_pan_offset_timer.is_none() {
            self.reset_pan_offset_timer = Some(Instant::now());
            self.reset_pan_animation_id = rand::thread_rng().gen::<u32>();
            debug!("Pan offset reset has been scheduled.");
        }
    }
}