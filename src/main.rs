#[macro_use]
extern crate log;

use std::io;
use std::time::Duration;

use log::LevelFilter;

use simulator::Simulator;

use crate::actors::Timing;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::block::MacroDigest;
use crate::datastructures::block::MacroExtrinsics;
use crate::datastructures::block::MacroHeader;
use crate::datastructures::hash::Hash;
use crate::datastructures::signature::KeyPair;
use crate::logging::AlbatrossDispatch;
use crate::protocol::ProtocolConfig;
use crate::simulation::Event;
use crate::simulation::metrics::DefaultMetrics;
use crate::simulation::network::SimpleNetwork;
use crate::simulation::SimulationConfig;

pub mod datastructures;
pub mod protocol;
pub mod actors;
pub mod simulation;
pub mod logging;

fn main() {
    // Setup logging.
    fern::Dispatch::new()
        .pretty_logging(false)
        .level(LevelFilter::Info)
        .chain(io::stdout())
        .apply().unwrap();

    let num_nodes = 3usize;
    let delay = Duration::from_millis(500);
    let simulation_config = SimulationConfig {
        blocks: 50,
    };
    let protocol_config = ProtocolConfig {
        micro_block_timeout: Duration::from_secs(2),
        macro_block_timeout: Duration::from_secs(4),
        num_micro_blocks: 4,
        num_validators: num_nodes as u16,
    };
    let timing = Timing {
        signature_verification: Duration::default(),
    };

    // Create genesis block.
    let digest = MacroDigest {
        validators: (0..num_nodes).map(|i| KeyPair::from_id(i as u64).public_key()).collect(),
        block_number: 0,
        view_number: 0,
        parent_macro_hash: Hash::default(),
    };

    let seed = KeyPair::from_id(num_nodes as u64)
        .secret_key()
        .sign(&Hash::default());
    let extrinsics = MacroExtrinsics {
        timestamp: 0,
        seed,
        view_change_messages: None,
    };

    let header = MacroHeader {
        parent_hash: Hash::default(),
        digest,
        extrinsics_root: extrinsics.hash(),
        state_root: Hash::default(), // TODO: Simulate stake.
    };

    let genesis_block = MacroBlock {
        header,
        extrinsics,
        justification: None, // Only block without justification.
    };

    info!("Simulating {} parties Albatross!", num_nodes);
    debug!("Delay: {:#?}", delay);
    debug!("Simulation: {:#?}", simulation_config);
    debug!("Protocol: {:#?}", protocol_config);
    debug!("Timing: {:#?}", timing);
    debug!("Genesis block: {:#?}", genesis_block);

    let metrics = DefaultMetrics::default();
    // Setting this to true will cause the program to take much longer with the same result.
    let network = SimpleNetwork::new(num_nodes, delay,
                                         simulation_config, protocol_config,
                                         timing, genesis_block);

    let mut simulator = Simulator::new(network, metrics);

    simulator.build();

    for i in 0..num_nodes {
        simulator.initial_event(i, Event::Init);
    }

    simulator.run();

    info!("Simulation ended, analyzing metrics.");

    simulator.metrics().analyze();
}
