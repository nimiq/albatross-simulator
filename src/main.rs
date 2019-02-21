#[macro_use]
extern crate log;

use std::time::Duration;
use crate::simulation::SimulationConfig;
use crate::protocol::ProtocolConfig;
use crate::actors::Timing;
use crate::datastructures::block::MacroDigest;
use crate::datastructures::block::MacroExtrinsics;
use crate::datastructures::block::MacroHeader;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::signature::KeyPair;
use crate::datastructures::hash::Hash;
use crate::simulation::metrics::DefaultMetrics;
use crate::simulation::network::SimpleNetwork;
use simulator::Simulator;
use crate::simulation::Event;

pub mod datastructures;
pub mod protocol;
pub mod actors;
pub mod simulation;

fn main() {
    simple_logger::init().unwrap();

    let num_nodes = 3usize;
    let delay = Duration::from_millis(200);
    let simulation_config = SimulationConfig {
        blocks: 20,
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
    let mut network = SimpleNetwork::new(num_nodes, delay,
                                         simulation_config, protocol_config,
                                         timing, genesis_block);

    let mut simulator = Simulator::new(network, metrics);

    simulator.build();

    for i in 0..num_nodes {
        simulator.initial_event(i, Event::Init);
    }

    simulator.run();

    info!("Simulation ended, analyzing metrics.");
//
//    let events = &simulator.metrics().events;
//
//    let mut pings = HashMap::new();
//    let mut pongs = HashMap::new();
//    for (event, time) in events.iter() {
//        match event {
//            PingPongMetrics::Ping(i) => {
//                pings.insert(*i, *time);
//            },
//            PingPongMetrics::Pong(i, from) => {
//                pongs.insert(*i, (*time, *from));
//            },
//            _ => (),
//        }
//    }
//
//    // Average round trip time.
//    let mut sorted = HashMap::new();
//    for (i, ping) in pings.iter() {
//        let pong = pongs.get(i);
//        if let Some((pong, from)) = pong {
//            let duration = *pong - *ping;
//            sorted.entry(from)
//                .or_insert_with(Vec::new)
//                .push(duration);
//        }
//    }
//
//    let mut total_duration = 0;
//    let mut total_count = 0;
//    for (k, v) in sorted.iter() {
//        let duration: Duration = v.iter().sum();
//        let duration = duration_to_millis(duration);
//        total_duration += duration;
//        total_count += v.len();
//        println!("Average round trip time for 0 - {} was: {}ms", k, (duration as f64 / v.len() as f64));
//    }
//
//    println!("Average round trip time was: {}ms", (total_duration as f64 / total_count as f64));
}
