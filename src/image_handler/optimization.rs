use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImageOptimizations {
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
    MonitorDownsampling(u32),
    /// Basically `MonitorDownsampling` but the image is dynamically sampled up and down relative to the zoom factor. 
    /// When the user zooms into an image (especially one that was already downsampled via something like MonitorDownsampling) 
    /// detail in the image will be lost, so to combat this the image is dynamically upsampled to bring back that detail 
    /// when necessary (such as the user zooming in).
    /// 
    /// The opposite happens when the full detail is no longer required to save your memory.
    DynamicSampling(bool, bool)
}

impl ImageOptimizations {
    // TODO: remove this when has_optimization is also removed.
    pub fn id(&self) -> &str {
        match self {
            Self::MonitorDownsampling(_) => "monitor-downsampling",
            Self::DynamicSampling(_, _) => "dynamic-sampling",
        }
    }
}

impl Display for ImageOptimizations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MonitorDownsampling(marginal_factor) => write!(f, "Monitor Downsampling (@ {})", marginal_factor / 100),
            Self::DynamicSampling(_, _) => write!(f, "Dynamic Sampling"),
        }
    }
}