use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

use crate::datastructures::pbft::ViewChange;
use crate::protocol::micro_block::MicroBlockError;

pub mod macro_block;
pub mod micro_block;
pub mod honest_protocol;

pub struct ProtocolConfig {
    pub block_timeout: Duration,
    pub num_micro_blocks: u32,
    pub num_validators: u16,
}

impl ProtocolConfig {
    pub fn two_third_threshold(&self) -> u16 {
        // TODO
        0
    }
}

#[derive(Debug)]
pub enum BlockError {
    Micro(MicroBlockError),
//    Macro(MacroBlockError),
}

pub struct ViewChangeState {
    pub view_number: u16,
    pub view_change_messages: HashMap<u16, HashSet<ViewChange>>,
}

impl ViewChangeState {
    pub fn add_message(&mut self, view_change: ViewChange) {
        self.view_change_messages.entry(view_change.internals.new_view_number)
            .or_insert_with(HashSet::new)
            .insert(view_change);
    }

    pub fn num_messages(&self, view_number: u16) -> u16 {
        self.view_change_messages.get(&view_number)
            .map(|s| s.len())
            .unwrap_or(0) as u16
    }

    pub fn reset(&mut self) {
        self.view_number = 0;
        self.view_change_messages.clear();
    }
}
