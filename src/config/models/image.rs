use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Image {
    #[serde(default)]
    pub loading: ImageLoading,
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