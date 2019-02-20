use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Signature<M: Eq> {
    public_key: PublicKey,
    message: M,
}

impl<M: Eq> Signature<M> {
    pub fn verify(&self, public_key: &PublicKey, message: &M) -> bool {
        &self.public_key == public_key && &self.message == message
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        // Required to generate deterministic randomness.
        // Simply hash public key and message for our simulation.
        unimplemented!()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct PublicKey {
    id: u64,
}

#[derive(Clone, Debug)]
pub struct AggregatePublicKey {
    public_keys: Vec<PublicKey>,
}

#[derive(Clone, Debug)]
pub struct AggregateSignature<M: Eq> {
    signatures: HashMap<PublicKey, Signature<M>>,
}

impl<M: Eq> AggregateSignature<M> {
    pub fn verify_single(&self, public_keys: AggregatePublicKey, message: &M) -> bool {
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

    pub fn verify_multiple(&self, public_keys: AggregatePublicKey, messages: &[M]) -> bool {
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
