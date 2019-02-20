use simulator::Environment;
use simulator::Time;

use crate::actors::MetricsEventType;
use crate::datastructures::block::{BlockType, MicroBlock};
use crate::protocol::Protocol;
use crate::simulation::Event;
use crate::datastructures::pbft::ViewChangeInternals;
use crate::datastructures::signature::AggregatePublicKey;

pub enum MicroBlockError {
    InvalidBlockType,
    InvalidBlockNumber,
    InvalidBlockProducer,
    InvalidSignature,
    MissingViewChangeMessages,
    InvalidViewChangeMessages,
}

pub struct MicroBlockState {
    pub view_change_number: u16,
}

pub trait MicroBlockProtocol: Protocol {
    fn current_view_change_number(&self) -> u16;

    fn handle_block(&mut self, block: MicroBlock, env: Environment<Event, MetricsEventType>) {
    }
    fn produce_block(&mut self, time: Time) -> MicroBlock;

    fn verify_block(&self, block: &MicroBlock) -> Result<(), MicroBlockError> {
        if self.next_block_type() != BlockType::Micro {
            return Err(MicroBlockError::InvalidBlockType);
        }

        // Check Digest.
        if self.next_block_number() != block.header.digest.block_number {
            return Err(MicroBlockError::InvalidBlockNumber);
        }

        if self.next_block_producer(block.header.digest.view_number) != block.header.digest.validator {
            return Err(MicroBlockError::InvalidBlockProducer);
        }

        // Check Signature.
        if !block.justification.verify(&block.header.digest.validator, &block.header) {
            return Err(MicroBlockError::InvalidSignature);
        }

        // Check view change.
        if block.header.digest.view_number > 0 {
            if let Some(ref view_change_proof) = block.extrinsics.view_change_messages {
                let expected_message = ViewChangeInternals {
                    block_number: block.header.digest.block_number,
                    new_view_number: block.header.digest.view_number,
                };
                let keys = self.get_validators(&view_change_proof.public_key_bitmap);
                let aggregate_key = AggregatePublicKey::from(keys);
                if !view_change_proof.signatures.verify_single(&aggregate_key, &expected_message) {
                    return Err(MicroBlockError::InvalidViewChangeMessages);
                }
            } else {
                return Err(MicroBlockError::MissingViewChangeMessages);
            }
        }

        // TODO: Check timestamp
        // TODO: Check transactions
        // TODO: Check slash inherents
        // TODO: Check Merkle hashes

        Ok(())
    }
}
