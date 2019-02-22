use std::time::Duration;

use crate::datastructures::block::Block;
use crate::datastructures::block::BlockType;
use crate::datastructures::block::MacroBlock;

pub mod honest;

#[derive(Clone, Debug)]
pub struct Timing {
    pub signature_verification: Duration,
}

impl Timing {
    pub fn block_processing_time(&self, block: &Block) -> Duration {
        match block.block_type() {
            BlockType::Macro => Duration::from_millis(200),
            BlockType::Micro => Duration::from_millis(10),
        }
    }

    pub fn proposal_processing_time(&self, _block: &MacroBlock) -> Duration {
        Duration::from_millis(10)
    }

    pub fn block_production_time(&self, _block: &Block) -> Duration {
        Duration::from_millis(10)
    }
}
