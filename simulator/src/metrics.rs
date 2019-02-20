use crate::Time;

/// Trait that a Metrics Collector needs to define.
/// This can be used to assemble and collect any metrics about the system.
/// All events have to be reported manually by the node.
pub trait Metrics {
    type EventType;

    /// Notes an event.
    fn note_event(&mut self, event: &Self::EventType, time: Time);
}
