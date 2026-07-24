use eframe::egui::{self, Align, Context, CornerRadius, FontId, Id, LayerId, Layout, Popup, PopupAnchor, PopupCloseBehavior, PopupKind, Pos2, Style, Ui};

use crate::{ui_controls::UIControlsManager, windows::WindowsManager};

pub struct ContextMenu {
    show_menu: Option<Pos2>
}

impl ContextMenu {
    pub fn new() -> Self {
        Self {
            show_menu: None,
        }
    }

    pub fn handle_input(&mut self, ctx: &Context, windows_manager: &WindowsManager) {
        if ctx.input(|i| i.pointer.secondary_released()) {
            if let Some(mouse_position) = ctx.pointer_latest_pos() {
                // I want to follow gnome's behaviour of 
                // another right-click hides the context menu.
                if self.show_menu.is_some() {
                    self.show_menu = None;
                    return;
                }

                // content menu should not display in windows.
                if !windows_manager.rect.contains(mouse_position) {
                    self.show_menu = Some(mouse_position);
                }
            }
        }
    }

    pub fn show(&mut self, ui: &mut Ui, windows_manager: &mut WindowsManager, ui_controls_manager: &mut UIControlsManager) {
        if let Some(mouse_position) = self.show_menu {
            let id = Id::new("context_menu");

            // NOTE: for some reason Popup::content_menu or Popup::menu does not work 
            // so most of the code below here are to recreate their behaviours and looks.
            let response = Popup::new(
                id,
                ui.ctx().clone(),
                PopupAnchor::Position(mouse_position),
                LayerId::new(egui::Order::Foreground, id)
            ).kind(PopupKind::Menu)
                .style(|style: &mut Style| {
                    egui::containers::menu::menu_style(style);

                    // Don't want to use monospace in the context menu.
                    // 
                    // I might even completely move away from monospace text globally
                    // and just switch to it for special design reasons going forward.
                    style.override_font_id = Some(FontId::default());

                    let widgets = &mut style.visuals.widgets;

                    widgets.inactive.corner_radius = CornerRadius::same(3);
                    widgets.active.corner_radius = CornerRadius::same(3);
                    widgets.hovered.corner_radius = CornerRadius::same(3);
                    widgets.noninteractive.corner_radius = CornerRadius::same(3);
                    widgets.open.corner_radius = CornerRadius::same(3);
                })
                // doesn't work, just trying to disable "CloseOnClick"
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        // TODO: Implement "Open With" to allow opening image in another application.
                        // TODO: Implement "Copy" button to allow for coping the image to your clipboard for pasting elsewhere.

                        // ui.button("Open With...");
                        // ui.button("Copy")

                        // ui.separator();

                        ui.menu_button("Show Info", |ui| {
                            if ui.button("Toggle Info Window").clicked() {
                                windows_manager.show_info = !windows_manager.show_info;
                                windows_manager.show_extra_info = false;

                                self.show_menu = None;
                            }

                            if ui.button("Toggle Info Window (Extra)").clicked() {
                                match windows_manager.show_info {
                                    true => {
                                        windows_manager.show_info = false;
                                    },
                                    false => {
                                        windows_manager.show_extra_info = true;
                                        windows_manager.show_info = true;
                                    },
                                }

                                self.show_menu = None;
                            }
                        });

                        if ui.button("Toggle Controls").clicked() {
                            ui_controls_manager.show_controls = match ui_controls_manager.show_controls {
                                Some(show) => Some(!show),
                                None => Some(true),
                            };

                            self.show_menu = None;
                        }
                    });
                }).unwrap().response;

            // We wouldn't have to do this if "Popup::content_menu" or 
            // ".close_behavior(PopupCloseBehavior::CloseOnClickOutside)" just worked.
            if ui.input(|i| i.pointer.primary_clicked()) {
                if let Some(current_mouse_position) = ui.ctx().pointer_latest_pos() {
                    if !response.rect.contains(current_mouse_position) {
                        self.show_menu = None;
                    }
                }
            }
        }
    }
}
