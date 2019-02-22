use std::cmp::Ordering;
use std::collections::HashSet;

use num_bigint::BigUint;
use num_traits::ToPrimitive;

use simulator::Environment;
use simulator::metrics::Metrics;

use crate::actors::Timing;
use crate::datastructures::block::*;
use crate::datastructures::hash::*;
use crate::datastructures::pbft::*;
use crate::datastructures::signature::*;
use crate::datastructures::slashing::SlashInherent;
use crate::protocol::BlockError;
use crate::protocol::macro_block::{MacroBlockPhase, MacroBlockState};
use crate::protocol::ProtocolConfig;
use crate::protocol::ViewChangeState;
use crate::simulation::Event;
use crate::simulation::metrics::MetricsEventType;

pub struct HonestProtocol {
    protocol_config: ProtocolConfig,
    timing: Timing,
    view_change_state: ViewChangeState,
    macro_block_state: MacroBlockState,
    chain: Vec<Block>,
    key_pair: KeyPair,
    validators: Vec<PublicKey>,

    // Do not accept known blocks.
    known_blocks: HashSet<Hash>,
}

impl HonestProtocol {
    /// Create a protocol instance.
    pub fn new(protocol_config: ProtocolConfig, timing: Timing,
               genesis_block: MacroBlock, key_pair: KeyPair) -> Self {
        HonestProtocol {
            protocol_config,
            timing,
            view_change_state: ViewChangeState::default(),
            macro_block_state: MacroBlockState::default(),
            validators: genesis_block.header.digest.validators.clone(),
            chain: vec![Block::Macro(genesis_block)],
            key_pair,

            known_blocks: HashSet::new(),
        }
    }

    /// Returns the next block number.
    pub fn current_block_number(&self) -> u32 {
        self.chain.len() as u32 - 1
    }

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
        if (block_number + 1 /*next block*/) % (self.protocol_config.num_micro_blocks + 1 /*macro block*/) == 0 {
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

        self.known_blocks.insert(block.hash()); // Also store known block if we produced it.
        self.chain.push(block);

        self.view_change_state.reset();
        self.macro_block_state.reset();
    }

