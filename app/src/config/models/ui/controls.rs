use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Controls {
    pub show: bool,

    pub magnification: bool,
    pub fullscreen: bool,
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
