use std::{hash::Hash, thread::available_parallelism};
use log::warn;
use serde::{Deserialize, Deserializer, Serialize};
use crate::{image_handler::optimization};

#[derive(Serialize, Deserialize, Hash)]
pub struct ImageOptimizations {
    #[serde(default = "super::none_default")]
    pub mode: Option<String>,
    #[serde(default, deserialize_with = "deserialize_image_optimization_field_value")]
    pub monitor_downsampling: MonitorDownsampling,
    #[serde(default, deserialize_with = "deserialize_image_optimization_field_value")]
    pub free_memory_after_gpu_upload: FreeMemoryAfterGPUUpload,
    #[serde(default, deserialize_with = "deserialize_image_optimization_field_value")]
    pub experimental_dynamic_sampling: DynamicSampling,
    #[serde(default, deserialize_with = "deserialize_image_optimization_field_value")]
    pub experimental_multi_threaded_sampling: MultiThreadedSampling,
}

impl Default for ImageOptimizations {
    fn default() -> Self {
        Self {
            mode: None,
            monitor_downsampling: MonitorDownsampling::default(),
            free_memory_after_gpu_upload: FreeMemoryAfterGPUUpload::default(),
            experimental_dynamic_sampling: DynamicSampling::default(),
            experimental_multi_threaded_sampling: MultiThreadedSampling::default(),
        }
    }
}

impl ImageOptimizations {
    /// Returns the optimizations the user has configured in config.toml.
    pub fn get_optimizations(&self) -> optimization::ImageOptimizations {
        match &self.mode {
            Some(mode) => {
                match mode.to_lowercase().as_str() {
                    "s" | "speed" => optimization::ImageOptimizations::default(), // by default it's none for now
                    "d" | "default" | &_ => {
                        optimization::ImageOptimizations {
                            monitor_downsampling: Some(
                                optimization::MonitorDownsampling::default()
                            ),
                            ..Default::default() // everything else None
                        }
                    }
                }
            },
            None => {
                // TODO: figure out a way to avoid having duplicates of optimization 
                // structs (e.g: "optimization::MonitorDownsampling" and "MonitorDownsampling")
                optimization::ImageOptimizations {
                    monitor_downsampling: match self.monitor_downsampling.enabled {
                        true => Some(
                            optimization::MonitorDownsampling {
                                marginal_allowance: self.monitor_downsampling.strength
                            }
                        ),
                        false => None,
                    },
                    free_memory_after_gpu_upload: self.free_memory_after_gpu_upload.enabled,
                    dynamic_sampling: match self.experimental_dynamic_sampling.enabled {
                        true => Some(
                            optimization::DynamicSampling {
                                up: true, down: self.experimental_dynamic_sampling.also_downsample
                            }
                        ),
                        false => None,
                    },
                    multi_threaded_sampling: match self.experimental_multi_threaded_sampling.enabled {
                        true => Some(
                            optimization::MultiThreadedSampling {
                                number_of_threads: self.experimental_multi_threaded_sampling.threads
                            }
                        ),
                        false => None,
                    }
                }
            }
        }
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
    optimization::MonitorDownsampling::default().marginal_allowance
    // (default: 1.4) if our monitor is 1920x1080, this strength will 
    // allow images up to 2688x1512 until it decides to downsample
}


#[derive(Serialize, Deserialize, Hash)]
pub struct FreeMemoryAfterGPUUpload {
    #[serde(default = "super::true_default")]
    pub enabled: bool,
}

impl Default for FreeMemoryAfterGPUUpload {
    fn default() -> Self {
        Self::default_with_enabled(true)
    }
}

impl DefaultWithEnabled for FreeMemoryAfterGPUUpload {
    fn default_with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }
}


#[derive(Serialize, Deserialize, Hash)]
pub struct MultiThreadedSampling {
    #[serde(default = "super::false_default")]
    pub enabled: bool,
    #[serde(default = "multi_threaded_sampling_threads_default")]
    pub threads: Option<usize>,
}

impl Default for MultiThreadedSampling {
    fn default() -> Self {
        Self::default_with_enabled(false)
    }
}

impl DefaultWithEnabled for MultiThreadedSampling {
    fn default_with_enabled(enabled: bool) -> Self {
        Self { enabled, threads: multi_threaded_sampling_threads_default() }
    }
}

fn multi_threaded_sampling_threads_default() -> Option<usize> {
    match available_parallelism() {
        Ok(non_zero) => Some((non_zero.get().saturating_sub(2)).max(2)),
        Err(error) => {
            warn!(
                "Failed to retrieve available threads for parallelism from the OS! Error: {}", error.to_string()
            );

            None
        },
    }
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


trait DefaultWithEnabled: Default {
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
