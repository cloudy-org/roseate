use std::{fs::{self, OpenOptions}, io::{Read, Seek, Write}, thread, time::Duration};

use cirrus_egui::{notifier::{Notifier, ToastLevel, toast::ToastText}, scheduler::Scheduler};
use cirrus_path::get_user_cache_cloudy_folder_path;
use fs2::FileExt;

use crate::error::{Error, Result};

type Size = (u32, u32);

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

    pub fn update_size(&mut self, monitor_size: Size, notifier: &mut Notifier) {
        if let Some(monitor_size_to_write) = self.write_to_disk_delay_scheduler.update() {
            let notifier = notifier.clone();

            thread::spawn(move || {
                notifier.set_loading(Some("Updating monitor size on disk..."));

                if let Err(error) = Self::write_size_to_disk(monitor_size_to_write) {
                    notifier.show_toast(
                        ToastText::Error(error.into()),
                        ToastLevel::Error,
                        |_| {}
                    );
                }

                notifier.unset_loading();
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
        log::debug!("Updating monitor size from cache on disk...");

        let cloudy_cache_path = get_user_cache_cloudy_folder_path()
            .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?;

        let monitor_size_cache_path = cloudy_cache_path.join("roseate").join("monitor_size");

        if !monitor_size_cache_path.exists() {
            log::warn!("Cannot pull from monitor size from cache as it does not exist yet.");
            return Ok(());
        }

        log::debug!("Opening 'monitor_size' cache file for reading...");

        let mut monitor_size_file = OpenOptions::new()
            .read(true)
            .open(monitor_size_cache_path)
            .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?;

        log::debug!("Waiting for 'monitor_size' cache file to be unlocked by another instance...");

        monitor_size_file.lock_shared()
            .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?;

        log::debug!("Reading monitor size from cache file...");

        let mut monitor_size_string = String::new();

        monitor_size_file.read_to_string(&mut monitor_size_string)
            .map_err(
                |error| Error::GetCachedMonitorSizeFailure {
                    error: error.to_string()
                }
            )?;

        log::debug!("Parsing monitor size '({})' from cache file...", monitor_size_string);

        let monitor_size = match monitor_size_string.split_once("x") {
            Some((width, height)) => (
                width.parse::<u32>()
                    .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?,
                height.parse::<u32>()
                    .map_err(|error| Error::GetCachedMonitorSizeFailure { error: error.to_string() })?
            ),
            None => return Err(
                Error::GetCachedMonitorSizeFailure {
                    error: String::from(
                        "Failed to parse monitor size from file \
                        correctly! 'x' to split w/h was not found in the string!"
                    )
                }
            ),
        };

        self.size = Some(monitor_size);

        Ok(())
    }

    // NOTE: might remove this
    pub fn rewrite_size_to_disk(&self, notifier: &mut Notifier) {
        let notifier = notifier.clone();
        let monitor_size = self.get();

        thread::spawn(move || {
            notifier.set_loading(Some("Updating monitor size on disk..."));

            if let Err(error) = Self::write_size_to_disk(monitor_size) {
                notifier.show_toast(
                    ToastText::Error(error.into()),
                    ToastLevel::Error,
                    |_| {}
                );
            }

            notifier.unset_loading();
        });
    }

    fn write_size_to_disk(monitor_size: Size) -> Result<()> {
        log::debug!("Writing to cached monitor size file with '{:?}'...", monitor_size);

        match get_user_cache_cloudy_folder_path() {
            Ok(cloudy_cache_path) => {
                let cache_path = cloudy_cache_path.join("roseate");

                // TODO: modularize into a function for reuse
                if !cache_path.exists() {
                    log::debug!("Creating cache directory for roseate at '{}'...", cache_path.to_string_lossy());

                    fs::create_dir_all(&cache_path)
                        .map_err(|error| Error::CacheDirectoryCreationFailure {
                            path: cache_path.to_string_lossy().to_string(),
                            error: error.to_string()
                        })?;

                    log::debug!("Cache directory created ('{}')!", cache_path.to_string_lossy());
                }

                let monitor_size_file_path = cache_path.join("monitor_size");

                log::debug!("Creating and opening 'monitor_size' cache file...");
                let mut monitor_size_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(monitor_size_file_path)
                    .map_err(|error| Error::WriteCachedMonitorSizeFailure { error: error.to_string() })?;

                log::debug!("Appling file lock to 'monitor_size' cache file...");

                match monitor_size_file.try_lock_exclusive() {
                    Ok(_) => {
                        log::debug!("File locked successfully! Writing monitor resolution to 'monitor_size' cache file...");

                        monitor_size_file.rewind()
                            .map_err(|error| Error::WriteCachedMonitorSizeFailure { error: error.to_string() })?;

                        monitor_size_file.write_all(format!("{}x{}", monitor_size.0, monitor_size.1).as_bytes())
                            .map_err(|error| Error::WriteCachedMonitorSizeFailure { error: error.to_string() })?;

                        log::debug!("Monitor size written to disk successfully!");

                        Ok(())
                    },
                    Err(_) => {
                        log::error!(
                            "The 'monitor_size' cache file is currently locked by another instance \
                            of Roseate, hence we cannot update the file at this moment.",
                        );

                        Ok(())
                    },
                }
            },
            Err(error) => Err(
                Error::WriteCachedMonitorSizeFailure {
                    error: error.to_string()
                }
            )
        }
    }
}