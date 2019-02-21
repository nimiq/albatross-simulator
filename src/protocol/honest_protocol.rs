use std::cmp::Ordering;

use simulator::Environment;

use crate::actors::MetricsEventType;
use crate::actors::Timing;
use crate::datastructures::block::Block;
use crate::datastructures::block::BlockType;
use crate::datastructures::block::MicroBlock;
use crate::datastructures::pbft::{ViewChange, ViewChangeInternals, get_validators};
use crate::datastructures::signature::{KeyPair, PublicKey, AggregatePublicKey};
use crate::datastructures::slashing::SlashInherent;
use crate::protocol::BlockError;
use crate::protocol::macro_block::{MacroBlockState, MacroBlockPhase};
use crate::protocol::ProtocolConfig;
use crate::protocol::ViewChangeState;
use crate::simulation::Event;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::signature::Signature;
use crate::datastructures::hash::Hash;
use crate::datastructures::block::MacroHeader;
use crate::datastructures::pbft::PbftProof;
use crate::datastructures::pbft::PbftJustification;
use crate::datastructures::pbft::AggregateProof;

pub struct HonestProtocol {
    protocol_config: ProtocolConfig,
    timing: Timing,
    view_change_state: ViewChangeState,
    macro_block_state: MacroBlockState,
    chain: Vec<Block>,
    key_pair: KeyPair,
    validators: Vec<PublicKey>,
    processing_block: bool, // Do not process blocks concurrently.
}

impl HonestProtocol {
    /// Returns the next block number.
    fn next_block_number(&self) -> u32 {
        self.chain.len() as u32
    }

    /// Last macro block number.
    fn last_macro_block(&self) -> u32 {
        let current_block_number = self.chain.len() as u32 - 1;
        current_block_number - (current_block_number % (self.protocol_config.num_micro_blocks + 1 /*macro block*/))
    }

    /// Block type at a given number.
    fn block_type_at(&self, block_number: u32) -> BlockType {
        let last_block = self.chain.last().expect("Empty chain");
        if (last_block.block_number() + 1 /*next block*/) % (self.protocol_config.num_micro_blocks + 1 /*macro block*/) == 0 {
            BlockType::Macro
        } else {
            BlockType::Micro
        }
    }

    /// Stores a block in the chain without any additional verifications.
    /// This method only has some basic assertions to ensure correctness of the implementation.
    fn store_block(&mut self, block: Block) {
        let block_number = block.block_number();
        // Do not allow orphan blocks.
        assert!(block_number <= self.chain.len() as u32);

        // Revert chain until len == block_number
        while block_number < self.chain.len() as u32 {
            let block = self.chain.pop();
            // Macro blocks cannot be forked.
            assert_ne!(block.map(|b| b.block_type()), Some(BlockType::Macro));
        }

        self.chain.push(block);

        self.view_change_state.reset();
        self.macro_block_state.reset();
    }

    /// Prepare protocol for next block:
    /// Check if we are the next block producer.
    /// If so, produce a block, otherwise set a timeout.
    fn prepare_next_block(&mut self, mut env: Environment<Event, MetricsEventType>) {
        let next_producer = false; // TODO
        if next_producer {
//            self.produce_block() TODO
        } else {
            // Set a timeout.
            match self.block_type_at(self.next_block_number()) {
                BlockType::Micro => {
                    let delay = self.protocol_config.micro_block_timeout * (self.view_change_state.view_number + 1).into();
                    env.schedule_self(Event::MicroBlockTimeout(self.next_block_number(), self.view_change_state.view_number), env.time() + delay);
                },
                BlockType::Macro => {
                    let delay = self.protocol_config.macro_block_timeout * (self.view_change_state.view_number + 1).into();
                    env.schedule_self(Event::MacroBlockTimeout(self.next_block_number(), self.view_change_state.view_number, self.macro_block_state.phase), env.time() + delay);
                },
            }
        }
    }

