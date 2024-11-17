use eframe::egui::Context;
use egui_notify::{Toast, ToastLevel, Toasts};

use crate::error::Error;

#[derive(Clone)]
pub enum StringOrError {
    Error(Error),
    String(String),
}

impl Into<StringOrError> for Error {
    fn into(self) -> StringOrError {
        StringOrError::String(self.message())
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

pub struct ToastsManager {
    pub toasts: Toasts,
}

// Struct that brings an interface to manage toasts.
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
        let log_message = self.string_or_error_to_string(message.clone());

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
}