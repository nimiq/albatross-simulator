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
    fn adjacent(&self, from: UniqueId) -> &[UniqueId];

    /// Returns the latency for a link if it exists, None otherwise.
    /// Links are not duplex by default!
    ///
    /// Moreover, channels can be shared via events and thus new channels can be created.
    /// Their latency must be set via the channel's `set_latency` method.
    fn latency(&self, from: UniqueId, to: UniqueId) -> Option<Duration>;

    /// Returns the behavior for a node.
    fn node(&self, id: UniqueId) -> Box<Node<EventType=Self::EventType, MetricsEventType=Self::MetricsEventType>>;
}
