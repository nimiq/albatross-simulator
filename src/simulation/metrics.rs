use std::fmt;

use simulator::{Metrics, Time};

use crate::simulation::Event;

#[derive(Debug, Clone)]
pub struct MetricsEventType {
    pub own: usize,
    pub from: usize,
    pub event: Event,
}

impl fmt::Display for MetricsEventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} {} from {}", self.own, self.event, self.from)
    }
}

/// A default metrics implementation.
pub struct DefaultMetrics<ME: Clone + fmt::Display> {
    pub events: Vec<(ME, Time)>,
}

impl<ME: Clone + fmt::Display> Metrics for DefaultMetrics<ME> {
    type EventType = ME;

    fn note_event(&mut self, event: &ME, time: Time) {
        trace!("Event {}", event);
        self.events.push((event.clone(), time));
    }
}

impl<ME: Clone + fmt::Display> Default for DefaultMetrics<ME> {
    fn default() -> Self {
        DefaultMetrics {
            events: Vec::new(),
        }
    }
}
