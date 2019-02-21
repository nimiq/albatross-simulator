use crate::datastructures::block::MacroBlock;

pub struct MacroBlockState {
    pub view_number: u16,
    pub proposal: Option<MacroBlock>,
    pub prepares: Vec<()>,
    pub commits: Vec<()>,
}

impl MacroBlockState {
    pub fn reset(&mut self) {
        self.view_number = 0;
        self.proposal = None;
        self.prepares.clear();
        self.commits.clear();
    }
}