    /// A block has been received, simulate processing.
    fn received_block(&self, block: Block, mut env: Environment<Event, MetricsEventType>) {
        let processing_time = env.time() + self.timing.block_processing_time(&block);
        env.schedule_self(Event::BlockProcessed(block), processing_time);
    }

    /// A block has been processed, ensure its validity.
    /// If it is invalid, ignore it.
    /// If it is valid, store block and reset state.
    fn processed_block(&mut self, block: Block, mut env: Environment<Event, MetricsEventType>) {
        // We verify the block.
        let result = self.verify_block(&block);

        // TODO: Handle slashing (we currently do not store the headers of known blocks).

        if let Err(ref e) = result {
            warn!("Got invalid block, reason {:?}", e);
        }

        if result.is_ok() {
            self.store_block(block.clone());

            // Relay block.
            self.relay(Event::Block(block), &mut env);

            self.prepare_next_block(env);
        } else {
            // Ignore block.
        }
    }

    /// Called when a timeout has been triggered.
    /// Check whether a corresponding (valid) block has been received in the meantime.
    /// If not, prepare and send out view change message.
    fn handle_timeout(&mut self, block_number: u32, view_number: u16, mut env: Environment<Event, MetricsEventType>) {
        // Check whether timeout was triggered and no new block has been accepted in the meanwhile.
        if self.next_block_number() == block_number && self.view_change_state.view_number == view_number {
            // Send and process view change message.
            let view_change = ViewChange::new(block_number, view_number + 1, &self.key_pair.secret_key());
            self.multicast_to_validators(Event::ViewChange(view_change.clone()), &mut env);

            // Handle own message exactly like others.
            self.handle_view_change(view_change, env);
        }
    }

    /// Called when a view change message has been received.
    /// First check validity, then add message.
    /// If we received enough view change messages for this view change number,
    /// stop accepting blocks for this number and move on.
    /// In this case, also check for next block producer or start timeout.
    fn handle_view_change(&mut self, view_change: ViewChange, mut env: Environment<Event, MetricsEventType>) {
        // Validate view change message:
        // Should be for current block and have a valid signature.
        if view_change.internals.block_number != self.next_block_number()
            || !view_change.verify() {
            return;
        }

        self.view_change_state.add_message(view_change);

        // When 2f + 1 view change messages have been received,
        // commit to not accepting a block from the previous owner anymore.
        if self.view_change_state.num_messages(self.view_change_state.view_number + 1) > self.protocol_config.two_third_threshold() {
            self.view_change_state.view_number += 1;

            let delay = self.protocol_config.micro_block_timeout * (self.view_change_state.view_number + 1).into();
            env.schedule_self(Event::MicroBlockTimeout(self.next_block_number(), self.view_change_state.view_number), env.time() + delay);

            // Also always make sure to reset the macro block state.
            self.macro_block_state.reset();

            self.prepare_next_block(env);
        }
    }

    /// Handles a macro block proposal.
    fn handle_macro_block_proposal(&mut self, proposal: MacroBlock, signature: Signature<MacroHeader>, mut env: Environment<Event, MetricsEventType>) {
        let processing_time = env.time() + self.timing.proposal_processing_time(&proposal);
        env.schedule_self(Event::ProposalProcessed(proposal, signature), processing_time);
    }

    /// A macro block proposal has been processed.
    fn processed_proposal(&mut self, proposal: MacroBlock, signature: Signature<MacroHeader>, mut env: Environment<Event, MetricsEventType>) {
        // We verify the proposal first.
        let mut result = self.verify_macro_block(&proposal, true);

        // Check block producer.
        let public_key = self.get_producer_at(proposal.header.digest.block_number, proposal.header.digest.view_number);
        if !signature.verify(&public_key,
                             &proposal.header) {
            result = Err(BlockError::InvalidBlockProducer);
        }

        if let Err(ref e) = result {
            warn!("Got invalid block proposal, reason {:?}", e);
        }

        if result.is_ok() {
            // Update state.
            self.macro_block_state.proposal = Some(proposal.clone());
            self.macro_block_state.phase = MacroBlockPhase::PROPOSED;

            let hash = proposal.header.hash();
            // Relay block.
            self.relay(Event::BlockProposal(proposal, signature), &mut env);

            // Send and process prepare message.
            // FIXME: Currently prepare/commit signatures are identical in the simulation.
            let prepare = PbftProof::new(&hash, &self.key_pair.secret_key());
            self.multicast_to_validators(Event::BlockPrepare(prepare.clone()), &mut env);

            self.handle_prepare(prepare, env);
        } else {
            // Ignore block.
        }
    }

