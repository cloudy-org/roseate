use egui::{CursorIcon, Rect, Sense, Ui, Vec2};

use crate::image::image::Image;

pub struct Viewport {
    pub zoom: f32,
    offset: Vec2
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO
        }
    }

    pub fn show(&mut self, ui: &mut Ui, image: &Image, egui_image: egui::Image, padding: f32, zoom_into_cursor: bool) {
        let window_size = ui.input(|i: &egui::InputState| i.viewport_rect()).size();

        let (available_rect, response) = ui.allocate_exact_size(
            ui.available_size(),
            Sense::click_and_drag()
        );

        let image_size = Vec2::new(
            image.image_size.0 as f32, image.image_size.1 as f32
        );

        // The image size relative to viewport padding and zoom factor.
        let image_size_with_padding = image_size * padding;
        let relative_image_size = (image_size_with_padding) * self.zoom;

        // we need the image size with padding alone to calculate 
        // what the image scale (or zoom if you want to call it that) 
        // would be when scaled up to window size.
        let some_scale = window_size / image_size_with_padding;

        println!("--> {}", some_scale);

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

            // ui.ctx().request_repaint();
        }

        let egui_image = egui_image
            .corner_radius(10.0);

        // Drawing the image to the viewport.
        egui_image.paint_at(ui, image_rect);
    }
}