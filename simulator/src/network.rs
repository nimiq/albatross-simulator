use std::borrow::Cow;
use std::time::Duration;

use crate::node::Node;
use crate::unique_id::UniqueId;

pub trait NetworkConfig {
    type EventType;
    type MetricsEventType;

    /// Returns the number of nodes.
    fn num_nodes(&self) -> UniqueId;

    /// Returns the adjacent nodes.
    /// Links are not duplex by default!
    fn adjacent(&self, from: UniqueId) -> Cow<Vec<UniqueId>>;

    /// Returns the delay for an event sent over a link if it exists, None otherwise.
    /// Links are not duplex by default!
    ///
    /// This is used to account for latency and transmission time.
    fn transmission_delay(&self, from: UniqueId, to: UniqueId, event: &Self::EventType) -> Option<Duration>;

    /// Returns the behavior for a node.
    fn node(&self, id: UniqueId) -> Box<Node<EventType=Self::EventType, MetricsEventType=Self::MetricsEventType>>;
}
