use std::hash::Hash;
use std::hash::Hasher;

use crate::datastructures::block::MacroHeader;
use crate::datastructures::signature::AggregateSignature;
use crate::datastructures::signature::PublicKey;
use crate::datastructures::signature::SecretKey;
use crate::datastructures::signature::Signature;

#[derive(Clone, Debug)]
pub struct PbftJustification {
    prepare: AggregateSignature<MacroHeader>,
    commit: AggregateSignature<MacroHeader>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ViewChangeInternals {
    pub block_number: u32,
    pub new_view_number: u16,
}

#[derive(Clone, Debug)]
pub struct ViewChange {
    pub internals: ViewChangeInternals,
    pub signature: Signature<ViewChangeInternals>,
    id: PublicKey,
}

impl ViewChange {
    pub fn new(block_number: u32, new_view_number: u16, key: &SecretKey) -> Self {
        let internals = ViewChangeInternals {
            block_number,
            new_view_number,
        };
        ViewChange {
            signature: key.sign(&internals),
            internals,
            id: key.into(),
        }
    }

    pub fn verify(&self) -> bool {
        self.signature.verify(&self.id, &self.internals)
    }
}

impl PartialEq for ViewChange {
    fn eq(&self, other: &ViewChange) -> bool {
        self.internals == other.internals
            && self.id == other.id
    }
}

impl Eq for ViewChange {}

impl Hash for ViewChange {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.internals.hash(state);
        self.id.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct ViewChangeProof {
    pub signatures: AggregateSignature<ViewChangeInternals>,
    pub public_key_bitmap: Vec<u16>,
}
