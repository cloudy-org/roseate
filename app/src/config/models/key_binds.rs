use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct KeyBinds {
    #[serde(default = "show_image_info")]
    pub show_image_info: String,
    #[serde(default = "show_extra_image_info")]
    pub show_extra_image_info: String,
    #[serde(default = "show_app_about")]
    pub show_app_about: String,
    #[serde(default = "reset_viewport")]
    pub reset_viewport: String,
    #[serde(default = "show_ui_controls")]
    pub show_ui_controls: String
}

fn show_image_info() -> String { "I".into() }
fn show_extra_image_info() -> String { "CTRL+I".into() }
fn show_app_about() -> String { "CTRL+A".into() }
fn reset_viewport() -> String { "R".into() }
fn show_ui_controls() -> String { "C".into() }

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            show_image_info: show_image_info(),
            show_extra_image_info: show_extra_image_info(),
            show_app_about: show_app_about(),
            reset_viewport: reset_viewport(),
            show_ui_controls: show_ui_controls()
        }
    }
}