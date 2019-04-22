use crate::environment::Environment;
use crate::event::Event;

pub trait Node: Send {
    type EventType;
    type MetricsEventType;

    /// Processes an input event and allows to output events to other channels.
    /// The return value determines whether the simulation should continue to run (`true`)
    /// or should terminate (`false`).
    fn run(&mut self, event: Event<Self::EventType>,
           env: Environment<Self::EventType, Self::MetricsEventType>) -> bool;
}
