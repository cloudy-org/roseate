use crate::monitor_size::MonitorSize;

pub fn get_monitor_downsampling_size(marginal_allowance: &u32, monitor_size: &MonitorSize) -> (u32, u32) {
    // marginal_allowance is supposed to be a f32 but instead 
    // it's a u32 hence all it's units have been shifted forward one.
    // 
    // E.g. "130" is "1.3"
    let marginal_allowance_scale = *marginal_allowance as f32 / 100.0;

    let (monitor_width, monitor_height) = monitor_size.get();

    let (width, height) = (
        (monitor_width as f32 * marginal_allowance_scale) as u32,
        (monitor_height as f32 * marginal_allowance_scale) as u32
    );

    (width, height)
}