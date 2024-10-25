use eframe::egui::{self, Pos2, Response, Vec2};

/// Struct that controls the zoom and panning of the image.
pub struct ZoomPan {
    zoom_factor: f32,
    last_zoom_factor: f32,
    is_panning: bool,
    pan_offset: Vec2,
    drag_start: Option<Pos2>,
}

impl ZoomPan {
    pub fn new() -> Self {
        Self {
            zoom_factor: 1.0,
            last_zoom_factor: 1.0,
            drag_start: None,
            is_panning: false,
            pan_offset: Vec2::ZERO,
        }
    }

    // Method to handle zoom input (scrolling)
    pub fn handle_zoom_input(&mut self, ctx: &egui::Context) {
        self.last_zoom_factor = self.zoom_factor;

        let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);

        if scroll_delta != 0.0 {
            let zoom_delta = scroll_delta * self.zoom_factor * 0.004;

            self.zoom_factor = (self.zoom_factor + zoom_delta).clamp(0.5, 100.0);
        }
    }

    // Method to handle panning (dragging)
    pub fn handle_pan_input(&mut self, ctx: &egui::Context, image_response: &Response, info_box_response: Option<&Response>) {
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

    pub fn get_transformation(&self, image_size: egui::Vec2, image_position: egui::Pos2) -> (Vec2, Pos2) {
        let scaled_size = image_size * self.zoom_factor;
        let image_position = image_position - scaled_size * 0.5 + self.pan_offset;

        (scaled_size, image_position)
    }

    pub fn has_been_messed_with(&mut self) -> bool {
        if self.zoom_factor != 1.0 {
            true
        } else if self.pan_offset != egui::Vec2::ZERO {
            true
        } else {
            false
        }
    }
}