    /// Handles an incoming prepare message.
    fn handle_prepare(&mut self, prepare: PbftProof, mut env: Environment<Event, MetricsEventType>) {
        let hash;
        if let Some(ref proposal) = self.macro_block_state.proposal {
            // Verify prepare.
            hash = proposal.header.hash();
            if !prepare.verify(&hash) {
                return;
            }
        } else {
            // Ignore if we cannot verify.
            return;
        }

        self.macro_block_state.add_prepare(prepare);

        // When 2f + 1 prepare messages have been received, commit to proposal.
        if self.macro_block_state.num_prepares() > self.protocol_config.two_third_threshold() {
            self.macro_block_state.phase = MacroBlockPhase::PREPARED;

            // Send and process prepare message.
            // FIXME: Currently prepare/commit signatures are identical in the simulation.
            let commit = PbftProof::new(&hash, &self.key_pair.secret_key());
            self.multicast_to_validators(Event::BlockCommit(commit.clone()), &mut env);

            self.handle_commit(commit, env);
        }
    }

    /// Handles an incoming prepare message.
    fn handle_commit(&mut self, commit: PbftProof, mut env: Environment<Event, MetricsEventType>) {
        let hash;
        if let Some(ref proposal) = self.macro_block_state.proposal {
            // Verify prepare.
            hash = proposal.header.hash();
            if !commit.verify(&hash) {
                return;
            }
        } else {
            // Ignore if we cannot verify.
            return;
        }

        self.macro_block_state.add_commit(commit);

        // When 2f + 1 prepare messages have been received, commit to proposal.
        if self.macro_block_state.num_commits() > self.protocol_config.two_third_threshold() {
            self.macro_block_state.phase = MacroBlockPhase::COMMITTED;

            // Block proposal accepted, build it and relay it.
            let mut block = self.macro_block_state.proposal.take().unwrap();
            block.justification = Some(PbftJustification {
                prepare: AggregateProof::create(&self.macro_block_state.prepares, &self.validators),
                commit: AggregateProof::create(&self.macro_block_state.commits, &self.validators),
            });

            let block = Block::Macro(block);

            self.store_block(block.clone());

            // Relay block.
            self.relay(Event::Block(block), &mut env);

            self.prepare_next_block(env);
        }
    }

    /// Verifies a block of any type.
    fn verify_block(&self, block: &Block) -> Result<(), BlockError> {
        match block {
            Block::Micro(ref micro_block) => self.verify_micro_block(micro_block),
            Block::Macro(ref macro_block) => self.verify_macro_block(macro_block, false),
        }
    }

