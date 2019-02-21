use std::time::Duration;

use crate::datastructures::block::Block;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::block::MicroBlock;

pub mod honest;

#[derive(Clone, Debug)]
pub struct Timing {
    pub signature_verification: Duration,
}

impl Timing {
    pub fn block_processing_time(&self, block: &Block) -> Duration {
        Duration::from_millis(100)
    }

    pub fn proposal_processing_time(&self, block: &MacroBlock) -> Duration {
        Duration::from_millis(100)
    }

    pub fn block_production_time(&self, block: &Block) -> Duration {
        Duration::from_millis(100)
    }
}
