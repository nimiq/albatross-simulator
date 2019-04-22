use std::time::Duration;

use crate::actors::Timing;
use crate::actors::VerificationTime;
use crate::datastructures::block::MicroHeader;
use crate::datastructures::signature::Signature;

#[derive(Clone, Debug)]
pub struct SlashInherent {
    pub header1: MicroHeader,
    pub justification1: Signature<MicroHeader>,
    pub header2: MicroHeader,
    pub justification2: Signature<MicroHeader>,
}

impl VerificationTime for SlashInherent {
    fn verification_time(&self, timing: &Timing) -> Duration {
        self.justification1.verification_time(timing)
            + self.justification2.verification_time(timing)
    }
}
