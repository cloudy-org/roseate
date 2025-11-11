use egui::{Pos2, Vec2};


pub struct Viewport {
    zoom: f32,
    offset: Vec2,
    last_drag: Option<Pos2>
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,
            last_drag: None
        }
    }

    pub fn show(&mut self, egui_image: egui::Image) {}
}