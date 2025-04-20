use std::{fs::{File, OpenOptions}, io::{BufReader, Read, Seek, Write}, time::{Duration, Instant}};

use egui_notify::ToastLevel;
use fs2::FileExt;
use eframe::egui::Context;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

use crate::{error::Error, files, notifier::NotifierAPI};

// TODO: move this to cirrus when it's good and stable enough.

#[derive(Debug, Serialize, Deserialize)]
struct MonitorSizeCacheData {
    #[serde(default)]
    sizes: Vec<(f32, f32)>,
}

#[derive(Clone)]
pub struct MonitorSize {
    monitor_size: Option<(f32, f32)>,
    last_monitor_size: Option<(f32, f32)>,
    retry_persistent_state_update: Option<Instant>,
    default_fallback_size: (f32, f32),
    override_monitor_size: Option<(f32, f32)>
}

// NOTE: need this to be a struct because soon I'll be
// implementing a system where monitor size is cached somewhere so
// roseate knows the correct monitor resolution before the window is available.
impl MonitorSize {
    pub fn new(fallback_monitor_size: Option<(f32, f32)>, override_monitor_size: Option<(f32, f32)>) -> Self {
        Self {
            monitor_size: override_monitor_size,
            last_monitor_size: None,
            retry_persistent_state_update: None,
            default_fallback_size: fallback_monitor_size.unwrap_or((1920.0, 1080.0)),
            override_monitor_size: override_monitor_size
        }
    }

    /// Returns the resolution of the current monitor (the monitor the window is currently at)
    /// if it can't retrieve the real monitor resolution the default fallback resolution is returned.
    pub fn get(&self) -> (f32, f32) {
        self.monitor_size.unwrap_or(self.default_fallback_size)
    }

    pub fn exists(&self) -> bool {
        self.monitor_size.is_some()
    }

    pub fn fetch_from_cache(&mut self) {
        if self.override_monitor_size.is_some() {
            debug!("Ignoring fetch from cache as monitor size was overridden!");
            return;
        }

        let cache_path_result = files::get_cache_path();

        if let Ok(cache_path) = cache_path_result {
            let monitor_size_file_path = cache_path.join("monitor_size");

            let file = File::open(&monitor_size_file_path);

            if let Err(error) = file {
                let error = Error::FailedToOpenFile(
                    Some(error.to_string()), monitor_size_file_path
                );
    
                warn!("{}", error);
                return;
            }
    
            let data_result = serde_json::from_reader::<BufReader<File>, MonitorSizeCacheData>(
                BufReader::new(file.unwrap())
            );
    
            if let Ok(data) = data_result {
                // NOTE: should we make this customizable???
                let last_size = data.sizes.last();
    
                self.monitor_size = last_size.copied();
            }
        }
    }

    pub fn update(&mut self, ctx: &Context, notifier: &mut NotifierAPI) {
        self.last_monitor_size = self.monitor_size;

        if self.override_monitor_size.is_some() {
            // monitor size is forever overridden so we can 
            // now ignore anything from egui's viewport monitor size.
            return;
        }

        ctx.input(|i| {
            let active_viewpoint = i.viewport();

            if let Some(size) = active_viewpoint.monitor_size {
                self.monitor_size = Some((size.x, size.y));
            }
        });

        self.persistent_state_update(notifier);
    }

    fn persistent_state_update(&mut self, notifier: &mut NotifierAPI) {
        if self.monitor_size == self.last_monitor_size && self.retry_persistent_state_update == None {
            return;
        }

        // skip update if 2 seconds hasn't passed since the last time we tried.
        if let Some(last_retry) = self.retry_persistent_state_update {
            if Duration::from_secs(2) > last_retry.elapsed() {
                return;
            }
        }

        // we can unwrap because "self.monitor_size" will never be None once it's Some() in my logic.
        let monitor_size_to_add = self.monitor_size.unwrap();

        debug!("Updating persistent monitor size state with '{:?}'...", monitor_size_to_add);

        let cache_path_result = files::get_cache_path();

        match cache_path_result {
            Ok(cache_path) => {
                let monitor_size_file_path = cache_path.join("monitor_size");

                debug!("Creating and opening 'monitor_size' cache file...");
                let json_file_result = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .read(true)
                    .open(monitor_size_file_path);

                if let Ok(mut json_file) = json_file_result {
                    debug!("Appling file lock to 'monitor_size' cache file...");
                    let result = json_file.try_lock_shared();

                    match result {
                        Ok(_) => {
                            debug!("File locked successfully! Reading json string from 'monitor_size' cache file...");
                            let mut json_contents = String::new();

                            if let Err(error) = json_file.read_to_string(&mut json_contents) {
                                error!("Failed to read json string from the 'monitor_size' file! Error: {}", error);
                                return ;
                            }

                            debug!("Parsing json string ({})...", json_contents);
                            let json_data_result = serde_json::from_str::<MonitorSizeCacheData>(&json_contents);

                            let mut json_data = match json_data_result {
                                Ok(json_data) => json_data,
                                Err(error) => {
                                    // if the file is empty like when it's first created there will be a serde_json error.
                                    if !error.is_eof() {
                                        error!(
                                            "Failed to parse json in 'monitor_size'! Defaulting to default values! Error: {}",
                                            error
                                        );
                                    }
        
                                    MonitorSizeCacheData { sizes: Vec::default() }
                                },
                            };

                            if let Some(last_size) = json_data.sizes.last() {
                                if last_size == &monitor_size_to_add {
                                    debug!(
                                        "The 'monitor_size' persistent cache already contains this \
                                        monitor's size as it's last appended size so we'll skip a rewrite."
                                    );
                                    return;
                                }
                            }

                            json_data.sizes.retain(|size| *size != monitor_size_to_add);
                            json_data.sizes.push(monitor_size_to_add);

                            debug!("Writing json data to 'monitor_size' cache file...");
                            let rewind_result = json_file.rewind();

                            if let Err(error) = rewind_result {
                                error!(
                                    "Failed to write to 'monitor_size' cache because we failed to rewind the json file! Error: {}",
                                    error
                                );
                                return;
                            }

                            if let Err(error) = json_file.write(&serde_json::to_vec(&json_data).unwrap()) {
                                error!("Failed to write to 'monitor_size' cache file! Error: {}", error);
                                return;
                            }
                        },
                        Err(error) => {
                            warn!(
                                "The 'monitor_size' file is currently locked by another Roseate 
                                instant, so we will have to wait until the other instance let's go of 
                                the file until we can save our monitor size state to it. \n\n Error: {}",
                                error
                            );
                            self.retry_persistent_state_update = Some(Instant::now());
                        },
                    }
                }
            },
            Err(error) => {
                // TODO: test this and see how it looks.
                notifier.toasts.lock()
                    .unwrap()
                    .toast_and_log(
                        error.into(),
                        ToastLevel::Error
                    );

                return;
            }
        }
    }
}