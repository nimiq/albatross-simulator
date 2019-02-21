use std::fmt;

use crate::datastructures::block::Block;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::block::MacroHeader;
use crate::datastructures::pbft::PbftProof;
use crate::datastructures::pbft::ViewChange;
use crate::datastructures::signature::Signature;
use crate::datastructures::transaction::Transaction;
use crate::protocol::macro_block::MacroBlockPhase;

pub mod metrics;
pub mod network;

#[derive(Clone, Debug)]
pub enum Event {
    // External events.
    Block(Block),
    Transaction(Transaction),

    // PBFT.
    ViewChange(ViewChange),
    BlockProposal(MacroBlock, Signature<MacroHeader>),
    BlockPrepare(PbftProof),
    BlockCommit(PbftProof),

    // Internal events.
    BlockProcessed(Block),
    BlockProduced(Block),
    ProposalProcessed(MacroBlock, Signature<MacroHeader>),
    TransactionProcessed(Transaction),
    MicroBlockTimeout(u32, u16),
    MacroBlockTimeout(u32, u16, MacroBlockPhase),

    Init,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            // External events.
            Event::Block(block) => write!(f, "received block {}", block),
            Event::Transaction(transaction) => write!(f, "received transaction"),

            // PBFT.
            Event::ViewChange(view_change) => write!(f, "received view change {}", view_change),
            Event::BlockProposal(proposal, _signature) => write!(f, "received macro block proposal {}", proposal),
            Event::BlockPrepare(proof) => write!(f, "received prepare from {}", proof),
            Event::BlockCommit(proof) => write!(f, "received commit from {}", proof),

            // Internal events.
            Event::BlockProcessed(block) => write!(f, "processed block {}", block),
            Event::BlockProduced(block) => write!(f, "produced block {}", block),
            Event::ProposalProcessed(block, signature) => write!(f, "processed proposal {}", block),
            Event::TransactionProcessed(transaction) => write!(f, "processed transaction"),
            Event::MicroBlockTimeout(block_number, view_number) | Event::MacroBlockTimeout(block_number, view_number, _) => write!(f, "timeout @ {} (view {})", block_number, view_number),

            Event::Init => write!(f, "initialised"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub blocks: u32,
}
