use serde::{Deserialize, Deserializer, Serialize};
use crate::image_handler::optimization::ImageOptimizations as ImageOptimizationsEnum;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Default, Hash)]
pub struct Image {
    #[serde(default)]
    pub loading: ImageLoading,
    #[serde(default)]
    pub optimizations: ImageOptimizations
}


#[derive(Serialize, Deserialize, Default, Hash)]
pub struct ImageLoading {
    #[serde(default)]
    pub gui: LoadingGUISettings,
    #[serde(default)]
    pub initial: InitialSettings,
}


#[derive(Serialize, Deserialize, Hash)]
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


#[derive(Serialize, Deserialize, Hash)]
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


#[derive(Serialize, Deserialize, Hash)]
pub struct ImageOptimizations {
    #[serde(default = "super::none_default")]
    mode: Option<String>,
    #[serde(default, deserialize_with = "deserialize_monitor_downsampling")]
    monitor_downsampling: MonitorDownsampling
}

impl Default for ImageOptimizations {
    fn default() -> Self {
        Self {
            mode: None,
            monitor_downsampling: MonitorDownsampling::default()
        }
    }
}

impl ImageOptimizations {
    /// Returns the optimizations the user has configured in config.toml.
    pub fn get_optimizations(&self) -> Vec<ImageOptimizationsEnum> {
        let optimizations_enums: Vec<ImageOptimizationsEnum> = match &self.mode {
            Some(mode) => {
                match mode.to_lowercase().as_str() {
                    "s" | "speed" => Vec::new(),
                    "d" | "default" | &_ => vec![
                        ImageOptimizationsEnum::MonitorDownsampling(
                            (MonitorDownsampling::default().strength * 100.0) as u32
                        ),
                        // TODO: when dynamic sampling is ready to move away from misc.experimental add it here.
                    ],
                }
            },
            None => {
                let mut optimizations = Vec::new();

                if self.monitor_downsampling.enabled {
                    optimizations.push(
                        ImageOptimizationsEnum::MonitorDownsampling(
                            (self.monitor_downsampling.strength * 100.0) as u32
                        )
                    );
                }

                // TODO: when dynamic sampling is ready to move away from misc.experimental add it here.

                optimizations
            }
        };

        optimizations_enums
    }
}


#[derive(Serialize, Deserialize)]
pub struct MonitorDownsampling {
    #[serde(default = "super::true_default")]
    enabled: bool,
    #[serde(default = "monitor_downsampling_strength")]
    strength: f32
}

impl Default for MonitorDownsampling {
    fn default() -> Self {
        Self {
            enabled: true,
            strength: 1.3
        }
    }
}

impl Hash for MonitorDownsampling {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.enabled.hash(state);

        // rust does not support hashing floats, there is 
        // a work around via the crate "ordered-float" but 
        // that's over engineered for my needs here.
        // 
        // I just need to show change to the hasher.
        let strength_as_u32 = (self.strength * 100.0) as u32;
        strength_as_u32.hash(state);
    }
}

fn monitor_downsampling_strength() -> f32 {
    1.3
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ImageOptimizationFieldValue<T> {
    Bool(bool),
    Struct(T),
}

fn deserialize_monitor_downsampling<'de, D>(
    deserializer: D,
) -> Result<MonitorDownsampling, D::Error>
where
    D: Deserializer<'de>,
{
    match ImageOptimizationFieldValue::<MonitorDownsampling>::deserialize(deserializer)? {
        ImageOptimizationFieldValue::Bool(enabled) => Ok(
            MonitorDownsampling {
                enabled,
                ..Default::default()
            }
        ),
        ImageOptimizationFieldValue::Struct(_struct) => Ok(_struct),
    }
}
