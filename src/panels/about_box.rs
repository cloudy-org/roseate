use egui_notify::ToastLevel;
use eframe::egui::{self, pos2, ImageSource, Key, RichText, Vec2};

use crate::{
    config::config::Config, 
    notifier::NotifierAPI
};

pub struct AboutBox {
    pub show: bool,
    toggle_key: Key,
}

impl AboutBox {
    pub fn new(config: &Config, notifier: &mut NotifierAPI) -> Self {
        let config_key = match Key::from_name(&config.key_binds.about_box.toggle) {
            Some(key) => key,
            None => {
                notifier.toasts.lock().unwrap().toast_and_log(
                    "The key bind set for 'about.toggle' is invalid! Defaulting to `A`.".into(), 
                    ToastLevel::Error
                );

                Key::A
            },
        };

        Self {
            show: false,
            toggle_key: config_key,
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(self.toggle_key)) {
            self.show = !self.show;
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }

        egui::Window::new(
            egui::WidgetText::RichText(
                egui::RichText::new("About roseate").size(15.0)
            )
        )
            .default_pos(pos2(200.0, 200.0))
            .resizable(false)
            .fade_in(false)
            .fade_out(false)
            .show(ctx, |ui| {
                egui::Grid::new("about_box_grid")
                .num_columns(3)
                .spacing([20.0, 10.0])
                .show(ui, |ui| {
                    let version = env!("CARGO_PKG_VERSION");
                    ui.vertical_centered_justified(|ui| {
                        ui.add(
                            egui::Image::new(get_platform_rose_image())
                            .fit_to_exact_size(
                                Vec2::new(100.0, 100.0)
                            )
                        );

                        ui.label(RichText::new("Roseate")
                            .strong()
                            .heading()
                        );
                    });
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.label(format!("Version: {}", version));

                        ui.label("|");

                        ui.hyperlink_to(
                            RichText::new("What's new").underline(), 
                            format!("https://github.com/cloudy-org/roseate/releases/tag/{}", version)
                        );
                    });
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.hyperlink_to(
                            RichText::new("Repository").underline(), 
                            "https://github.com/cloudy-org/roseate"
                        );

                        ui.label("|");

                        ui.hyperlink_to(
                            RichText::new("License").underline(), 
                            "https://github.com/cloudy-org/roseate/blob/main/LICENSE"
                        );
                    });
                    ui.end_row();
                });
            }
        );
        
    }
}

fn get_platform_rose_image<'a>() -> ImageSource<'a> {
    if cfg!(target_os = "windows") {
        return egui::include_image!("../../assets/rose_emojis/microsoft.png");
    } else if cfg!(target_os = "macos") {
        return egui::include_image!("../../assets/rose_emojis/apple.png");
    }

    return egui::include_image!("../../assets/rose_emojis/google_noto.png");
}