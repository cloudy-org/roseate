use log::debug;

use crate::monitor_size::MonitorSize;

// TODO: when I have a centralized place for individual optimization logic move this there.
// 27/03/2025: We will have a centralized place soon:tm:, prbos under "image_handler/".
pub fn get_monitor_downsampling_size(marginal_allowance: u32, image_size: (u32, u32), monitor_size: &MonitorSize) -> (u32, u32) {
    // marginal_allowance is supposed to be a f32 but instead 
    // it's a u32 hence all it's units have been shifted forward one.
    // 
    // E.g. "130" is "1.3"
    let marginal_allowance_scale = marginal_allowance as f32 / 100.0;

    debug!(
        "Image Size: {} x {}", image_size.0, image_size.1
    );

    let (monitor_width, monitor_height) = monitor_size.get();

    debug!(
        "Display (Monitor) Size: {} x {}", monitor_width, monitor_height
    );

    let (width, height) = (
        (monitor_width as f32 * marginal_allowance_scale) as u32,
        (monitor_height as f32 * marginal_allowance_scale) as u32
    );

    debug!(
        "Display + Monitor Downsample Marginal Allowance ({}): {} x {}",
        marginal_allowance_scale, width, height
    );

    (width, height)
}