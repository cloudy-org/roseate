use cirrus_egui::v1::notifier::Notifier;
use egui::{Context, InputState, Key, Ui};
use egui_notify::ToastLevel;

use crate::{ui_controls::magnification_panel::MagnificationPanel, utils::get_input_reader_from_soft_binds, viewport::Viewport};

mod magnification_panel;

pub struct UIControlsManager {
    mag_panel: MagnificationPanel,

    show_controls_reader: Option<Box<dyn FnMut(&InputState) -> bool>>,

    show_controls: Option<bool>,
}

impl UIControlsManager {
    pub fn new() -> Self {
        let mag_panel = MagnificationPanel::new();

        Self {
            mag_panel,

            show_controls_reader: None,

            show_controls: None
        }
    }

    pub fn handle_input(
        &mut self,
        ctx: &Context,
        notifier: &mut Notifier,
        show_controls_key: &String,
        hide_by_default: bool
    ) {
        let show_controls = self.show_controls.get_or_insert(!hide_by_default);

        let show_controls_reader = self.show_controls_reader.get_or_insert_with(|| {
            match get_input_reader_from_soft_binds(
                show_controls_key,
                |i, key| i.key_pressed(key)
            ) {
                Ok(reader) => Box::new(reader),
                Err(error) => {
                    notifier.toast(
                        Box::new(error), ToastLevel::Error, |_| {}
                    );

                    Box::new(|i| i.key_pressed(Key::C))
                },
            }
        });

        if ctx.input(show_controls_reader) {
            *show_controls ^= true;
        }
    }

    pub fn show(&mut self, ui: &mut Ui, viewport: &mut Viewport) {
        if self.show_controls.unwrap_or(false) {
            self.mag_panel.show(ui, viewport);
        }
    }
}