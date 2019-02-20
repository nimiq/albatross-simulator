use std::time::Instant;
use std::time::Duration;
use std::ops::{Add, AddAssign, Sub};
use crate::timer::Timer;

/// This struct keeps track of time in our network.
/// Time can be advanced by nodes to simulate processing.
/// There is a single start time at the beginning of the simulation,
/// and time is passed together with events.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    pub(crate) now: Instant,
}

impl Time {
    pub(crate) fn new() -> Self {
        Time {
            now: Instant::now(),
        }
    }

    /// Advances time by a certain duration.
    pub fn advance(&mut self, duration: Duration) {
        self.now += duration;
    }
}

impl Add<Duration> for Time {
    type Output = Time;

    fn add(self, other: Duration) -> Time {
        Time {
            now: self.now + other
        }
    }
}

impl AddAssign<Duration> for Time {
    fn add_assign(&mut self, other: Duration) {
        *self = *self + other;
    }
}

impl Add<Timer> for Time {
    type Output = Time;

    fn add(self, other: Timer) -> Time {
        self + other.elapsed()
    }
}

impl AddAssign<Timer> for Time {
    fn add_assign(&mut self, other: Timer) {
        *self = *self + other;
    }
}

impl Sub<Time> for Time {
    type Output = Duration;

    fn sub(self, other: Time) -> Duration {
        self.now.duration_since(other.now)
    }
}
