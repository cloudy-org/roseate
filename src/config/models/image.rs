use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Image {
    #[serde(default)]
    pub loading: ImageLoading,

    #[serde(default = "image_marginal_allowance")]
    pub marginal_allowance: f32
}

impl Default for Image {
    fn default() -> Self {
        Self {
            loading: ImageLoading::default(),
            marginal_allowance: image_marginal_allowance()
        }
    }
}

fn image_marginal_allowance() -> f32 {
    1.3
}


#[derive(Serialize, Deserialize, Default)]
pub struct ImageLoading {
    #[serde(default)]
    pub gui: LoadingGUISettings,
    #[serde(default)]
    pub initial: InitialSettings,
    #[serde(default)]
    pub dynamic: DynamicSettings,
}


#[derive(Serialize, Deserialize)]
pub struct LoadingGUISettings {
    #[serde(default = "super::true_default")]
    pub lazy_loading: bool,
}

impl Default for LoadingGUISettings {
    fn default() -> Self {
        Self {
            lazy_loading: true,
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct InitialSettings {
    #[serde(default = "super::false_default")]
    pub lazy_loading: bool,
}

impl Default for InitialSettings {
    fn default() -> Self {
        Self {
            lazy_loading: false
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct DynamicSettings {
    #[serde(default = "super::true_default")]
    pub lazy_loading: bool,
}

impl Default for DynamicSettings {
    fn default() -> Self {
        Self {
            lazy_loading: true
        }
    }
}