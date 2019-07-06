#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::io;
use std::time::Duration;

use futures::future::{join_all, lazy, ok};
use futures::prelude::*;
use log::LevelFilter;
use rand::rngs::OsRng;

use simulator::Simulator;

use crate::actors::Timing;
use crate::cmdline::Options;
use crate::logging::AlbatrossDispatch;
use crate::protocol::ProtocolConfig;
use crate::simulation::Event;
use crate::simulation::metrics::DefaultMetrics;
use crate::simulation::network::AdvancedNetwork;
use crate::simulation::settings::ProtocolSettings;
use crate::simulation::settings::Settings;
use crate::simulation::settings::TimingSettings;
use crate::simulation::SimulationConfig;
use crate::simulation::topology_helper::AdvancedTopologyHelper;

pub mod datastructures;
pub mod protocol;
pub mod actors;
pub mod simulation;
pub mod logging;
pub mod distributions;
pub mod cmdline;

fn main() {
    let options = Options::parse().unwrap();

    // Setup logging.
    let mut dispatch = fern::Dispatch::new()
        .pretty_logging(true)
        .chain(
            fern::Dispatch::new()
                .level(LevelFilter::Debug)
                .chain(io::stdout())
        );

    if let Some(ref trace_file) = options.trace_file {
        dispatch = dispatch.chain(
            fern::Dispatch::new()
                .level(LevelFilter::Trace)
                .chain(fern::log_file(trace_file.clone()).unwrap())
        );
    }

    dispatch.apply().unwrap();

    tokio::run(lazy(|| {
        start_simulations(options);
        ok(())
    }))
}

fn start_simulations(options: Options) {
    let mut settings = Settings::from_file(options.network_settings.unwrap()).unwrap();
    let timing = Timing::from_settings(TimingSettings::from_file(options.timing_settings.unwrap()).unwrap());
    let protocol = ProtocolSettings::from_file(options.protocol_settings.unwrap()).unwrap();
    let topology = AdvancedTopologyHelper::from_settings(&mut settings).unwrap();

    // Sequentially run simulations.
    for &num_nodes in options.num_nodes.iter() {
        let mut iterations = Vec::with_capacity(options.iterations);
        for _ in 0..options.iterations {
            let simulation_config = SimulationConfig {
                blocks: options.blocks,
            };
            let protocol_config = ProtocolConfig {
                micro_block_timeout: options.micro_block_timeout.unwrap_or(Duration::from_micros(protocol.micro_block_timeout)),
                macro_block_timeout: options.macro_block_timeout.unwrap_or(Duration::from_micros(protocol.macro_block_timeout)),
                num_micro_blocks: options.num_micro_blocks.unwrap_or(protocol.num_micro_blocks),
                num_validators: num_nodes as u16,
            };

            iterations.push(run_simulation(num_nodes, &topology, simulation_config, protocol_config, timing.clone()).map(|simulator| {
                simulator.metrics().analyze()
            }));
        }
        tokio::spawn(join_all(iterations).map(|_| ()));
    }
}

fn run_simulation(num_nodes: usize, topology: &AdvancedTopologyHelper, simulation_config: SimulationConfig, protocol_config: ProtocolConfig, timing: Timing) -> impl Future<Item=Simulator<AdvancedNetwork, DefaultMetrics>, Error=()> {
    info!("Simulating {} parties Albatross!", num_nodes);
    debug!("Simulation: {:#?}", simulation_config);
    debug!("Protocol: {:#?}", protocol_config);
    debug!("Timing: {:#?}", timing);

    let metrics = DefaultMetrics::default();

    info!("Creating network topology distributions.");

    let mut rng = OsRng::new().unwrap();
    info!("Setting up network.");
    let network = AdvancedNetwork::new(num_nodes, &topology, simulation_config,
                                       protocol_config, timing, &mut rng);

    let mut simulator = Simulator::new(network, metrics);

    simulator.build();

    for i in 0..num_nodes {
        simulator.initial_event(i, Event::Init);
    }

    IntoFuture::into_future(simulator).map(|simulator| {
        info!("Simulation ended, analyzing metrics.");
        simulator
    }).map_err(|_| {
        info!("Simulation ended with error.");
    })
}
