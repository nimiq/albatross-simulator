use crate::datastructures::block::MicroHeader;
use crate::datastructures::signature::Signature;

#[derive(Clone, Debug)]
pub struct SlashInherent {
    header1: MicroHeader,
    justification1: Signature<MicroHeader>,
    header2: MicroHeader,
    justification2: Signature<MicroHeader>,
}
