use simulator::Environment;
use simulator::Time;

use crate::actors::MetricsEventType;
use crate::datastructures::block::{BlockType, MicroBlock};
use crate::protocol::Protocol;
use crate::simulation::Event;
use crate::datastructures::pbft::ViewChangeInternals;
use crate::datastructures::signature::AggregatePublicKey;
use crate::actors::Timing;
use crate::datastructures::block::Block;
use crate::datastructures::pbft::ViewChange;
use std::collections::HashSet;

#[derive(Debug)]
pub enum MicroBlockError {
    InvalidBlockType,
    InvalidBlockNumber,
    InvalidBlockProducer,
    InvalidSignature,
    MissingViewChangeMessages,
    InvalidViewChangeMessages,
    OldViewChangeNumber,
}

pub struct MicroBlockState {
    pub view_change_number: u16,
    pub view_change_messages: HashSet<ViewChange>,
}

pub trait MicroBlockProtocol: Protocol {
    fn state(&self) -> &MicroBlockState;
    fn state_mut(&mut self) -> &mut MicroBlockState;
    fn produce_micro_block(&mut self) -> Option<MicroBlock>;

    fn received_block(&mut self, block: MicroBlock, mut env: Environment<Event, MetricsEventType>) {
        // We verify here already to allow for different processing times depending on the verification result.
        let result = self.verify_block(&block);

        if let Err(ref e) = result {
            warn!("Got invalid block, reason {:?}", e);
        }

        let block = Block::Micro(block);
        let processing_time = env.time() + self.timing().block_processing_time(&block);
        env.schedule_self(Event::BlockProcessed(block, result.is_ok()), processing_time);
    }

    fn processed_block(&mut self, block: MicroBlock, valid: bool, mut env: Environment<Event, MetricsEventType>) {
        if valid {
            self.store_block(Block::Micro(block.clone()));

            // Now check who the next block producer is.
            if &self.next_block_producer(0) == self.own_public_key() {
                self.produce_block();
            } else {
                // Set a timeout.
                let delay = self.protocol_config().block_timeout * (self.state().view_change_number + 1).into();
                env.schedule_self(Event::Timeout(self.next_block_number(), self.state().view_change_number), env.time() + delay);
            }
        } else {
            // Ignore block.
        }
    }

    fn handle_timeout(&mut self, block_number: u32, view_number: u16, mut env: Environment<Event, MetricsEventType>) {
        // Check whether timeout was triggered and no new block has been accepted in the meanwhile.
        if self.next_block_number() == block_number && self.state().view_change_number == view_number {
            self.state_mut().view_change_messages.clear();

            let view_change = ViewChange::new(block_number, view_number + 1, self.own_secret_key());
            self.state_mut().view_change_messages.insert(view_change.clone());
            self.broadcast_to_validators(Event::ViewChange(view_change), env);
        }
    }

    fn handle_view_change(&mut self, view_change: ViewChange, mut env: Environment<Event, MetricsEventType>) {
        // Validate view change message:
        // Should be for current block and next view number.
        if view_change.internals.block_number != self.next_block_number()
            || view_change.internals.new_view_number != self.state().view_change_number + 1 {
            return;
        }

        self.state_mut().view_change_messages.insert(view_change.clone());

        // When 2f + 1 view change messages have been received,
        // commit to not accepting a block from the previous owner anymore.
        if self.state().view_change_messages.len() > self.protocol_config().two_third_threshold().into() {
            self.state_mut().view_change_number += 1;

            let delay = self.protocol_config().block_timeout * (self.state().view_change_number + 1).into();
            env.schedule_self(Event::Timeout(self.next_block_number(), self.state().view_change_number), env.time() + delay);

            // TODO: Check whether we are the next block producer.
        }
    }

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
            // Check whether we committed not to accept blocks from this view change number.
            if block.header.digest.view_number < self.state().view_change_number {
                return Err(MicroBlockError::OldViewChangeNumber);
            }

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
