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
    pub validators: Vec<PublicKey>,
    pub parent_macro_hash: Hash,
    pub block_number: u32,
    pub view_number: u16,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MicroDigest {
    pub validator: PublicKey,
    pub block_number: u32,
    pub view_number: u16,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MacroHeader {
    pub parent_hash: Hash,
    pub digest: MacroDigest,
    pub extrinsics_root: Hash,
    pub state_root: Hash,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MicroHeader {
    pub parent_hash: Hash,
    pub digest: MicroDigest,
    pub extrinsics_root: Hash,
    pub state_root: Hash,
}

#[derive(Clone, Debug)]
pub struct MacroExtrinsics {
    pub timestamp: u64,
    pub seed: Signature<Seed>,
    pub view_change_messages: Option<ViewChangeProof>,
}

#[derive(Clone, Debug)]
pub struct MicroExtrinsics {
    pub timestamp: u64,
    pub seed: Signature<Seed>,
    pub view_change_messages: Option<ViewChangeProof>,
    pub slash_inherents: Vec<SlashInherent>,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug)]
pub struct MacroBlock {
    pub header: MacroHeader,
    pub extrinsics: MacroExtrinsics,
    pub justification: PbftJustification,

}

#[derive(Clone, Debug)]
pub struct MicroBlock {
    pub header: MicroHeader,
    pub extrinsics: MicroExtrinsics,
    pub justification: Signature<MicroHeader>,
}
