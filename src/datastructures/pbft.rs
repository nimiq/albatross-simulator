use crate::datastructures::signature::AggregateSignature;

pub struct PbftJustification {
    prepare: AggregateSignature,
    commit: AggregateSignature,
}

pub type ViewChangeProof = AggregateSignature;
