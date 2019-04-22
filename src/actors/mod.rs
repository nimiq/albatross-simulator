use std::time::Duration;

use crate::datastructures::block::Block;
use crate::datastructures::block::BlockType;
use crate::datastructures::block::MacroBlock;
use crate::simulation::settings::TimingSettings;

pub mod honest;

#[derive(Clone, Debug)]
pub struct Timing {
    pub signing: Duration,
    pub verification: Duration,
    pub batch_verification: Duration,
    pub generate_aggregate_signature_same_message: Duration,
    pub generate_aggregate_public_key: Duration,
    pub verify_aggregate_signature_same_message: Duration,
    pub generate_aggregate_signature_distinct_message: Duration,
    pub verify_aggregate_signature_distinct_message: Duration,
}

pub trait VerificationTime {
    fn verification_time(&self, timing: &Timing) -> Duration;
}

impl Timing {
    pub(crate) fn from_settings(timing: TimingSettings) -> Self {
        Timing {
            signing: Duration::from_micros(timing.signatures.signing),
            verification: Duration::from_micros(timing.signatures.verification),
            batch_verification: Duration::from_micros(timing.signatures.batch_verification),
            generate_aggregate_signature_same_message: Duration::from_micros(timing.signatures.generate_aggregate_signature_same_message),
            generate_aggregate_public_key: Duration::from_micros(timing.signatures.generate_aggregate_public_key),
            verify_aggregate_signature_same_message: Duration::from_micros(timing.signatures.verify_aggregate_signature_same_message),
            generate_aggregate_signature_distinct_message: Duration::from_micros(timing.signatures.generate_aggregate_signature_distinct_message),
            verify_aggregate_signature_distinct_message: Duration::from_micros(timing.signatures.verify_aggregate_signature_distinct_message),
        }
    }

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
