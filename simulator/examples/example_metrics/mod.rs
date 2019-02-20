use simulator::{Metrics, Time};

/// A default metrics implementation.
pub struct DefaultMetrics<ME: Clone> {
    pub events: Vec<(ME, Time)>,
}

impl<ME: Clone> Metrics for DefaultMetrics<ME> {
    type EventType = ME;

    fn note_event(&mut self, event: &ME, time: Time) {
        self.events.push((event.clone(), time));
    }
}

impl<ME: Clone> Default for DefaultMetrics<ME> {
    fn default() -> Self {
        DefaultMetrics {
            events: Vec::new(),
        }
    }
}
