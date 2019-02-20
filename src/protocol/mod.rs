use crate::datastructures::block::BlockType;
use crate::datastructures::signature::PublicKey;
use crate::datastructures::block::Block;
use simulator::Environment;
use crate::simulation::Event;

pub mod macro_block;
pub mod micro_block;

pub struct ProtocolConfig {
    pub num_micro_blocks: u32,
}

pub trait Protocol {
    fn next_block_type(&self) -> BlockType;
    fn next_block_producer(&self, view_change_number: u16) -> PublicKey;
    fn next_block_number(&self) -> u32;
    fn handle_block(&mut self, event: Event, env: Environment<Event, ()>);
    fn get_validators(&self, bitmap: &[u16]) -> Vec<PublicKey>;
}
