use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::Duration;

use crate::actors::Timing;
use crate::actors::VerificationTime;
use crate::datastructures::hash::Hash as ShaHash;
use crate::datastructures::signature::{AggregatePublicKey, PublicKey};
use crate::datastructures::signature::AggregateSignature;
use crate::datastructures::signature::SecretKey;
use crate::datastructures::signature::Signature;

#[derive(Clone, Debug)]
pub struct PbftJustification {
    pub prepare: AggregateProof<ShaHash>,
    pub commit: AggregateProof<ShaHash>,
}

impl PbftJustification {
    pub fn verify(&self, validators: &[PublicKey], hash: &ShaHash) -> bool {
        let aggregate_key = AggregatePublicKey::from(get_validators(validators, &self.prepare.public_key_bitmap));
        if !self.prepare.signatures.verify_single(&aggregate_key, hash) {
            return false;
        }

        let aggregate_key = AggregatePublicKey::from(get_validators(validators, &self.commit.public_key_bitmap));
        self.commit.signatures.verify_single(&aggregate_key, hash)
    }
}

impl VerificationTime for PbftJustification {
    fn verification_time(&self, timing: &Timing) -> Duration {
        self.prepare.verification_time(timing) + self.commit.verification_time(timing)
    }
}

#[derive(Clone, Debug)]
pub struct PbftProof {
    pub signature: Signature<ShaHash>,
    id: PublicKey,
}

impl PbftProof {
    pub fn new(hash: &ShaHash, key: &SecretKey) -> Self {
        PbftProof {
            signature: key.sign(&hash),
            id: key.into(),
        }
    }

    pub fn verify(&self, hash: &ShaHash) -> bool {
        self.signature.verify(&self.id, &hash)
    }
}

impl VerificationTime for PbftProof {
    fn verification_time(&self, timing: &Timing) -> Duration {
        self.signature.verification_time(timing)
    }
}

impl PartialEq for PbftProof {
    fn eq(&self, other: &PbftProof) -> bool {
        self.signature == other.signature
            && self.id == other.id
    }
}

impl Eq for PbftProof {}

impl Hash for PbftProof {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.signature, state);
        self.id.hash(state);
    }
}

impl fmt::Display for PbftProof {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "PbftMessage(from {})", self.id)
    }
}

/// Return a set of public keys given to a bitmap.
/// We only need this for the current validator set, since macro blocks cannot be reverted.
pub fn get_validators(validators: &[PublicKey], bitmap: &[u16]) -> Vec<PublicKey> {
    let mut keys = Vec::new();
    for validator in bitmap.iter() {
        keys.push(validators[usize::from(*validator)].clone());
    }
    keys
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

impl fmt::Display for ViewChange {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ViewChange(block {}, new view {})",
               self.internals.block_number,
               self.internals.new_view_number)
    }
}

#[derive(Clone, Debug)]
pub struct AggregateProof<T: Eq> {
    pub signatures: AggregateSignature<T>,
    pub public_key_bitmap: Vec<u16>,
}

impl AggregateProof<ShaHash> {
    pub fn create(set: &HashSet<PbftProof>, validators: &[PublicKey]) -> Self {
        let mut signatures = Vec::with_capacity(set.len());
        let mut key_bitmap = Vec::with_capacity(set.len());

        // FIXME: Inefficient.
        let mut key_to_id_map = HashMap::new();
        for (i, key) in validators.iter().enumerate() {
            key_to_id_map.insert(key.clone(), i as u16);
        }

        for proof in set.iter() {
            signatures.push(proof.signature.clone());
            key_bitmap.push(*key_to_id_map.get(&proof.id).unwrap());
        }

        AggregateProof {
            signatures: AggregateSignature::from(signatures),
            public_key_bitmap: key_bitmap,
        }
    }
}

impl<T: Eq> VerificationTime for AggregateProof<T> {
    fn verification_time(&self, timing: &Timing) -> Duration {
        self.signatures.verification_time(timing) + self.public_key_bitmap.len() as u32 * timing.generate_aggregate_public_key
    }
}

pub type ViewChangeProof = AggregateProof<ViewChangeInternals>;

impl AggregateProof<ViewChangeInternals> {
    pub fn create_from_view_change(set: &HashSet<ViewChange>, validators: &[PublicKey]) -> Self {
        let mut signatures = Vec::with_capacity(set.len());
        let mut key_bitmap = Vec::with_capacity(set.len());

        // FIXME: Inefficient.
        let mut key_to_id_map = HashMap::new();
        for (i, key) in validators.iter().enumerate() {
            key_to_id_map.insert(key.clone(), i as u16);
        }

        for proof in set.iter() {
            signatures.push(proof.signature.clone());
            key_bitmap.push(*key_to_id_map.get(&proof.id).unwrap());
        }

        AggregateProof {
            signatures: AggregateSignature::from(signatures),
            public_key_bitmap: key_bitmap,
        }
    }
}
