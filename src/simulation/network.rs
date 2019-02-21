use std::borrow::Cow;
use std::time::Duration;

use simulator::NetworkConfig;
use simulator::Node;

use crate::actors::honest::HonestActor;
use crate::actors::Timing;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::signature::KeyPair;
use crate::protocol::ProtocolConfig;
use crate::simulation::Event;
use crate::simulation::metrics::MetricsEventType;
use crate::simulation::SimulationConfig;

/// A small and fully connected network of honest nodes.
pub struct SimpleNetwork {
    num_nodes: usize,
    delay: Duration,
    simulation_config: SimulationConfig,
    protocol_config: ProtocolConfig,
    timing: Timing,
    genesis_block: MacroBlock
}

impl SimpleNetwork {
    pub fn new(num_nodes: usize, delay: Duration,
               simulation_config: SimulationConfig,
               protocol_config: ProtocolConfig, timing: Timing,
               genesis_block: MacroBlock) -> Self {
        SimpleNetwork {
            num_nodes,
            delay,
            simulation_config,
            protocol_config,
            timing,
            genesis_block,
        }
    }
}

impl NetworkConfig for SimpleNetwork {
    type EventType = Event;
    type MetricsEventType = MetricsEventType;

    fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    fn adjacent(&self, from: usize) -> Cow<Vec<usize>> {
        Cow::Owned((0..self.num_nodes).filter(|i| *i != from).collect::<Vec<usize>>())
    }

    fn transmission_delay(&self, from: usize, to: usize, _event: &Event) -> Option<Duration> {
        if from != to {
            Some(self.delay)
        } else {
            None
        }
    }

    fn node(&self, id: usize) -> Box<Node<EventType=Self::EventType, MetricsEventType=Self::MetricsEventType>> {
        Box::new(HonestActor::new(self.simulation_config.clone(),
                                  self.protocol_config.clone(), self.timing.clone(),
                                  self.genesis_block.clone(), KeyPair::from_id(id as u64 )))
    }
}