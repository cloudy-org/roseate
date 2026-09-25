use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct KeyBinds {
    pub show_image_info: String,
    pub show_extra_image_info: String,
    pub reset_viewport: String,
    pub show_ui_controls: String,
    pub open_image: String,
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            show_image_info: String::from("I"),
            show_extra_image_info: String::from("CTRL+I"),
            reset_viewport: String::from("R"),
            show_ui_controls: String::from("C"),
            open_image: String::from("CTRL+O"),
        }
    }
}