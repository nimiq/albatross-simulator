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
    block_number: u32,
    new_validator: u16,
}

#[derive(Clone, Debug)]
pub struct ViewChange {
    internals: ViewChangeInternals,
    signature: Signature<ViewChangeInternals>,
}

pub type ViewChangeProof = AggregateSignature<ViewChangeInternals>;
