use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct LoadingGUISettings {
    #[serde(default = "true_default")]
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct InitialSettings {
    #[serde(default = "false_default")]
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct DynamicSettings {
    #[serde(default = "true_default")]
    pub lazy_loading: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ImageLoading {
    #[serde(default)]
    pub gui: LoadingGUISettings,
    #[serde(default)]
    pub initial: InitialSettings,
    #[serde(default)]
    pub dynamic: DynamicSettings,
}

#[derive(Serialize, Deserialize)]
pub struct Image {
    #[serde(default)]
    pub loading: ImageLoading,

    #[serde(default = "image_marginal_allowance")]
    pub marginal_allowance: f32
}

#[derive(Serialize, Deserialize)]
pub struct UISettings {
    pub magnification_panel: MagnificationPanel,
    pub window_scaling: WindowScaling
}

#[derive(Serialize, Deserialize)]
pub struct MagnificationPanel {
    #[serde(default = "false_default")]
    pub enabled_default: bool,
}

#[derive(Serialize, Deserialize)]
pub struct WindowScaling {
    #[serde(default = "ui_padding")]
    pub padding: f32,
}

#[derive(Serialize, Deserialize)]
pub struct InfoBoxBinds {
    #[serde(default = "info_box_toggle")]
    pub toggle: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImageBinds {
    #[serde(default = "image_reset_pos")]
    pub reset_pos: String,
}

#[derive(Serialize, Deserialize)]
pub struct UIControlsBinds {
    #[serde(default = "ui_controls_toggle")]
    pub toggle: String,
}

#[derive(Serialize, Deserialize)]
pub struct Keybinds {
    #[serde(default)]
    pub info_box: InfoBoxBinds,
    #[serde(default)]
    pub image: ImageBinds,
    #[serde(default)]
    pub ui_controls: UIControlsBinds
}

impl Default for LoadingGUISettings {
    fn default() -> Self {
        Self {
            lazy_loading: true,
        }
    }
}

impl Default for InitialSettings {
    fn default() -> Self {
        Self {
            lazy_loading: false
        }
    }
}

impl Default for DynamicSettings {
    fn default() -> Self {
        Self {
            lazy_loading: true
        }
    }
}

impl Default for ImageLoading {
    fn default() -> Self {
        Self {
            gui: LoadingGUISettings::default(),
            initial: InitialSettings::default(),
            dynamic: DynamicSettings::default()
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            loading: ImageLoading::default(),
            marginal_allowance: image_marginal_allowance()
        }
    }
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            magnification_panel: MagnificationPanel::default(),
            window_scaling: WindowScaling::default()
        }
    }
}


impl Default for MagnificationPanel {
    fn default() -> Self {
        Self {
            enabled_default: false
        }
    }
}

impl Default for WindowScaling {
    fn default() -> Self {
        Self {
            padding: ui_padding()
        }
    }
}

impl Default for InfoBoxBinds {
    fn default() -> Self {
        Self {
            toggle: info_box_toggle()
        }
    }
}

impl Default for ImageBinds {
    fn default() -> Self {
        Self {
            reset_pos: image_reset_pos()
        }
    }
}

impl Default for UIControlsBinds {
    fn default() -> Self {
        Self {
            toggle: ui_controls_toggle()
        }
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            info_box: InfoBoxBinds::default(),
            image: ImageBinds::default(),
            ui_controls: UIControlsBinds::default()
        }
    }
}

fn true_default() -> bool {
    true
}

fn false_default() -> bool {
    true
}

fn info_box_toggle() -> String {
    "I".to_string()
}

fn image_reset_pos() -> String {
    "R".to_string()
}

fn ui_controls_toggle() -> String {
    "C".to_string()
}

fn image_marginal_allowance() -> f32 {
    1.3
}

fn ui_padding() -> f32 {
    0.98
}

// This file is so unreadable :sob: ~ Ananas