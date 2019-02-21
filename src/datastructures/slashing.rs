use crate::datastructures::block::MicroHeader;
use crate::datastructures::signature::Signature;

#[derive(Clone, Debug)]
pub struct SlashInherent {
    pub header1: MicroHeader,
    pub justification1: Signature<MicroHeader>,
    pub header2: MicroHeader,
    pub justification2: Signature<MicroHeader>,
}
