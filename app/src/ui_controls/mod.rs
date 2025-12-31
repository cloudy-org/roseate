use egui::{Context, Key, Ui};

use crate::{ui_controls::magnification_panel::MagnificationPanel, viewport::Viewport};

mod magnification_panel;

pub struct UIControlsManager {
    mag_panel: MagnificationPanel,

    show_controls: Option<bool>,
}

impl UIControlsManager {
    pub fn new() -> Self {
        let mag_panel = MagnificationPanel::new();

        Self {
            mag_panel,

            show_controls: None
        }
    }

    pub fn handle_input(&mut self, ctx: &Context, hide_by_default: bool) {
        let show_controls = self.show_controls.get_or_insert(!hide_by_default);

        // TODO: key bind
        if ctx.input(|input| input.key_pressed(Key::C)) {
            *show_controls ^= true;
        }
    }

    pub fn show(&mut self, ui: &mut Ui, viewport: &mut Viewport) {
        if self.show_controls.unwrap_or(false) {
            self.mag_panel.show(ui, viewport);
        }
    }
}