use std::borrow::Cow;
use std::collections::binary_heap::BinaryHeap;
use std::time::Duration;

use crate::event::Event;
use crate::Metrics;
use crate::NetworkConfig;
use crate::Time;
use crate::unique_id::UniqueId;

pub struct Environment<'a, E, ME> {
    network_config: &'a NetworkConfig<EventType=E, MetricsEventType=ME>,
    metrics: &'a mut Metrics<EventType=ME>,
    queue: &'a mut BinaryHeap<Event<E>>,
    own_id: UniqueId,
    time: Time,
}

impl<'a, E, ME> Environment<'a, E, ME> {
    #[inline]
    pub(crate) fn new(own_id: UniqueId, config: &'a NetworkConfig<EventType=E, MetricsEventType=ME>, time: Time,
                      queue: &'a mut BinaryHeap<Event<E>>,
                      metrics: &'a mut Metrics<EventType=ME>) -> Self {
        Environment {
            own_id,
            network_config: config,
            time,
            queue,
            metrics,
        }
    }

    /// Returns a slice of peers this node has.
    #[inline]
    pub fn peers(&self) -> Cow<Vec<UniqueId>> {
        self.network_config.adjacent(self.own_id)
    }

    /// Sends an event to another peer at the current time.
    /// The latency will be added automatically.
    /// Returns `true` on success and `false` on error (e.g. if no link has been found).
    #[inline]
    pub fn send_to(&mut self, to: UniqueId, event: E) -> bool {
        self.schedule(to, event, self.time)
    }

    /// Sends a scheduled event to another peer.
    /// The latency will be added automatically.
    /// Returns `true` on success and `false` on error (e.g. if no link has been found).
    pub fn schedule(&mut self, to: UniqueId, event: E, scheduled_send_time: Time) -> bool {
        if let Some(delay) = self.network_config.transmission_delay(self.own_id, to, &event) {
            let e = Event::new(event,
                               scheduled_send_time + delay, self.own_id, to);
            self.queue.push(e);
            true
        } else {
            false
        }
    }

    /// Schedules an event executed by the same peer at a later time.
    /// Simulates processing or timeouts.
    pub fn schedule_self(&mut self, event: E, scheduled_time: Time) {
        let e = Event::new(event,
                           scheduled_time, self.own_id, self.own_id);
        self.queue.push(e);
    }

    /// Returns the current time.
    #[inline]
    pub fn time(&self) -> Time {
        self.time
    }

    /// Advances time on the clock.
    #[inline]
    pub fn advance_time(&mut self, duration: Duration) {
        self.time.advance(duration);
    }

    /// Returns the own id.
    #[inline]
    pub fn own_id(&self) -> UniqueId {
        self.own_id
    }
}

impl<'a, E: Clone, K> Environment<'a, E, K> {
    /// Sends an event to all connected peers.
    #[inline]
    pub fn broadcast(&mut self, event: E) {
        self.scheduled_broadcast(event, self.time)
    }

    /// Sends a scheduled event to all connected peers.
    #[inline]
    pub fn scheduled_broadcast(&mut self, event: E, scheduled_send_time: Time) {
        for channel in self.network_config.adjacent(self.own_id).iter() {
            self.schedule(*channel, event.clone(), scheduled_send_time);
        }
    }
}

impl<'a, E, ME> Metrics for Environment<'a, E, ME> {
    type EventType = ME;

    #[inline]
    fn note_event(&mut self, event: &ME, time: Time) {
        self.metrics.note_event(event, time)
    }
}
