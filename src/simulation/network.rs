use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::Rng;

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
use crate::simulation::topology_helper::AdvancedTopologyHelper;

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

    fn full_transmission_time(&self, from: usize, to: usize, _event: &Event) -> Option<Duration> {
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

pub struct AdvancedNetwork {
    nodes: Vec<NodeConfig>,
    links: Vec<HashMap<usize, LinkConfig>>,
    simulation_config: SimulationConfig,
    protocol_config: ProtocolConfig,
    timing: Timing,
    genesis_block: MacroBlock
}

struct NodeConfig {
    download_bandwidth: f64, // Mbps
    upload_bandwidth: f64, // Mbps
    region: usize,
    connections: Vec<usize>,
}

struct LinkConfig {
    bandwidth: f64, // Mbps
    latency: f64, // ms
}

impl AdvancedNetwork {
    pub(crate) fn new<R: Rng + ?Sized>(num_nodes: usize, topology_helper: &AdvancedTopologyHelper,
                                                        simulation_config: SimulationConfig,
                                                        protocol_config: ProtocolConfig,
                                                        timing: Timing,
                                                        rng: &mut R) -> Self {
        let mut nodes = Vec::new();

        debug!("Create {} nodes.", num_nodes);
        // Assign nodes to regions and estimate their bandwidths.
        for _ in 0..num_nodes {
            let region = topology_helper.nodes_distribution.sample(rng);

            nodes.push(NodeConfig {
                region,
                download_bandwidth: topology_helper.regions[region].download_bandwidth_distribution.sample(rng),
                upload_bandwidth: topology_helper.regions[region].upload_bandwidth_distribution.sample(rng),
                connections: Vec::new(),
            });
        }

        debug!("Select {} validators.", protocol_config.num_validators);
        // Compute first set of validators uniformly at random.
        let mut validators: HashSet<usize> = HashSet::new();
        let uniform_node_distribution = Uniform::new(0, num_nodes);
        while validators.len() < protocol_config.num_validators as usize {
            validators.insert(uniform_node_distribution.sample(rng));
        }

        debug!("Interconnect validators.");
        // Interconnect all validators.
        for &validator_id in validators.iter() {
            for &connection in validators.iter() {
                // Do not connect to oneself.
                if validator_id != connection {
                    nodes[validator_id].connections.push(connection);
                }
            }
        }

        debug!("Sample random connections.");
        // Sample random other connections.
        for node_id in 0..num_nodes {
            let min_connections = if validators.contains(&node_id) {
                topology_helper.min_connections_per_validator
            } else {
                topology_helper.min_connections_per_node
            };

            // Sample random connections and add them.
            // Try at most three times.
            let mut tries = 0;
            while nodes[node_id].connections.len() < min_connections && tries < 3 {
                let connection = uniform_node_distribution.sample(rng);
                tries += 1;

                let max_connections_peer = if validators.contains(&connection) {
                    topology_helper.max_connections_per_validator
                } else {
                    topology_helper.max_connections_per_node
                };

                // Only add connection if:
                // - not oneself
                // - connection does not exist yet
                // - peer still has open slots
                if connection != node_id && !nodes[node_id].connections.contains(&connection)
                    && nodes[connection].connections.len() < max_connections_peer {
                    tries = 0;

                    nodes[node_id].connections.push(connection);
                    nodes[connection].connections.push(node_id);
                }
            }
        }

        debug!("Sample link configuration.");
        // Then sample link configurations.
        let mut links: Vec<HashMap<usize, LinkConfig>> = Vec::new();
        for node_id in 0..num_nodes {
            let mut link_configs = HashMap::new();
            for &peer_id in nodes[node_id].connections.iter() {
                // Only add them once.
                if node_id < peer_id {
                    let bandwidth = f64::min(
                        f64::min(nodes[node_id].upload_bandwidth, nodes[peer_id].download_bandwidth),
                        f64::min(nodes[node_id].download_bandwidth, nodes[peer_id].upload_bandwidth)
                    );

                    let latency = topology_helper.get_latency(
                        nodes[node_id].region,
                        nodes[peer_id].region,
                        rng
                    );

                    link_configs.insert(peer_id, LinkConfig {
                        bandwidth,
                        latency,
                    });
                }
            }
            links.push(link_configs);
        }

        let genesis_block = MacroBlock::create_genesis_block(&validators);

        AdvancedNetwork {
            nodes,
            links,
            simulation_config,
            protocol_config,
            timing,
            genesis_block,
        }
    }
}

impl NetworkConfig for AdvancedNetwork {
    type EventType = Event;
    type MetricsEventType = MetricsEventType;

    fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    fn adjacent(&self, from: usize) -> Cow<Vec<usize>> {
        Cow::Borrowed(&self.nodes[from].connections)
    }

    fn full_transmission_time(&self, from: usize, to: usize, event: &Event) -> Option<Duration> {
        if from != to {
            // We only do a very rough estimation. We assume this is the only packet sent over this link.
            // Also we do not consider splitting the event into packets right now.
            // Thus, the time it takes should equal approximately:
            // size / bandwidth + latency
            let size = (event.byte_size() * 8 /* bits */) as f64;
            let link_config = self.links.get(usize::min(from, to))?.get(&usize::max(from, to))?;
            let bandwidth = link_config.bandwidth * 100 /* Mbps -> bits per ms */ as f64;
            let delay: f64 = size / bandwidth + link_config.latency; // ms
            Some(Duration::from_millis(delay.ceil() as u64))
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
