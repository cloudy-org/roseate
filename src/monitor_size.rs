use eframe::egui::Context;

pub struct MonitorSize {
    monitor_size: Option<(f32, f32)>,
}

// NOTE: need this to be a struct because soon I'll be
// implementing a system where monitor size is cached somewhere so
// roseate knows the correct monitor resolution before the window is available.
impl MonitorSize {
    pub fn new() -> Self {
        Self {
            monitor_size: None,
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        ctx.input(|i| {
            let active_viewpoint = i.viewport();

            if let Some(size) = active_viewpoint.monitor_size {
                self.monitor_size = Some((size.x, size.y));
            }
        });
    }

    /// Returns the resolution of the current monitor.
    pub fn get(&self) -> (f32, f32) {
        self.monitor_size.unwrap_or((1920.0, 1080.0))
    }
}