use crate::datastructures::hash::Hash;
use crate::datastructures::pbft::PbftJustification;
use crate::datastructures::pbft::ViewChangeProof;
use crate::datastructures::signature::PublicKey;
use crate::datastructures::signature::Signature;
use crate::datastructures::slashing::SlashInherent;
use crate::datastructures::transaction::Transaction;

pub type Seed = Hash;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum BlockType {
    Macro,
    Micro,
}

#[derive(Clone, Debug)]
pub enum Block {
    Macro(MacroBlock),
    Micro(MicroBlock),
}

impl Block {
    pub fn block_number(&self) -> u32 {
        match self {
            Block::Macro(ref block) => block.header.digest.block_number,
            Block::Micro(ref block) => block.header.digest.block_number,
        }
    }

    pub fn block_type(&self) -> BlockType {
        match self {
            Block::Macro(_) => BlockType::Macro,
            Block::Micro(_) => BlockType::Micro,
        }
    }
}

#[derive(Clone, Debug)]
pub enum BlockHeader {
    Micro(MicroHeader),
    Macro(MacroHeader),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MacroDigest {
    validators: Vec<PublicKey>,
    parent_macro_hash: Hash,
    block_number: u32,
    view_change_number: u16,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MicroDigest {
    validator: PublicKey,
    block_number: u32,
    view_change_number: u16,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MacroHeader {
    parent_hash: Hash,
    digest: MacroDigest,
    extrinsics_root: Hash,
    state_root: Hash,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MicroHeader {
    parent_hash: Hash,
    digest: MicroDigest,
    extrinsics_root: Hash,
    state_root: Hash,
}

#[derive(Clone, Debug)]
pub struct MacroExtrinsics {
    timestamp: u64,
    seed: Signature<Seed>,
    view_change_messages: Option<ViewChangeProof>,
}

#[derive(Clone, Debug)]
pub struct MicroExtrinsics {
    timestamp: u64,
    seed: Signature<Seed>,
    view_change_messages: Option<ViewChangeProof>,
    slash_inherents: Vec<SlashInherent>,
    transactions: Vec<Transaction>,
}

#[derive(Clone, Debug)]
pub struct MacroBlock {
    header: MacroHeader,
    extrinsics: MacroExtrinsics,
    justification: PbftJustification,

}

#[derive(Clone, Debug)]
pub struct MicroBlock {
    header: MicroHeader,
    extrinsics: MicroExtrinsics,
    justification: Signature<MicroHeader>,
}
