use crate::datastructures::block::{MicroBlock, BlockType};
use simulator::Time;
use crate::protocol::Protocol;

pub struct MicroBlockState {
    pub view_change_number: u16,
}

pub trait MicroBlockProtocol: Protocol {
    fn handle_block(&mut self, block: MicroBlock, time: Time);
    fn produce_block(&mut self, time: Time) -> MicroBlock;
    fn verify_block(&self, block: &MicroBlock) -> bool;
}
