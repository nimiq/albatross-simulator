use std::collections::HashMap;

use crate::datastructures::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Signature<M: Eq> {
    public_key: PublicKey,
    message: M,
}

impl<M: Eq> Signature<M> {
    pub fn verify(&self, public_key: &PublicKey, message: &M) -> bool {
        &self.public_key == public_key && &self.message == message
    }
}

impl<M: Eq + AsRef<[u8]>> Signature<M> {
    pub fn to_hash(&self) -> Hash {
        // Required to generate deterministic randomness.
        // Simply hash public key and message for our simulation.
        Hasher::default()
            .chain(&self.public_key.to_bytes())
            .chain(&self.message)
            .result()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_hash().into()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct KeyPair {
    id: u64,
}

impl KeyPair {
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            id: self.id,
        }
    }

    pub fn secret_key(&self) -> SecretKey {
        SecretKey {
            id: self.id,
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PublicKey {
    id: u64,
}

impl PublicKey {
    pub fn to_bytes(&self) -> [u8; 8] {
        self.id.to_be_bytes()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SecretKey {
    id: u64,
}

impl SecretKey {
    pub fn sign<M: Eq + Clone>(&self, message: &M) -> Signature<M> {
        Signature {
            public_key: PublicKey {
                id: self.id,
            },
            message: message.clone(),
        }
    }
}

impl<'a> From<&'a SecretKey> for PublicKey {
    fn from(key: &'a SecretKey) -> Self {
        PublicKey {
            id: key.id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AggregatePublicKey {
    public_keys: Vec<PublicKey>,
}

impl From<Vec<PublicKey>> for AggregatePublicKey {
    fn from(keys: Vec<PublicKey>) -> Self {
        AggregatePublicKey {
            public_keys: keys,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AggregateSignature<M: Eq> {
    signatures: HashMap<PublicKey, Signature<M>>,
}

impl<M: Eq> AggregateSignature<M> {
    pub fn verify_single(&self, public_keys: &AggregatePublicKey, message: &M) -> bool {
        for public_key in public_keys.public_keys.iter() {
            let signature = self.signatures.get(public_key);
            if let Some(signature) = signature {
                if !signature.verify(public_key, message) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn verify_multiple(&self, public_keys: &AggregatePublicKey, messages: &[M]) -> bool {
        for (public_key, message) in public_keys.public_keys.iter().zip(messages) {
            let signature = self.signatures.get(public_key);
            if let Some(signature) = signature {
                if !signature.verify(public_key, message) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}
