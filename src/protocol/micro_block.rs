use crate::datastructures::slashing::SlashInherent;

#[derive(Debug)]
pub enum MicroBlockError {
    InvalidBlockType,
    InvalidBlockNumber,
    InvalidBlockProducer,
    InvalidSignature,
    MissingViewChangeMessages,
    InvalidViewChangeMessages,
    OldViewChangeNumber,
    MicroBlockFork(SlashInherent),
}
