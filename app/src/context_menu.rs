use egui::{Context, Id, LayerId, Popup, PopupAnchor, PopupCloseBehavior, PopupKind, Pos2, Ui};

use crate::windows::WindowsManager;

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

    pub fn show(&mut self, ui: &mut Ui, windows_manager: &mut WindowsManager) {
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
                .style(egui::containers::menu::menu_style)
                // doesn't work, just trying to disable "CloseOnClick"
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .show(|pop_ui| {
                    if pop_ui.button("Toggle Image Info").clicked() {
                        windows_manager.show_info = !windows_manager.show_info;
                        self.show_menu = None;
                    }

                    if pop_ui.button("Toggle extra Image Info").clicked() {
                        windows_manager.show_info = !windows_manager.show_info;

                        windows_manager.show_extra_info = true;
                        self.show_menu = None;
                    }
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
