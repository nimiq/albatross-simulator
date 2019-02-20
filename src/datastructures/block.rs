use crate::datastructures::hash::Hash;
use crate::datastructures::pbft::PbftJustification;
use crate::datastructures::pbft::ViewChangeProof;
use crate::datastructures::signature::PublicKey;
use crate::datastructures::signature::Signature;
use crate::datastructures::slashing::SlashInherent;
use crate::datastructures::transaction::Transaction;

pub enum Block {
    Macro(MacroBlock),
    Micro(MicroBlock),
}

pub enum BlockHeader {
    Micro(MicroHeader),
    Macro(MacroHeader),
}

pub struct MacroDigest {
    validators: Vec<PublicKey>,
    parent_macro_hash: Hash,
    block_number: u32,
    view_change_number: u16,
}

pub struct MicroDigest {
    validator: PublicKey,
    block_number: u32,
    view_change_number: u16,
}

pub struct MacroHeader {
    parent_hash: Hash,
    digest: MacroDigest,
    extrinsics_root: Hash,
    state_root: Hash,
}

pub struct MicroHeader {
    parent_hash: Hash,
    digest: MicroDigest,
    extrinsics_root: Hash,
    state_root: Hash,
}

pub struct MacroExtrinsics {
    timestamp: u64,
    seed: Signature,
    view_change_messages: Option<ViewChangeProof>,
}

pub struct MicroExtrinsics {
    timestamp: u64,
    seed: Signature,
    view_change_messages: Option<ViewChangeProof>,
    slash_inherents: Vec<SlashInherent>,
    transactions: Vec<Transaction>,
}

pub struct MacroBlock {
    header: MacroHeader,
    extrinsics: MacroExtrinsics,
    justification: PbftJustification,

}

pub struct MicroBlock {
    header: MicroHeader,
    extrinsics: MicroExtrinsics,
    justification: Signature,
}
