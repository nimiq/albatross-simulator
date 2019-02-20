use crate::datastructures::block::MicroBlock;

pub trait MicroBlockProtocol {
    fn handle_block(&mut self, block: MicroBlock);
    fn produce_block(&mut self) -> MicroBlock;

}
