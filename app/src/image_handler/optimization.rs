use roseate_core::decoded_image::ImageSize;

use crate::monitor_size::MonitorSize;

#[derive(Default, Debug)]
pub struct ImageOptimizations {
    /// Downsamples the image roughly to the resolution of your monitor.
    /// 
    /// Images don't always have to be displayed at their full native resolution, especially when 
    /// the image is significantly bigger than your monitor can even display, so to save memory 
    /// we downsample the image. Downsampling decreases the amount of memory eaten up by the image 
    /// at the cost of CPU time wasted actually resizing the image. The bigger the image the more time 
    /// it will take to downsample but we think memory savings are more valuable in this circumstance. 
    /// You can enable or disable downsampling in the config if you do not wish for such memory savings. 
    /// Setting the overall optimization mode to speed ("s") will automatically disable this.
    /// 
    /// NOTE: "The image's aspect ratio is preserved. The image is scaled to the maximum 
    /// possible size that fits within the bounds specified by the width and height." ~ Image Crate
    pub monitor_downsampling: Option<MonitorDownsampling>,
    /// Basically `MonitorDownsampling` but the image is dynamically sampled up and down relative to the zoom factor. 
    /// When the user zooms into an image (especially one that was already downsampled via something like MonitorDownsampling) 
    /// detail in the image will be lost, so to combat this the image is dynamically upsampled to bring back that detail 
    /// when necessary (such as the user zooming in).
    /// 
    /// The opposite happens when the full detail is no longer required to save your memory.
    pub dynamic_sampling: Option<DynamicSampling>,
}

#[derive(Debug)]
pub struct MonitorDownsampling { pub marginal_allowance: f32 }

impl MonitorDownsampling {
    pub fn get_size_relative_to_monitor(&self, monitor_size: &MonitorSize) -> ImageSize {
        // marginal_allowance is supposed to be a f32 but instead 
        // it's a u32 hence all it's units have been shifted forward one.
        // 
        // E.g. "130" is "1.3"
        // TODO: remove the above

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