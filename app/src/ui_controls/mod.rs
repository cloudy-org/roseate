use egui::{Context, Key, Ui};

use crate::{ui_controls::magnification_panel::MagnificationPanel, viewport::Viewport};

mod magnification_panel;

pub struct UIControlsManager {
    mag_panel: MagnificationPanel,

    show_controls: bool,
}

impl UIControlsManager {
    pub fn new() -> Self {
        let mag_panel = MagnificationPanel::new();

        Self {
            mag_panel,

            show_controls: false
        }
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.input(|input| input.key_pressed(Key::C)) {
            self.show_controls = !self.show_controls;
        }
    }

    pub fn show(&mut self, ui: &mut Ui, viewport: &mut Viewport) {
        if self.show_controls {
            self.mag_panel.show(ui, viewport);
        }
    }
}