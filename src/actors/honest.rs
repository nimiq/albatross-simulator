use simulator::Environment;
use simulator::Event as SimulatorEvent;
use simulator::metrics::Metrics;
use simulator::Node;

use crate::actors::Timing;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::signature::KeyPair;
use crate::protocol::honest_protocol::HonestProtocol;
use crate::protocol::ProtocolConfig;
use crate::simulation::Event;
use crate::simulation::metrics::MetricsEventType;
use crate::simulation::SimulationConfig;

pub struct HonestActor {
    protocol: HonestProtocol,
    simulation_config: SimulationConfig,
}

impl Node for HonestActor {
    type EventType = Event;
    type MetricsEventType = MetricsEventType;

    fn run(&mut self, event: SimulatorEvent<Self::EventType>, mut env: Environment<Self::EventType, Self::MetricsEventType>) -> bool {
        env.note_event(&MetricsEventType::MessageEvent {
            own: env.own_id(),
            event: event.inner().clone(),
            from: event.from(),
        }, event.receive_time());

        match event.inner() {
            // External events.
            Event::Block(block) => self.protocol.received_block(block.clone(), &mut env),
            Event::Transaction(_transaction) => (),

            // PBFT.
            Event::ViewChange(view_change) => self.protocol.handle_view_change(view_change.clone(), &mut env),
            Event::BlockProposal(proposal, signature) => self.protocol.handle_macro_block_proposal(proposal.clone(), signature.clone(), &mut env),
            Event::BlockPrepare(proof) => self.protocol.handle_prepare(proof.clone(), &mut env),
            Event::BlockCommit(proof) => self.protocol.handle_commit(proof.clone(), &mut env),

            // Internal events.
            Event::BlockProcessed(block) => self.protocol.processed_block(block.clone(), &mut env),
            Event::BlockProduced(block) => self.protocol.produced_block(block.clone(), &mut env),
            Event::ProposalProcessed(block, signature) => self.protocol.processed_proposal(block.clone(), signature.clone(), &mut env),
            Event::TransactionProcessed(_transaction) => (),
            Event::MicroBlockTimeout(block_number, view_number) | Event::MacroBlockTimeout(block_number, view_number, _) => self.protocol.handle_timeout(*block_number, *view_number, &mut env),

            Event::Init => self.protocol.prepare_next_block(&mut env),
        }

        // Run for the configured amount of blocks.
        self.protocol.current_block_number() < self.simulation_config.blocks
    }
}

impl HonestActor {
    pub fn new(simulation_config: SimulationConfig,
               protocol_config: ProtocolConfig, timing: Timing,
               genesis_block: MacroBlock, key_pair: KeyPair) -> Self {
        HonestActor {
            protocol: HonestProtocol::new(protocol_config, timing,
                                          genesis_block, key_pair),
            simulation_config,
        }
    }
}
