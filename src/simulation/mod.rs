use crate::datastructures::block::Block;
use crate::datastructures::block::MacroBlock;
use crate::datastructures::block::MacroHeader;
use crate::datastructures::pbft::PbftProof;
use crate::datastructures::pbft::ViewChange;
use crate::datastructures::signature::Signature;
use crate::datastructures::transaction::Transaction;
use crate::protocol::macro_block::MacroBlockPhase;

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
    ProposalProcessed(MacroBlock, Signature<MacroHeader>),
    BlockProduced(Block),
    TransactionProcessed(Transaction),
    MicroBlockTimeout(u32, u16),
    MacroBlockTimeout(u32, u16, MacroBlockPhase),
}

pub struct SimulationConfig {
    pub blocks: u32,
}
