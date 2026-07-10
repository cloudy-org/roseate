use std::sync::{Arc, Mutex};

pub(super) enum InnerState {
    Idling,
    Decoding,
    Uploading,
}

impl Default for InnerState {
    fn default() -> Self {
        Self::Idling
    }
}

#[derive(Clone, Default)]
pub struct ImageLoaderState {
    pub(super) inner_state: Arc<Mutex<InnerState>>,
    pub(super) load_image_to_gpu: Arc<Mutex<bool>>,
}

impl ImageLoaderState {
    pub fn ready_for_uploading(&self) -> bool {
        match self.load_image_to_gpu.try_lock() {
            Ok(load_image_texture_mutex) => *load_image_texture_mutex,
            Err(_) => false,
        }
    }

    pub fn is_loading(&self) -> bool {
        match self.inner_state.try_lock() {
            Ok(inner_state) => match *inner_state {
                InnerState::Idling => false,
                InnerState::Decoding | InnerState::Uploading => true,
            },
            // umm will this stab me in the back later... *find out next time...*
            Err(_) => true,
        }
    }
}
