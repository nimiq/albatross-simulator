use std::collections::binary_heap::BinaryHeap;

use futures::Async;
use futures::Future;
use futures::IntoFuture;
use futures::Stream;

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
    initial_time: Time,
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
            initial_time: Time::new(),
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
        self.queue.push(Event::new(inner, self.initial_time, to, to));
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

    /// Returns the start time of the simulation.
    pub fn initial_time(&self) -> Time {
        self.initial_time
    }
}

impl<N: NetworkConfig, M: Metrics<EventType=N::MetricsEventType>> IntoFuture for Simulator<N, M> {
    type Future = Simulation<N, M>;
    type Item = Self;
    type Error = ();

    /// Transforms the simulator into a future that returns the simulator again upon successful completion.
    fn into_future(mut self) -> Self::Future {
        // Build first if nodes are empty.
        if self.nodes.is_empty() {
            self.build();
        }

        Simulation {
            simulator: Some(self),
        }
    }
}

pub struct Simulation<N: NetworkConfig, M: Metrics<EventType=N::MetricsEventType>> {
    simulator: Option<Simulator<N, M>>,
}

impl<N: NetworkConfig, M: Metrics<EventType=N::MetricsEventType>> Future for Simulation<N, M> {
    type Item = Simulator<N, M>;
    type Error = ();

    /// Runs the stream to completion and returns the simulator again.
    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if let Some(ref mut simulator) = self.simulator {
            loop {
                match try_ready!(simulator.poll()) {
                    Some(_) => (),
                    None => break,
                }
            }
        }

        match self.simulator.take() {
            Some(simulator) => return Ok(Async::Ready(simulator)),
            None => return Err(()),
        }
    }
}

impl<N: NetworkConfig, M: Metrics<EventType=N::MetricsEventType>> Stream for Simulator<N, M> {
    type Item = ();
    type Error = ();

    /// This stream returns one () per event processed.
    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        // Build first if nodes are empty.
        if self.nodes.is_empty() {
            self.build();
        }

        match self.queue.pop() {
            None => Ok(Async::Ready(None)),
            Some(event) => {
                if let Some(recipient) = self.nodes.get_mut(event.to) {
                    let env = Environment::new(event.to,
                                               &self.network_config,
                                               event.receive_time(),
                                               &mut self.queue,
                                               &mut self.metrics);
                    if !recipient.run(event, env) {
                        Ok(Async::Ready(None))
                    } else {
                        Ok(Async::Ready(Some(())))
                    }
                } else {
                    Err(())
                }
            },
        }
    }
}
