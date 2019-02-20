use crate::datastructures::block::MacroBlock;
use simulator::Time;
use crate::simulation::Event;

pub struct MacroBlockState {
    view_change_number: u16,
    proposal: Option<MacroBlock>,
    prepares: Vec<()>,
    commits: Vec<()>,
}

pub trait MacroBlockProtocol {
    fn produce_block(&mut self, time: Time) -> MacroBlock;
    fn prepare(&mut self, block: MacroBlock, time: Time);
    fn commit(&mut self, block: MacroBlock, time: Time);
    fn verify_block(&self, block: &MacroBlock) -> bool;
}

