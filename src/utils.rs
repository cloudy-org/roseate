/// Where I dump functions temporally that I don't know where to place.
/// Don't just be lazy and dump everything in here ~~like me~~.

use display_info::DisplayInfo;
use crate::error::{Error, Result};

// TODO: Return actual error instead of "()".
pub fn get_monitor_size_before_egui_window() -> Result<(u32, u32)> {
    let all_display_infos = DisplayInfo::all().expect(
        "Failed to get information about your display monitor!"
    );

    // NOTE: I don't think the first monitor is always the primary and 
    // if that is the case then we're gonna have a problem. (i.e images overly downsampled or not at all)
    match all_display_infos.first() {
        Some(primary_monitor_maybe) => {
            Ok((primary_monitor_maybe.width, primary_monitor_maybe.height))
        },
        None => Err(
            Error::MonitorNotFound(None)
        ),
    }
}