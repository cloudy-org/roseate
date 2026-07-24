use std::{fs::{self, File, OpenOptions}, io::{Read, Seek, Write}, thread, time::Duration};

use cirrus_egui::{scheduler::Scheduler};
use cirrus_path::get_user_cache_cloudy_folder_path;
use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

type Size = (u32, u32);

#[derive(Debug, Serialize, Deserialize)]
struct MonitorSizeCacheData {
    #[serde(default)]
    sizes: Vec<Size>,
}

#[derive(Clone)]
pub struct MonitorSize {
    size: Option<Size>,

    fallback_size: Size,
    override_size: Option<Size>,

    write_to_disk_delay_scheduler: Scheduler<Size>
}

impl MonitorSize {
    pub fn new(fallback_size: Size, override_size: Option<Size>) -> Self {
        Self {
            size: None,

            fallback_size,
            override_size,

            write_to_disk_delay_scheduler: Scheduler::UNSET
        }
    }

    pub fn get(&self) -> Size {
        match self.override_size {
            Some(size) => size,
            None => self.size.unwrap_or(self.fallback_size),
        }
    }

    pub fn exists(&self) -> bool {
        self.size.is_some() || self.override_size.is_some()
    }

    pub fn update_size(&mut self, monitor_size: Size) {
        if let Some(monitor_size_to_write) = self.write_to_disk_delay_scheduler.update() {
            thread::spawn(move || {
                if let Err(error) = Self::write_size_to_disk(monitor_size_to_write) {
                    log::error!("Failing to write to monitor size cache file! Error: {error}");
                }
            });
        }

        if monitor_size == self.size.unwrap_or_default() {
            return;
        }

        log::info!("Updating monitor size to '{monitor_size:?}'...");

        self.write_to_disk_delay_scheduler = Scheduler::new(
            move || monitor_size,
            Duration::from_secs(2)
        );

        self.size = Some(monitor_size);
    }

    pub fn update_size_from_cache(&mut self) -> Result<()> {
        let cloudy_cache_path = get_user_cache_cloudy_folder_path()
            .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?;

        let monitor_size_cache_path = cloudy_cache_path.join("roseate").join("monitor_size");

        if !monitor_size_cache_path.exists() {
            return Ok(());
        }

        let monitor_size_file = File::open(&monitor_size_cache_path)
            .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?;

        let data: MonitorSizeCacheData = serde_json::from_reader(monitor_size_file)
            .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?;

        // NOTE: should we make this customizable???
        let latest_size = data.sizes.last();

        self.size = latest_size.copied();

        Ok(())
    }

    fn write_size_to_disk(monitor_size: Size) -> Result<()> {
        debug!("Writing to cached monitor size file with '{:?}'...", monitor_size);

        match get_user_cache_cloudy_folder_path() {
            Ok(cloudy_cache_path) => {
                let cache_path = cloudy_cache_path.join("roseate");

                // TODO: modularize into a function for reuse
                if !cache_path.exists() {
                    debug!("Creating cache directory for roseate at '{}'...", cache_path.to_string_lossy());

                    fs::create_dir_all(&cache_path)
                        .map_err(|error| Error::CacheDirectoryCreationFailure {
                            path: cache_path.to_string_lossy().to_string(),
                            error: error.to_string()
                        })?;

                    debug!("Cache directory created ({})!", cache_path.to_string_lossy());
                }

                let monitor_size_file_path = cache_path.join("monitor_size");

                debug!("Creating and opening 'monitor_size' cache file...");
                let mut json_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .read(true)
                    .open(monitor_size_file_path)
                    .map_err(|error| Error::WriteCachedMonitorSizeFailure { error: error.to_string() })?;

                debug!("Appling file lock to 'monitor_size' cache file...");

                match json_file.try_lock_shared() {
                    Ok(_) => {
                        debug!("File locked successfully! Reading json string from 'monitor_size' cache file...");
                        let mut json_contents = String::new();

                        json_file.read_to_string(&mut json_contents)
                            .map_err(|error| Error::WriteCachedMonitorSizeFailure { error: error.to_string() })?;

                        debug!("Parsing json string ({})...", json_contents);

                        let mut json_data = match serde_json::from_str::<MonitorSizeCacheData>(&json_contents) {
                            Ok(json_data) => json_data,
                            Err(error) => {
                                // if the file is empty like when it's first created there will be a serde_json error.
                                if !error.is_eof() {
                                    warn!(
                                        "Failed to parse json in 'monitor_size'! Rewriting with default values! Error: {}",
                                        error
                                    );
                                }
    
                                MonitorSizeCacheData { sizes: Vec::default() }
                            },
                        };

                        if let Some(latest_size) = json_data.sizes.last() {
                            if latest_size == &monitor_size {
                                debug!(
                                    "The 'monitor_size' persistent cache already contains this \
                                    monitor's size as it's last appended size so we'll skip a rewrite..."
                                );

                                return Ok(());
                            }
                        }

                        json_data.sizes.retain(|size| *size != monitor_size);
                        json_data.sizes.push(monitor_size);

                        debug!("Writing json data to 'monitor_size' cache file...");

                        json_file.rewind()
                            .map_err(|error| Error::WriteCachedMonitorSizeFailure { error: error.to_string() })?;

                        json_file.write(&serde_json::to_vec(&json_data).unwrap())
                            .map_err(|error| Error::WriteCachedMonitorSizeFailure { error: error.to_string() })?;

                        Ok(())
                    },
                    Err(error) => Err(
                        Error::CachedMonitorSizeAlreadyLocked { error: error.to_string() }
                    ),
                }
            },
            Err(error) => Err(
                Error::WriteCachedMonitorSizeFailure { error: error.to_string() }
            )
        }
    }
}