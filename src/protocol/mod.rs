use crate::datastructures::block::BlockType;
use crate::datastructures::signature::PublicKey;
use crate::datastructures::block::Block;
use simulator::Environment;
use crate::simulation::Event;
use crate::actors::Timing;
use std::time::Duration;
use crate::actors::MetricsEventType;
use crate::datastructures::signature::SecretKey;

pub mod macro_block;
pub mod micro_block;

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

pub trait Protocol {
    fn produce_block(&mut self);
    fn own_public_key(&self) -> &PublicKey;
    fn own_secret_key(&self) -> &SecretKey;
    fn store_block(&mut self, block: Block);
    fn timing(&self) -> &Timing;
    fn protocol_config(&self) -> &ProtocolConfig;
    fn next_block_type(&self) -> BlockType;
    fn next_block_producer(&self, view_change_number: u16) -> PublicKey;
    fn next_block_number(&self) -> u32;
    fn handle_block(&mut self, event: Event, env: Environment<Event, MetricsEventType>);
    fn get_validators(&self, bitmap: &[u16]) -> Vec<PublicKey>;

    fn broadcast_to_validators(&self, event: Event, env: Environment<Event, MetricsEventType>);
}
