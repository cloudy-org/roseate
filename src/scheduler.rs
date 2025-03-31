use std::time::{Duration, Instant};

pub struct Scheduler {
    delay: Duration,
    callback: Box<dyn FnMut()>,
    time_scheduled: Instant,
    pub done: bool
}

impl Scheduler {
    pub fn new(callback: impl FnMut() + 'static, delay: Duration) -> Self {
        Self {
            delay,
            callback: Box::new(callback),
            time_scheduled: Instant::now(),
            done: false
        }
    }

    pub fn update(&mut self) {
        if self.done == true {
            return;
        }

        if self.time_scheduled.elapsed() >= self.delay {
            (self.callback)();
            self.done = true;
        }
    }
}