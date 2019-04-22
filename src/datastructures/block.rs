use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use crate::actors::Timing;
use crate::actors::VerificationTime;
use crate::datastructures::hash::Hash;
use crate::datastructures::hash::Hasher;
use crate::datastructures::pbft::PbftJustification;
use crate::datastructures::pbft::ViewChangeProof;
use crate::datastructures::signature::KeyPair;
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub fn view_number(&self) -> u16 {
        match self {
            Block::Macro(ref block) => block.header.digest.view_number,
            Block::Micro(ref block) => block.header.digest.view_number,
        }
    }

    pub fn block_type(&self) -> BlockType {
        match self {
            Block::Macro(_) => BlockType::Macro,
            Block::Micro(_) => BlockType::Micro,
        }
    }

    pub fn seed(&self) -> &Signature<Seed> {
        match self {
            Block::Macro(ref block) => &block.extrinsics.seed,
            Block::Micro(ref block) => &block.extrinsics.seed,
        }
    }

    pub fn hash(&self) -> Hash {
        match self {
            Block::Macro(ref block) => block.header.hash(),
            Block::Micro(ref block) => block.header.hash(),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[#{}, view {}, type {:?}]", self.block_number(), self.view_number(), self.block_type())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockHeader {
    Micro(MicroHeader),
    Macro(MacroHeader),
}

impl BlockHeader {
    pub fn hash(&self) -> Hash {
        match self {
            BlockHeader::Micro(ref header) => header.hash(),
            BlockHeader::Macro(ref header) => header.hash(),
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MacroDigest {
    pub validators: Vec<PublicKey>,
    pub parent_macro_hash: Hash,
    pub block_number: u32,
    pub view_number: u16,
}

impl MacroDigest {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.validators.len() * 4 + 32 + 8 + 4);
        for validator in self.validators.iter() {
            v.extend_from_slice(&validator.to_bytes());
        }
        v.extend_from_slice(self.parent_macro_hash.as_ref());
        v.extend_from_slice(&self.block_number.to_be_bytes());
        v.extend_from_slice(&self.view_number.to_be_bytes());
        v
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MicroDigest {
    pub validator: PublicKey,
    pub block_number: u32,
    pub view_number: u16,
}

impl MicroDigest {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(4 + 8 + 4);
        v.extend_from_slice(&self.validator.to_bytes());
        v.extend_from_slice(&self.block_number.to_be_bytes());
        v.extend_from_slice(&self.view_number.to_be_bytes());
        v
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MacroHeader {
    pub parent_hash: Hash,
    pub digest: MacroDigest,
    pub extrinsics_root: Hash,
    pub state_root: Hash,
}

impl MacroHeader {
    pub fn hash(&self) -> Hash {
        Hasher::default()
            .chain(&self.parent_hash)
            .chain(&self.digest.to_bytes())
            .chain(&self.extrinsics_root)
            .chain(&self.state_root)
            .result()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct MicroHeader {
    pub parent_hash: Hash,
    pub digest: MicroDigest,
    pub extrinsics_root: Hash,
    pub state_root: Hash,
}

impl MicroHeader {
    pub fn hash(&self) -> Hash {
        Hasher::default()
            .chain(&self.parent_hash)
            .chain(&self.digest.to_bytes())
            .chain(&self.extrinsics_root)
            .chain(&self.state_root)
            .result()
    }
}

impl fmt::Display for MicroHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[#{} view {}, type Micro]", self.digest.block_number, self.digest.view_number)
    }
}

#[derive(Clone, Debug)]
pub struct MacroExtrinsics {
    pub timestamp: u64,
    pub seed: Signature<Seed>,
    pub view_change_messages: Option<ViewChangeProof>,
}

impl MacroExtrinsics {
    pub fn hash(&self) -> Hash {
        // TODO: Implement hash.
        Hash::default()
    }
}

#[derive(Clone, Debug)]
pub struct MicroExtrinsics {
    pub timestamp: u64,
    pub seed: Signature<Seed>,
    pub view_change_messages: Option<ViewChangeProof>,
    pub slash_inherents: Vec<SlashInherent>,
    pub transactions: Vec<Transaction>,
}

impl MicroExtrinsics {
    pub fn hash(&self) -> Hash {
        // TODO: Implement hash.
        Hash::default()
    }
}

#[derive(Clone, Debug)]
pub struct MacroBlock {
    pub header: MacroHeader,
    pub extrinsics: MacroExtrinsics,
    pub justification: Option<PbftJustification>,
}

impl MacroBlock {
    pub fn create_genesis_block(validators: &HashSet<usize>) -> Self {
        let digest = MacroDigest {
            validators: validators.iter().map(|&i| KeyPair::from_id(i as u64).public_key()).collect(),
            block_number: 0,
            view_number: 0,
            parent_macro_hash: Hash::default(),
        };

        let seed = KeyPair::from_id(0)
            .secret_key()
            .sign(&Hash::default());
        let extrinsics = MacroExtrinsics {
            timestamp: 0,
            seed,
            view_change_messages: None,
        };

        let header = MacroHeader {
            parent_hash: Hash::default(),
            digest,
            extrinsics_root: extrinsics.hash(),
            state_root: Hash::default(), // TODO: Simulate stake.
        };

        MacroBlock {
            header,
            extrinsics,
            justification: None, // Only block without justification.
        }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash()
    }
}

impl VerificationTime for MacroBlock {
    fn verification_time(&self, timing: &Timing) -> Duration {
        let mut time = self.extrinsics.seed.verification_time(timing);

        if let Some(ref proof) = self.extrinsics.view_change_messages {
            time += proof.verification_time(timing);
        }

        if let Some(ref justification) = self.justification {
            time += justification.verification_time(timing);
        }

        time
    }
}

impl PartialEq for MacroBlock {
    fn eq(&self, other: &MacroBlock) -> bool {
        self.header == other.header
        // TODO
    }
}

impl Eq for MacroBlock {}

impl fmt::Display for MacroBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[#{}, view {}, type Macro]",
               self.header.digest.block_number,
               self.header.digest.view_number)
    }
}

#[derive(Clone, Debug)]
pub struct MicroBlock {
    pub header: MicroHeader,
    pub extrinsics: MicroExtrinsics,
    pub justification: Signature<MicroHeader>,
}

impl VerificationTime for MicroBlock {
    fn verification_time(&self, timing: &Timing) -> Duration {
        let mut time = self.extrinsics.seed.verification_time(timing)
            + self.justification.verification_time(timing);

        if let Some(ref proof) = self.extrinsics.view_change_messages {
            time += proof.verification_time(timing);
        }

        // Batch verify transactions.
        time += self.extrinsics.transactions.len() as u32 * timing.batch_verification;
        time += self.extrinsics.slash_inherents.iter().map(|inherent| inherent.verification_time(timing)).sum();

        time
    }
}

impl PartialEq for MicroBlock {
    fn eq(&self, other: &MicroBlock) -> bool {
        self.header == other.header
        // TODO
    }
}

impl Eq for MicroBlock {}
