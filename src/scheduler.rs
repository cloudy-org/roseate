use std::time::{Duration, Instant};

pub struct Scheduler {
    delay: Duration,
    callback: (),
    time_scheduled: Instant,
}

impl Scheduler {
    pub fn new(callback: (), delay: Duration) -> Self {
        Self {
            delay,
            callback,
            time_scheduled: Instant::now()
        }
    }

    pub fn update(self) {
        // TODO: finish this
    }
}