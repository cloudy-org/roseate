use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct Controls {
    #[serde(default = "super::false_default")]
    pub show: bool,

    #[serde(default = "super::true_default")]
    pub magnification: bool,
    #[serde(default = "super::true_default")]
    pub fullscreen: bool,
    #[serde(default = "super::true_default")]
    pub settings: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            show: false,

            magnification: true,
            fullscreen: true,
            settings: true,
        }
    }
}

// pub struct Magnification {
//     pub enabled: bool
// }

// impl Default for Magnification {
//     fn default() -> Self {
//         Self {
//             enabled: true
//         }
//     }
// }
