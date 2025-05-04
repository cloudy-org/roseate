use std::time::Duration;

use cirrus_egui::v1::scheduler::Scheduler;
use eframe::egui::Vec2;
use log::debug;

use crate::{image::image::ImageSizeT, image_handler::{monitor_downsampling::get_monitor_downsampling_size, optimization::ImageOptimizations}, monitor_size::MonitorSize, zoom_pan::ZoomPan};

use super::ImageHandler;

impl ImageHandler {
    pub fn dynamic_sampling_update(&mut self, zoom_pan: &ZoomPan, monitor_size: &MonitorSize) {
        if let Some(image) = &self.image {
            let is_enabled = self.has_optimization(
                &ImageOptimizations::DynamicSampling(bool::default(), bool::default())
            ).is_some();

            if !is_enabled {
                return;
            }

            if zoom_pan.zoom_factor <= 1.0 {
                self.last_zoom_factor = 1.0;
                self.accumulated_zoom_factor_change = 0.0;
                // self.dynamic_sampling_new_resolution = (0, 0);
                // self.dynamic_sampling_old_resolution = (0, 0);

                if zoom_pan.zoom_factor < 1.0 {
                    return;
                }
            }

            self.accumulated_zoom_factor_change += (zoom_pan.zoom_factor).log2() - (self.last_zoom_factor).log2();

            self.last_zoom_factor = zoom_pan.zoom_factor;

            let change = 0.8;

            if !(self.accumulated_zoom_factor_change <= -change) && !(self.accumulated_zoom_factor_change >= change) {
                return;
            }

            // TODO: we seriously need to set image size universally to 
            // be ImageSizeT because those "as" statements below is not it bro.

            let max_image_size = (image.image_size.width as u32, image.image_size.height as u32);
            let mut image_size = max_image_size;

            // TODO: (28/03/2025) check if we even need this now
            if let Some(ImageOptimizations::MonitorDownsampling(marginal_allowance)) = self.has_optimization(
                &ImageOptimizations::MonitorDownsampling(u32::default())
            ) {
                image_size = get_monitor_downsampling_size(*marginal_allowance, monitor_size);
            }

            let new_resolution = zoom_pan.relative_image_size(
                Vec2::new(image_size.0 as f32, image_size.1 as f32)
            );
            let new_resolution = (
                new_resolution.x.clamp(0.0, max_image_size.0 as f32) as u32,
                new_resolution.y.clamp(0.0, max_image_size.1 as f32) as u32
            );

            if self.accumulated_zoom_factor_change >= change {
                self.set_resolution_and_schedule_dynamic_sample(
                    true,
                    new_resolution
                );
            }

            if self.accumulated_zoom_factor_change <= -change  {
                self.set_resolution_and_schedule_dynamic_sample(
                    false,
                    new_resolution
                );
            }

            self.accumulated_zoom_factor_change = 0.0;
        }
    }

    fn set_resolution_and_schedule_dynamic_sample(
        &mut self,
        upsample: bool,
        resolution: ImageSizeT
    ) {
        let delay = match upsample {
            true => Duration::from_secs_f32(2.5),
            false => Duration::from_secs(5),
        };

        self.dynamic_sampling_new_resolution = resolution;

        // this will tell the update loop in ImageHandler when it is time to downsample or upsample.
        let schedule = Scheduler::new(|| {}, delay);

        if self.dynamic_sample_schedule.is_some() {
            debug!("Last scheduled dynamic image sampling cancelled!");
        }

        self.dynamic_sample_schedule = Some(schedule);

        debug!(
            "Dynamic image sampling has been scheduled for '{:.0} x {:.0}' in {:.2} seconds...",
            resolution.0,
            resolution.1,
            delay.as_secs_f64()
        );
    }
}