use std::{time::Duration};

use cirrus_egui::v1::{notifier::Notifier, ui_utils::center_multi::ui_multiple_centered_double_render};
use cirrus_theming::v1::Colour;
use egui::{Button, Color32, CursorIcon, RichText, Sense, Stroke, Ui, Vec2};
use egui_notify::ToastLevel;

use crate::{files::get_rose_image, image::backends::ImageProcessingBackend, image_handler::{ImageHandler, optimization::ImageOptimizations}, monitor_size::MonitorSize};

pub struct ImageSelectionMenu {}

impl ImageSelectionMenu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        image_handler: &mut ImageHandler,
        configured_image_optimizations: Vec<ImageOptimizations>,
        notifier: &mut Notifier,
        monitor_size: &MonitorSize,
        image_processing_backend: ImageProcessingBackend,
        accent_colour: &Colour,
        show_open_image_button: bool
    ) {
        let (rose_or_button_response, rose_rect) = ui_multiple_centered_double_render(ui, |ui| {
            if image_handler.image.is_some() {
                ui.disable();
            }

            let mut rose_response = ui.add(
                egui::Image::new(get_rose_image())
                    .max_width(145.0)
                    .sense(Sense::click())
            );

            let rose_rect = rose_response.rect;

            if show_open_image_button {
                ui.add_space(8.0);

                rose_response = rose_response.union(
                    ui.add(
                        Button::new(
                            RichText::new("Open Image")
                                .size(19.0)
                        ).min_size(Vec2::new(135.0, 35.0))
                        .corner_radius(14.0)
                    )
                );
            }

            (
                rose_response.on_hover_cursor(CursorIcon::PointingHand),
                rose_rect
            )
        }).inner;

        if rose_or_button_response.clicked() {
            let result = image_handler.select_image(
                configured_image_optimizations
            );

            match result {
                Ok(_) => {
                    image_handler.load_image(
                        true,
                        notifier,
                        monitor_size,
                        image_processing_backend
                    );
                },
                Err(error) => {
                    notifier.toast(
                        Box::new(error),
                        ToastLevel::Error,
                        |toast| {
                            toast.duration(Duration::from_secs(5));
                        }
                    );
                },
            }
        }

        // TODO: drag and drop now needs re-testing.
        let file_is_hovering = !ui.ctx().input(|i| i.raw.hovered_files.is_empty());

        if file_is_hovering {
            ui.label("You're about to drop a file...");

            let rect = rose_rect.expand2(
                Vec2::new(150.0, 100.0)
            );
            let painter = ui.painter();

            // Draw dotted lines to indicate file being dropped.
            for index in 0..4 {
                let pos = match index {
                    0 => &[rect.left_top(), rect.right_top()],
                    1 => &[rect.right_top(), rect.right_bottom()],
                    2 => &[rect.right_bottom(), rect.left_bottom()],
                    3 => &[rect.left_bottom(), rect.left_top()],
                    _ => unreachable!()
                };

                painter.add(
                    egui::Shape::dashed_line(
                        pos,
                        Stroke {
                            width: 2.0,
                            color: Color32::from_hex(
                                &accent_colour.hex_code
                            ).unwrap()
                        },
                        11.0,
                        10.0
                    )
                );
            }
        }
    }
}