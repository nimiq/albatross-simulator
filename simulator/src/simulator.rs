use std::collections::binary_heap::BinaryHeap;

use crate::environment::Environment;
use crate::Event;
use crate::metrics::Metrics;
use crate::network::NetworkConfig;
use crate::node::Node;
use crate::Time;
use crate::UniqueId;

pub struct Simulator<N: NetworkConfig, M: Metrics<EventType=N::MetricsEventType>> {
    network_config: N,
    metrics: M,
    nodes: Vec<Box<Node<EventType=N::EventType, MetricsEventType=N::MetricsEventType>>>,
    queue: BinaryHeap<Event<N::EventType>>,
}

impl<N: NetworkConfig, M: Metrics<EventType=N::MetricsEventType>> Simulator<N, M> {
    /// Creates a new simulator.
    pub fn new(network_config: N,
               metrics: M) -> Self {
        Simulator {
            nodes: Vec::with_capacity(network_config.num_nodes()),
            network_config,
            metrics,
            queue: BinaryHeap::new(),
        }
    }

    /// Creates a future that will run the simulator.
    pub fn build(&mut self) {
        // Build only once.
        if !self.nodes.is_empty() {
            return;
        }

        let num_nodes = self.network_config.num_nodes();

        // Setup nodes first.
        info!("Setting up {} nodes.", num_nodes);
        for i in 0..num_nodes {
            let node = self.network_config.node(i);
            self.nodes.push(node);
        }

        info!("Finished setup.");
    }

    /// Sends an initial event to a node.
    pub fn initial_event(&mut self, to: UniqueId, inner: N::EventType) {
        self.queue.push(Event::new(inner, Time::new(), to, to));
    }

    /// Runs the simulation.
    pub fn run(&mut self) -> bool {
        // Build first if nodes are empty.
        if self.nodes.is_empty() {
            self.build();
        }

        while let Some(event) = self.queue.pop() {
            if let Some(recipient) = self.nodes.get_mut(event.to) {
                let env = Environment::new(event.to,
                                           &self.network_config,
                                           event.receive_time(),
                                           &mut self.queue,
                                           &mut self.metrics);
                if !recipient.run(event, env) {
                    break;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Returns access to the collected metrics.
    pub fn metrics(&self) -> &M {
        &self.metrics
    }
}
