use roseate_core::decoded_image::ImageSize;

use crate::monitor_size::MonitorSize;

#[derive(Debug)]
pub struct ImageOptimizations {
    pub monitor_downsampling: Option<MonitorDownsampling>,
    pub dynamic_sampling: Option<DynamicSampling>,
    pub free_memory_after_gpu_upload: bool,
    pub multi_threaded_sampling: Option<MultiThreadedSampling>,
}

impl Default for ImageOptimizations {
    fn default() -> Self {
        Self {
            monitor_downsampling: Some(MonitorDownsampling::default()),
            dynamic_sampling: None,
            free_memory_after_gpu_upload: true,
            multi_threaded_sampling: None
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct DynamicSampling { pub up: bool, pub down: bool }

#[derive(Debug)]
pub struct MultiThreadedSampling {}