    /// Verifies the validity of a micro block.
    fn verify_micro_block(&self, block: &MicroBlock) -> Result<(), BlockError> {
        let block_number = block.header.digest.block_number;
        // Check valid block number.
        if block_number > self.next_block_number()
            || block_number <= self.last_macro_block() {
            return Err(BlockError::InvalidBlockNumber);
        }

        // Check correct type.
        if self.block_type_at(block_number) != BlockType::Micro {
            return Err(BlockError::InvalidBlockType);
        }

        // Check Signature.
        if !block.justification.verify(&block.header.digest.validator, &block.header) {
            return Err(BlockError::InvalidSignature);
        }

        // Get potentially conflicting block.
        let other: Option<&Block> = self.chain.get(block_number as usize);

        // Check whether we committed not to accept blocks from this view change number.
        if block_number == self.next_block_number() {
            if block.header.digest.view_number < self.view_change_state.view_number {
                return Err(BlockError::OldViewChangeNumber);
            }
        } else {
            let other = other.unwrap();
            match block.header.digest.view_number.cmp(&other.view_number()) {
                Ordering::Less => {
                    return Err(BlockError::OldViewChangeNumber);
                },
                Ordering::Equal => {
                    // Easy slashing case.
                    let other_micro = match other {
                        Block::Micro(other) => other,
                        _ => unreachable!(),
                    };
                    return Err(BlockError::MicroBlockFork(SlashInherent {
                        header1: block.header.clone(),
                        justification1: block.justification.clone(),
                        header2: other_micro.header.clone(),
                        justification2: other_micro.justification.clone(),
                    }));
                },
                _ => {},
            }
        }

        if block.header.digest.view_number > 0 {
            // Verify aggregate view change signatures.
            if let Some(ref view_change_proof) = block.extrinsics.view_change_messages {
                let expected_message = ViewChangeInternals {
                    block_number: block.header.digest.block_number,
                    new_view_number: block.header.digest.view_number,
                };
                let keys = get_validators(&self.validators, &view_change_proof.public_key_bitmap);
                let aggregate_key = AggregatePublicKey::from(keys);
                if !view_change_proof.signatures.verify_single(&aggregate_key, &expected_message) {
                    return Err(BlockError::InvalidViewChangeMessages);
                }
            } else {
                return Err(BlockError::MissingViewChangeMessages);
            }
        }

        // TODO: Check timestamp.
        // TODO: Check transactions.
        // TODO: Check slash inherents.
        // TODO: Check Merkle hashes.
        // TODO: Check for conflicting block.

        Ok(())
    }

    /// Verifies the validity of a micro block.
    fn verify_macro_block(&self, block: &MacroBlock, proposal: bool) -> Result<(), BlockError> {
        let block_number = block.header.digest.block_number;
        // Check valid block number.
        if block_number != self.next_block_number() {
            return Err(BlockError::InvalidBlockNumber);
        }

        // Check correct type.
        if self.block_type_at(block_number) != BlockType::Macro {
            return Err(BlockError::InvalidBlockType);
        }

        let hash = block.header.hash();

        // Check Signature (if not a proposal).
        match (proposal, &block.justification) {
            (true, _) => {},
            (false, Some(justification)) =>  {
                if !justification.verify(&self.validators, &hash) {
                    return Err(BlockError::InvalidSignature);
                }
            },
            _ => {
                return Err(BlockError::MissingJustification);
            },
        }

        // Check whether we committed not to accept blocks from this view change number.
        if block.header.digest.view_number < self.view_change_state.view_number {
            return Err(BlockError::OldViewChangeNumber);
        }

        if block.header.digest.view_number > 0 {
            // Verify aggregate view change signatures.
            if let Some(ref view_change_proof) = block.extrinsics.view_change_messages {
                let expected_message = ViewChangeInternals {
                    block_number: block.header.digest.block_number,
                    new_view_number: block.header.digest.view_number,
                };
                let keys = get_validators(&self.validators, &view_change_proof.public_key_bitmap);
                let aggregate_key = AggregatePublicKey::from(keys);
                if !view_change_proof.signatures.verify_single(&aggregate_key, &expected_message) {
                    return Err(BlockError::InvalidViewChangeMessages);
                }
            } else {
                return Err(BlockError::MissingViewChangeMessages);
            }
        }

        // TODO: Check timestamp.
        // TODO: Check Merkle hashes.

        Ok(())
    }

    /// Calculates the next block producer from the validator list.
    fn get_producer_at(&self, block_number: u32, view_number: u16) -> PublicKey {
        // The block must not be before the last macro block.
        // Last macro block is at block_number - (block_number % num_micro_blocks + 1)
        assert!(block_number > self.last_macro_block());

        // TODO
        unimplemented!()
    }

    fn relay(&self, event: Event, env: &mut Environment<Event, MetricsEventType>) {

    }

    fn multicast_to_validators(&self, event: Event, env: &mut Environment<Event, MetricsEventType>) {

    }
}