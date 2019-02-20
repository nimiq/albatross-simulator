use std::time::Instant;
use std::time::Duration;

pub struct Timer {
    start_time: Option<Instant>,
    duration: Duration,
}

impl Timer {
    /// Creates a timer that has already been started
    /// and can be used to measure real time processing.
    pub fn new() -> Self {
        Timer {
            start_time: Some(Instant::now()),
            duration: Duration::default(),
        }
    }

    /// Creates a timer without starting it.
    /// This is useful for simulating processing.
    pub fn new_stopped() -> Self {
        Timer {
            start_time: None,
            duration: Duration::default(),
        }
    }

    /// Advance the timer without running it.
    pub fn advance(&mut self, duration: Duration) {
        self.duration += duration;
    }

    /// Starts a timer if it isn't started yet.
    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
    }

    /// Temporarily stops a timer.
    pub fn stop(&mut self) {
        if let Some(start_time) = self.start_time.take() {
            self.duration = start_time.elapsed();
        }
    }

    /// Returns the time elapsed during the runtime of this timer.
    pub fn elapsed(&self) -> Duration {
        match self.start_time {
            Some(ref start_time) => self.duration + start_time.elapsed(),
            None => self.duration,
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
