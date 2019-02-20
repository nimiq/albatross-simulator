use bls;
use pairing::bls12_381::Bls12;

pub struct Signature(bls::Signature<Bls12>);
pub struct PublicKey(bls::PublicKey<Bls12>);

pub struct AggregateSignature {
    signature: bls::AggregateSignature<Bls12>,
    bitmap: Vec<u8>,
}