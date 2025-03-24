use std::time::Duration;

use eframe::egui::Vec2;
use log::debug;

use crate::{image::optimization::{get_monitor_downsampling_size, EventImageOptimizations, ImageOptimizations, InitialImageOptimizations}, image_handler::ImageHandler, monitor_size::MonitorSize, scheduler::Scheduler, zoom_pan::ZoomPan};

impl ImageHandler {
    pub fn dynamic_sampling_update(&mut self, zoom_pan: &ZoomPan, monitor_size: &MonitorSize) {
        if let Some(image) = &self.image {
            // the zoom factor change since the last dynamic upsample / downsample.
            let is_enabled = image.optimizations.contains(
                &ImageOptimizations::EventBased(EventImageOptimizations::DynamicUpsampling)
            );

            if zoom_pan.zoom_factor <= 1.0 || !is_enabled {
                self.last_zoom_factor = 1.0;
                self.accumulated_zoom_factor_change = 0.0;
                return;
            }

            self.accumulated_zoom_factor_change += (zoom_pan.zoom_factor).log2() - (self.last_zoom_factor).log2();

            self.last_zoom_factor = zoom_pan.zoom_factor;

            let change = 0.8;

            if !(self.accumulated_zoom_factor_change <= -change) && !(self.accumulated_zoom_factor_change >= change) {
                return;
            }

            println!("zoom factor {}", zoom_pan.zoom_factor);

            // TODO: we seriously need to find about setting image size universally to some type 
            // like (usize, usize), (u32, u32) or (f32, f32) because those "as" statements below is not it bro.

            // TODO: if monitor downsampling is set use monitor downsampled image size instead of normal image size.
            let max_image_size = (image.image_size.width as u32, image.image_size.height as u32);
            let mut image_size = max_image_size;

            if let Some(ImageOptimizations::Initial(InitialImageOptimizations::MonitorDownsampling(marginal_allowance))) = image.has_optimization(
                &ImageOptimizations::Initial(InitialImageOptimizations::MonitorDownsampling(0))
            ) {
                image_size = get_monitor_downsampling_size(*marginal_allowance, image_size, monitor_size);
            }

            let new_resolution = zoom_pan.relative_image_size(
                Vec2::new(image_size.0 as f32, image_size.1 as f32)
            );
            let new_resolution = (
                new_resolution.x.clamp(image_size.0 as f32, max_image_size.0 as f32),
                new_resolution.y.clamp(image_size.1 as f32, max_image_size.1 as f32)
            );

            // TODO: also don't upsample if image is already at it's max resolution.
            if self.accumulated_zoom_factor_change >= change {
                // TODO: schedule a image reload with new dimensions.

                println!("change + {}", self.accumulated_zoom_factor_change);
                self.schedule_image_dynamic_sample(true, new_resolution);
            }

            if self.accumulated_zoom_factor_change <= -change  {
                println!("change - {}", self.accumulated_zoom_factor_change);
                self.schedule_image_dynamic_sample(false, new_resolution);
            }

            self.accumulated_zoom_factor_change = 0.0;
        }
    }

    pub fn schedule_image_dynamic_sample(&mut self, upsample: bool, resolution: (f32, f32)) {
        if resolution == self.dynamic_sampling_old_resolution {
            debug!(
                "Not scheduling dynamic image sample for '{:.0}x{:.0}' \
                as that resolution has already been set or scheduled.",
                resolution.0,
                resolution.1
            );
            return;
        }

        let delay = match upsample {
            true => Duration::from_secs_f32(1.5),
            false => Duration::from_secs(5),
        };

        let schedule = Scheduler::new(
            move || {
                // TODO: figure out how to tell the Image or ImageLoader struct 
                // to reload the image with DownSampling or UpSampling optimizations.
                println!("IMAGE SAMPLED! {:?}", resolution);
            },
            delay
        );

        if self.dynamic_sample_schedule.is_some() {
            debug!(
                "Last scheduled dynamic image sampling ({:.0}x{:.0}) cancelled!",
                resolution.0,
                resolution.1
            );
        }

        self.dynamic_sample_schedule = Some(schedule);

        debug!(
            "Dynamic image sampling has been scheduled for '{:.0}x{:.0}' in {:.2} seconds...",
            resolution.0,
            resolution.1,
            delay.as_secs_f64()
        );

        self.dynamic_sampling_old_resolution = resolution;
    }
}