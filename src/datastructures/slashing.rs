use crate::datastructures::block::MicroHeader;
use crate::datastructures::signature::Signature;

pub struct SlashInherent {
    header1: MicroHeader,
    justification1: Signature,
    header2: MicroHeader,
    justification2: Signature,
}
