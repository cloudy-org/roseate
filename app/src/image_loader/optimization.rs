use std::thread::available_parallelism;

use log::warn;
use roseate_core::decoded_image::ImageSize;

use crate::monitor_size::MonitorSize;

#[derive(Debug, Clone)]
pub struct ImageOptimizations {
    pub monitor_downsampling: Option<MonitorDownsampling>,
    pub dynamic_sampling: Option<DynamicSampling>,
    pub consume_pixels_during_gpu_upload: bool,
    pub multi_threaded_sampling: Option<MultiThreadedSampling>,
}

impl ImageOptimizations {
    pub fn balanced() -> Self {
        Self {
            monitor_downsampling: Some(MonitorDownsampling::default()),
            dynamic_sampling: None,
            consume_pixels_during_gpu_upload: true,
            multi_threaded_sampling: Some(MultiThreadedSampling::default())
        }
    }

    pub fn speed() -> Self {
        Self {
            monitor_downsampling: None,
            ..Self::balanced()
        }
    }

    pub fn quality() -> Self {
        Self {
            monitor_downsampling: None,
            multi_threaded_sampling: None,
            ..Self::balanced()
        }
    }

    // NOTE: Efficient mode will be added when support for loading
    // multiple images is added. Efficient mode will try to disable 
    // anything that is inefficient in the slightest and set optimizations 
    // like 'consume_pixels_during_gpu_upload' to FALSE so decoded images stay 
    // in cpu memory for rapid and efficient reuploading to the gpu while 
    // cycling through images a carousel mode.

    // pub fn efficient() -> Self {
    //     Self {
    //         monitor_downsampling: None,
    //         dynamic_sampling: None,
    //         consume_pixels_during_gpu_upload: false,
    //         multi_threaded_sampling: None
    //     }
    // }

    // might move this into something like 
    // 'ImageOptimizations::from_config()' in the future.
    pub fn normalize(mut self) -> Self {
        if self.dynamic_sampling.is_some() {
            self.consume_pixels_during_gpu_upload = false;
            warn!(
                "Consume pixels during GPU upload optimization was disabled \
                because dynamic sampling optimization was enabled! Pick one of them."
            );
        }

        self
    }
}

#[derive(Debug, Clone)]
pub struct MonitorDownsampling { pub marginal_allowance: f32 }

impl Default for MonitorDownsampling {
    fn default() -> Self {
        Self {
            marginal_allowance: 1.4
        }
    }
}

impl MonitorDownsampling {
    pub fn get_size_relative_to_monitor(&self, monitor_size: &MonitorSize) -> ImageSize {
        let (monitor_width, monitor_height) = monitor_size.get();

        let (width, height) = (
            (monitor_width as f32 * self.marginal_allowance) as u32,
            (monitor_height as f32 * self.marginal_allowance) as u32
        );

        (width, height)
    }
}

#[derive(Debug, Clone)]
pub struct DynamicSampling { pub up: bool, pub down: bool }

#[derive(Debug, Clone)]
pub struct MultiThreadedSampling { pub number_of_threads: Option<usize> }

impl Default for MultiThreadedSampling {
    fn default() -> Self {
        Self {
            number_of_threads: match available_parallelism() {
                Ok(non_zero) => Some((non_zero.get().saturating_sub(2)).max(2)),
                Err(error) => {
                    warn!(
                        "Failed to retrieve available threads for parallelism from the OS! Error: {}", error.to_string()
                    );

                    None
                },
            }
        }
    }
}