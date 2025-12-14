use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct KeyBinds {
    #[serde(default)]
    pub info_box: InfoBoxBinds,
    #[serde(default)]
    pub about_box: AboutBoxBinds,
    #[serde(default)]
    pub image: ImageBinds,
    #[serde(default)]
    pub ui_controls: UIControlsBinds
}


#[derive(Serialize, Deserialize, Hash)]
pub struct InfoBoxBinds {
    #[serde(default = "info_box_toggle")]
    pub toggle: String,
}

impl Default for InfoBoxBinds {
    fn default() -> Self {
        Self {
            toggle: info_box_toggle()
        }
    }
}

fn info_box_toggle() -> String {
    "I".to_string()
}


#[derive(Serialize, Deserialize, Hash)]
pub struct AboutBoxBinds {
    #[serde(default = "about_box_toggle")]
    pub toggle: String,
}

impl Default for AboutBoxBinds {
    fn default() -> Self {
        Self {
            toggle: about_box_toggle()
        }
    }
}

fn about_box_toggle() -> String {
    "A".to_string()
}


#[derive(Serialize, Deserialize, Hash)]
pub struct ImageBinds {
    #[serde(default = "image_reset_pos")]
    pub reset_pos: String,
}

impl Default for ImageBinds {
    fn default() -> Self {
        Self {
            reset_pos: image_reset_pos()
        }
    }
}

fn image_reset_pos() -> String {
    "R".to_string()
}


#[derive(Serialize, Deserialize, Hash)]
pub struct UIControlsBinds {
    #[serde(default = "ui_controls_toggle")]
    pub toggle: String,
}

impl Default for UIControlsBinds {
    fn default() -> Self {
        Self {
            toggle: ui_controls_toggle()
        }
    }
}

fn ui_controls_toggle() -> String {
    "C".to_string()
}
