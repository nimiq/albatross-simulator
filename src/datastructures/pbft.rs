use crate::datastructures::signature::AggregateSignature;
use crate::datastructures::signature::Signature;
use crate::datastructures::block::MacroHeader;

#[derive(Clone, Debug)]
pub struct PbftJustification {
    prepare: AggregateSignature<MacroHeader>,
    commit: AggregateSignature<MacroHeader>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ViewChangeInternals {
    pub block_number: u32,
    pub new_view_number: u16,
}

#[derive(Clone, Debug)]
pub struct ViewChange {
    internals: ViewChangeInternals,
    signature: Signature<ViewChangeInternals>,
}

#[derive(Clone, Debug)]
pub struct ViewChangeProof {
    pub signatures: AggregateSignature<ViewChangeInternals>,
    pub public_key_bitmap: Vec<u16>,
}
