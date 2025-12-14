use std::hash::Hash;
use serde::{Deserialize, Deserializer, Serialize};
use crate::{image_handler::optimization::ImageOptimizations as ImageOptimizationsEnum};

#[derive(Serialize, Deserialize, Hash)]
pub struct ImageOptimizations {
    #[serde(default = "super::none_default")]
    pub mode: Option<String>,
    #[serde(default, deserialize_with = "deserialize_image_optimization_field_value")]
    pub monitor_downsampling: MonitorDownsampling,
    #[serde(default, deserialize_with = "deserialize_image_optimization_field_value")]
    pub experimental_dynamic_sampling: DynamicSampling
}

impl Default for ImageOptimizations {
    fn default() -> Self {
        Self {
            mode: None,
            monitor_downsampling: MonitorDownsampling::default(),
            experimental_dynamic_sampling: DynamicSampling::default()
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
                    ],
                }
            },
            None => {
                let mut optimizations = Vec::new();

                // TODO: we'll need something to replace these if statements
                // so when we scale it won't turn into yandere dev's codebase
                // for yandere simulator.

                if self.monitor_downsampling.enabled {
                    optimizations.push(
                        ImageOptimizationsEnum::MonitorDownsampling(
                            (self.monitor_downsampling.strength * 100.0) as u32
                        )
                    );
                }

                if self.experimental_dynamic_sampling.enabled {
                    optimizations.push(
                        ImageOptimizationsEnum::DynamicSampling(
                            true,
                            self.experimental_dynamic_sampling.also_downsample
                        )
                    );
                }

                optimizations
            }
        };

        optimizations_enums
    }
}


#[derive(Serialize, Deserialize)]
pub struct MonitorDownsampling {
    #[serde(default = "super::true_default")]
    pub enabled: bool,
    #[serde(default = "monitor_downsampling_strength_default")]
    pub strength: f32
}

impl Default for MonitorDownsampling {
    fn default() -> Self {
        Self::default_with_enabled(true)
    }
}

impl DefaultWithEnabled for MonitorDownsampling {
    fn default_with_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            strength: monitor_downsampling_strength_default()
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

fn monitor_downsampling_strength_default() -> f32 {
    1.3
}


#[derive(Serialize, Deserialize, Hash)]
pub struct DynamicSampling {
    #[serde(default = "super::false_default")]
    pub enabled: bool,
    #[serde(default = "super::true_default")]
    pub also_downsample: bool
}

impl Default for DynamicSampling {
    fn default() -> Self {
        Self::default_with_enabled(false)
    }
}

impl DefaultWithEnabled for DynamicSampling {
    fn default_with_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            also_downsample: true
        }
    }
}


trait DefaultWithEnabled {
    fn default_with_enabled(enabled: bool) -> Self;
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ImageOptimizationFieldValue<T> {
    Bool(bool),
    Struct(T),
}

fn deserialize_image_optimization_field_value<'de, D, T>(
    deserializer: D,
) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + DefaultWithEnabled
{
    match ImageOptimizationFieldValue::<T>::deserialize(deserializer)? {
        ImageOptimizationFieldValue::Struct(_struct) => Ok(_struct),
        ImageOptimizationFieldValue::Bool(enabled) => Ok(T::default_with_enabled(enabled)),
    }
}