    /// Prepare protocol for next block:
    /// Check if we are the next block producer.
    /// If so, produce a block, otherwise set a timeout.
    pub fn prepare_next_block(&mut self, env: &mut Environment<Event, MetricsEventType>) {
        let next_producer = self.get_producer_at(self.next_block_number(), self.view_change_state.view_number);
        if next_producer == self.key_pair.public_key() {
            self.produce_block(env);
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
    pub fn received_block(&mut self, block: Block, env: &mut Environment<Event, MetricsEventType>) {
        // Check whether we already received this block.
        let hash = block.hash();
        if self.known_blocks.contains(&hash) {
            return;
        }
        self.known_blocks.insert(hash);

        let processing_time = env.time() + self.timing.block_processing_time(&block);
        env.schedule_self(Event::BlockProcessed(block), processing_time);
    }

    /// A block has been processed, ensure its validity.
    /// If it is invalid, ignore it.
    /// If it is valid, store block and reset state.
    pub fn processed_block(&mut self, block: Block, env: &mut Environment<Event, MetricsEventType>) {
        // We verify the block.
        let result = self.verify_block(&block);

        // TODO: Handle slashing (we currently do not store the headers of known blocks).

        if let Err(ref e) = result {
            warn!("Got invalid block, reason {:?}", e);
        }

        if result.is_ok() {
            self.store_block(block.clone());

            // Relay block.
            self.relay(Event::Block(block), env);

            self.prepare_next_block(env);
        } else {
            // Ignore block.
        }
    }

    /// Called when a timeout has been triggered.
    /// Check whether a corresponding (valid) block has been received in the meantime.
    /// If not, prepare and send out view change message.
    pub fn handle_timeout(&mut self, block_number: u32, view_number: u16, env: &mut Environment<Event, MetricsEventType>) {
        // Check whether timeout was triggered and no new block has been accepted in the meanwhile.
        if self.next_block_number() == block_number && self.view_change_state.view_number == view_number {
            // Send and process view change message.
            let view_change = ViewChange::new(block_number, view_number + 1, &self.key_pair.secret_key());
            self.multicast_to_validators(Event::ViewChange(view_change.clone()), env);

            // Handle own message exactly like others.
            self.handle_view_change(view_change, env);
        }
    }

    /// Called when a view change message has been received.
    /// First check validity, then add message.
    /// If we received enough view change messages for this view change number,
    /// stop accepting blocks for this number and move on.
    /// In this case, also check for next block producer or start timeout.
    pub fn handle_view_change(&mut self, view_change: ViewChange, env: &mut Environment<Event, MetricsEventType>) {
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
    pub fn handle_macro_block_proposal(&mut self, proposal: MacroBlock, signature: Signature<MacroHeader>, env: &mut Environment<Event, MetricsEventType>) {
        let processing_time = env.time() + self.timing.proposal_processing_time(&proposal);
        env.schedule_self(Event::ProposalProcessed(proposal, signature), processing_time);
    }

    /// A macro block proposal has been processed.
    pub fn processed_proposal(&mut self, proposal: MacroBlock, signature: Signature<MacroHeader>, env: &mut Environment<Event, MetricsEventType>) {
        // No duplicate proposal processing.
        if self.macro_block_state.proposal.is_some() {
            return;
        }

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
            self.relay(Event::BlockProposal(proposal, signature), env);

            // Send and process prepare message.
            // FIXME: Currently prepare/commit signatures are identical in the simulation.
            let prepare = PbftProof::new(&hash, &self.key_pair.secret_key());
            self.multicast_to_validators(Event::BlockPrepare(prepare.clone()), env);

            self.handle_prepare(prepare, env);
        } else {
            // Ignore block.
        }
    }

    /// Handles an incoming prepare message.
    pub fn handle_prepare(&mut self, prepare: PbftProof, env: &mut Environment<Event, MetricsEventType>) {
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
            self.multicast_to_validators(Event::BlockCommit(commit.clone()), env);

            self.handle_commit(commit, env);
        }
    }

    /// Handles an incoming prepare message.
    pub fn handle_commit(&mut self, commit: PbftProof, env: &mut Environment<Event, MetricsEventType>) {
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
            self.relay(Event::Block(block.clone()), env);

            env.note_event(&MetricsEventType::MacroBlockAccepted(block), env.time());

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
        // TODO: Check prev hash.

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
        // TODO: Check validator list.
        // TODO: Check prev hash.

        Ok(())
    }

    /// Calculates a new validator list.
    fn compute_validators(&self, _block_number: u32, _seed: &Signature<Seed>) -> Vec<PublicKey> {
        // TODO: Actually choose validators.
        self.validators.clone()
    }

    /// Called if we are the block producer and builds a block.
    fn produce_block(&mut self, env: &mut Environment<Event, MetricsEventType>) {
        let block_number = self.next_block_number();
        let view_messages = self.view_change_state.view_change_messages
            .get(&self.view_change_state.view_number)
            .map(|set| AggregateProof::create_from_view_change(set, &self.validators));

        let previous_block: &Block = self.chain.get(block_number as usize - 1).unwrap();
        let seed = self.key_pair.secret_key().sign(&previous_block.seed().hash());

        // TODO Fill block.
        let block = match self.block_type_at(block_number) {
            BlockType::Micro => {
                let extrinsics = MicroExtrinsics {
                    timestamp: 0,
                    seed,
                    view_change_messages: view_messages,
                    slash_inherents: Vec::new(),
                    transactions: Vec::new(),
                };

                let digest = MicroDigest {
                    validator: self.key_pair.public_key(),
                    block_number,
                    view_number: self.view_change_state.view_number,
                };

                let header = MicroHeader {
                    parent_hash: previous_block.hash(),
                    digest,
                    extrinsics_root: extrinsics.hash(),
                    state_root: Hash::default(), // TODO: Simulate stake.
                };

                Block::Micro(MicroBlock {
                    justification: self.key_pair.secret_key().sign(&header),
                    header,
                    extrinsics,
                })
            },
            BlockType::Macro => {
                let digest = MacroDigest {
                    validators: self.compute_validators(block_number, &seed),
                    block_number,
                    view_number: self.view_change_state.view_number,
                    parent_macro_hash: self.chain.get(self.last_macro_block() as usize).map(|block| block.hash()).unwrap(),
                };

                let extrinsics = MacroExtrinsics {
                    timestamp: 0,
                    seed,
                    view_change_messages: view_messages,
                };

                let header = MacroHeader {
                    parent_hash: previous_block.hash(),
                    digest,
                    extrinsics_root: extrinsics.hash(),
                    state_root: Hash::default(), // TODO: Simulate stake.
                };

                Block::Macro(MacroBlock {
                    header,
                    extrinsics,
                    justification: None,
                })
            },
        };

        let processing_time = env.time() + self.timing.block_production_time(&block);
        env.schedule_self(Event::BlockProduced(block), processing_time);
    }

    /// Called after successful block production.
    pub fn produced_block(&mut self, block: Block, env: &mut Environment<Event, MetricsEventType>) {
        match block {
            block @ Block::Micro(_) => {
                self.store_block(block.clone());
                self.relay(Event::Block(block), env);
                self.prepare_next_block(env);
            },
            Block::Macro(proposal) => {
                let signature = self.key_pair.secret_key().sign(&proposal.header);
                self.multicast_to_validators(Event::BlockProposal(proposal.clone(), signature.clone()), env);
                self.processed_proposal(proposal, signature, env);
            },
        }
    }

    /// Calculates the next block producer from the validator list.
    fn get_producer_at(&self, block_number: u32, view_number: u16) -> PublicKey {
        // The block must not be before the last macro block.
        // Last macro block is at block_number - (block_number % num_micro_blocks + 1)
        assert!(block_number > self.last_macro_block());

        let previous_block: &Block = self.chain.get(block_number as usize - 1).unwrap();

        // H(S || i)
        let r = Hasher::default()
            .chain(&previous_block.seed().to_bytes())
            .chain(&view_number.to_be_bytes())
            .result();
        let r: BigUint = BigUint::from_bytes_be(r.as_ref()) % self.validators.len();
        let r = r.to_usize().unwrap();
        self.validators[r].clone()
    }

    fn relay(&self, event: Event, env: &mut Environment<Event, MetricsEventType>) {
        env.broadcast(event);
    }

    fn multicast_to_validators(&self, event: Event, env: &mut Environment<Event, MetricsEventType>) {
        // TODO: Only send to validators.
        env.broadcast(event);
    }
}
