use std::time::Duration;

use crate::actors::Timing;
use crate::actors::VerificationTime;

#[derive(Clone, Debug)]
pub struct Transaction {}

impl VerificationTime for Transaction {
    fn verification_time(&self, timing: &Timing) -> Duration {
        timing.verification
    }
}