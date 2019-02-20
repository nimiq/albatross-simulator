use simulator::Environment;
use simulator::Event as SimulatorEvent;
use simulator::Node;
use simulator::Time;

use crate::actors::MetricsEventType;
use crate::actors::Timing;
use crate::datastructures::block::Block;
use crate::datastructures::block::BlockType;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::signature::PublicKey;
use crate::protocol::macro_block::MacroBlockProtocol;
use crate::protocol::micro_block::MicroBlockProtocol;
use crate::protocol::Protocol;
use crate::protocol::ProtocolConfig;
use crate::simulation::Event;
use crate::simulation::SimulationConfig;

pub struct HonestActor {
    protocol_config: ProtocolConfig,
    timing: Timing,
    last_macro_block: Block,
    last_block: Block,
    simulation_config: SimulationConfig,
}

impl Node for HonestActor {
    type EventType = Event;
    type MetricsEventType = MetricsEventType;

    fn run(&mut self, event: SimulatorEvent<Self::EventType>, env: Environment<Self::EventType, Self::MetricsEventType>) -> bool {
//        match event.inner() {
//            // External events.
//            event @ Event::Block(_) => (),
//            Event::Transaction(transaction) => (),
//            Event::ViewChange(view_change) => (),
//
//            // Internal events.
//            Event::BlockProcessed(block, valid) => (),
//            Event::BlockProduced(block) => (),
//            Event::TransactionProcessed(transaction) => (),
//            Event::Timeout => (),
//            _ => (),
//        }

        // Run for the configured amount of blocks.
        self.last_block.block_number() < self.simulation_config.blocks
    }
}
