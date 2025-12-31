use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Clone)]
pub struct Controls {
    #[serde(default = "super::true_default")]
    pub hide: bool,

    #[serde(default = "super::true_default")]
    pub magnification: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            hide: true,
            magnification: true
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
