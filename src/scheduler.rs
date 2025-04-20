use std::time::{Duration, Instant};

pub struct Scheduler<T = ()> {
    delay: Duration,
    callback: Box<dyn FnMut() -> T>,
    time_scheduled: Instant,
    pub done: bool
}

impl<T> Scheduler<T> {
    pub fn new(callback: impl FnMut() -> T + 'static, delay: Duration) -> Self {
        Self {
            delay,
            callback: Box::new(callback),
            time_scheduled: Instant::now(),
            done: false
        }
    }

    pub fn update(&mut self) -> Option<T> {
        if self.done == true {
            return None;
        }

        if self.time_scheduled.elapsed() >= self.delay {
            let return_value = (self.callback)();
            self.done = true;

            return Some(return_value);
        }

        None
    }
}