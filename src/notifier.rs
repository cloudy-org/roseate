use std::sync::{Arc, Mutex, RwLock};

use eframe::egui::Context;
use egui_notify::{Toast, ToastLevel, Toasts};
use log::info;

use crate::error::Error;

#[derive(Default, Clone)]
pub struct Loading {
    pub message: Option<String>
}

#[derive(Default)]
pub struct ToastsManager {
    pub toasts: Toasts
}

impl ToastsManager {
    pub fn new() -> Self {
        Self {
            toasts: Toasts::default(),
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        self.toasts.show(ctx);
    }

    pub fn toast(&mut self, message: StringOrError, level: ToastLevel) -> &mut Toast {
        let message = self.string_or_error_to_string(message);

        let toast = Toast::custom(
            textwrap::wrap(message.as_str(), 75).join("\n"),
            level
        );

        self.toasts.add(toast)
    }

    pub fn toast_and_log(&mut self, message: StringOrError, level: ToastLevel) -> &mut Toast {
        let log_message = format!(
            "{} Additional Detail: {}",
            self.string_or_error_to_string(message.clone()),
            self.string_or_error_full_error_msg(message.clone())
        );

        match level {
            ToastLevel::Info => log::info!("{}", log_message),
            ToastLevel::Warning => log::warn!("{}", log_message),
            ToastLevel::Error => log::error!("{}", log_message),
            ToastLevel::Success => log::info!("{}", log_message),
            ToastLevel::None => log::info!("{}", log_message),
            ToastLevel::Custom(_, _) => log::info!("{}", log_message),
        }

        self.toast(message, level)
    }

    fn string_or_error_to_string(&self, string_or_error: StringOrError)-> String {
        match string_or_error {
            StringOrError::Error(error) => error.message(),
            StringOrError::String(string) => string,
        }
    }

    fn string_or_error_full_error_msg(&self, string_or_error: StringOrError) -> String {
        match string_or_error {
            StringOrError::Error(error) => {
                match error {
                    Error::FileNotFound(actual_error, _, _) => actual_error.unwrap_or_default(),
                    Error::NoFileSelected(actual_error) => actual_error.unwrap_or_default(),
                    Error::FailedToApplyOptimizations(actual_error, _) => actual_error.unwrap_or_default(),
                    Error::FailedToInitImage(actual_error, _, _) => actual_error.unwrap_or_default(),
                    Error::ImageFormatNotSupported(actual_error, _) => actual_error.unwrap_or_default(),
                    Error::FailedToLoadImage(actual_error, _) => actual_error.unwrap_or_default(),
                }
            },
            StringOrError::String(string) => string,
        }
    }
}

#[derive(Default, Clone)]
pub struct NotifierAPI {
    pub toasts: Arc<Mutex<ToastsManager>>,
    pub loading_status: Arc<RwLock<Option<Loading>>>,
}

// Struct that brings an interface to manage toasts.
impl NotifierAPI {
    pub fn new() -> Self {
        Self {
            toasts: Arc::new(Mutex::new(ToastsManager::new())),
            loading_status: Arc::new(RwLock::new(None)),
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        if let Ok(mut toasts) = self.toasts.try_lock() {
            toasts.update(ctx);
        }
    }

    pub fn set_loading(&mut self, message: Option<String>) {
        *self.loading_status.write().unwrap() = Some(Loading { message })
    }

    pub fn set_loading_and_log(&mut self, message: Option<String>) {
        if let Some(message) = &message {
            info!("{}", message);
        }

        self.set_loading(message);
    }

    pub fn unset_loading(&mut self) {
        *self.loading_status.write().unwrap() = None;
    }
}

#[derive(Clone)]
pub enum StringOrError {
    Error(Error),
    String(String),
}

impl Into<StringOrError> for Error {
    fn into(self) -> StringOrError {
        StringOrError::Error(self)
    }
}

impl Into<StringOrError> for String {
    fn into(self) -> StringOrError {
        StringOrError::String(self)
    }
}

impl Into<StringOrError> for &str {
    fn into(self) -> StringOrError {
        StringOrError::String(self.to_string())
    }
}