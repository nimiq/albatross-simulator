use std::collections::HashSet;

use crate::datastructures::block::MacroBlock;
use crate::datastructures::pbft::PbftProof;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum MacroBlockPhase {
    WAITING,
    PROPOSED,
    PREPARED,
    COMMITTED,
}

pub struct MacroBlockState {
    pub view_number: u16,
    pub proposal: Option<MacroBlock>,
    pub prepares: HashSet<PbftProof>,
    pub commits: HashSet<PbftProof>,

    pub phase: MacroBlockPhase,
}

impl MacroBlockState {
    pub fn add_prepare(&mut self, prepare: PbftProof) {
        self.prepares.insert(prepare);
    }

    pub fn has_prepare(&mut self, prepare: &PbftProof) -> bool {
        self.prepares.contains(prepare)
    }

    pub fn num_prepares(&self) -> u16 {
        self.prepares.len() as u16
    }

    pub fn add_commit(&mut self, prepare: PbftProof) {
        self.commits.insert(prepare);
    }

    pub fn has_commit(&mut self, prepare: &PbftProof) -> bool {
        self.commits.contains(prepare)
    }

    pub fn num_commits(&self) -> u16 {
        self.commits.len() as u16
    }

    pub fn reset(&mut self) {
        self.view_number = 0;
        self.proposal = None;
        self.prepares.clear();
        self.commits.clear();
        self.phase = MacroBlockPhase::WAITING;
    }
}

impl Default for MacroBlockState {
    fn default() -> Self {
        MacroBlockState {
            view_number: 0,
            proposal: None,
            prepares: HashSet::new(),
            commits: HashSet::new(),
            phase: MacroBlockPhase::WAITING,
        }
    }
}
