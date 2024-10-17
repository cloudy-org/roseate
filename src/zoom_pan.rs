use eframe::egui::{self, Pos2, Vec2};

/// Struct that controls the zoom and panning of the image.
pub struct ZoomPan {
    zoom_factor: f32,
    is_panning: bool,
    pan_offset: egui::Vec2,
    drag_start: Option<egui::Pos2>,
}

impl ZoomPan {
    pub fn new() -> Self {
        Self {
            zoom_factor: 1.0,
            drag_start: None,
            is_panning: false,
            pan_offset: egui::Vec2::ZERO,
        }
    }

    // Method to handle zoom input (scrolling)
    pub fn handle_zoom(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.smooth_scroll_delta.y) != 0.0 {
            let zoom_delta = ctx.input(|i| i.smooth_scroll_delta.y) * self.zoom_factor * 0.004;
            self.zoom_factor = (self.zoom_factor + zoom_delta).clamp(0.5, 100.0);
        }
    }

    // Method to handle panning (dragging)
    pub fn handle_pan(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.pointer.primary_down()) {
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
}