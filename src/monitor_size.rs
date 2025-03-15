use std::fs::{self};

use eframe::egui::Context;
use serde::Deserialize;

use crate::files;

// TODO: move this to cirrus when it's good and stable enough.

#[derive(Debug, Deserialize)]
struct MonitorSizeCacheData {
    #[serde(default)]
    sizes: Vec<String>,
}

#[derive(Clone)]
pub struct MonitorSize {
    monitor_size: Option<(f32, f32)>,
    last_monitor_size: Option<(f32, f32)>,
    default_fallback_size: (f32, f32),
}

// NOTE: need this to be a struct because soon I'll be
// implementing a system where monitor size is cached somewhere so
// roseate knows the correct monitor resolution before the window is available.
impl MonitorSize {
    pub fn new(fallback_monitor_size: Option<(f32, f32)>) -> Self {
        Self {
            monitor_size: None,
            last_monitor_size: None,
            default_fallback_size: fallback_monitor_size.unwrap_or((1920.0, 1080.0))
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.last_monitor_size = self.monitor_size;

        ctx.input(|i| {
            let active_viewpoint = i.viewport();

            if let Some(size) = active_viewpoint.monitor_size {
                self.monitor_size = Some((size.x, size.y));
            }
        });

        self.persistent_state_update();
    }

    fn persistent_state_update(&self) {
        if self.monitor_size == self.last_monitor_size {
            return;
        }

        let cache_path = files::get_cache_path();

        if let Ok(cache_path) = cache_path {
            let monitor_size_file_path = cache_path.join("monitor_size");

            if !monitor_size_file_path.exists() {
                if fs::write(&monitor_size_file_path, "{}").is_err() {
                    return;
                }
            }

            if let Ok(contents) = fs::read_to_string(monitor_size_file_path) {
                if let Ok(data) = serde_json::from_str::<MonitorSizeCacheData>(&contents) {
                    let monitor_sizes = data.sizes;
                    // TODO: (15/03/2025) where I left off!
                }
            }
        }
    }

    /// Returns the resolution of the current monitor (the monitor the window is currently at)
    /// if it can't retrieve the real monitor resolution the default fallback resolution is returned.
    pub fn get(&self) -> (f32, f32) {
        self.monitor_size.unwrap_or(self.default_fallback_size)
    }
}