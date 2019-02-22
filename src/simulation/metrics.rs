use std::collections::HashMap;
use std::fmt;
use std::ops::Div;
use std::time::Duration;

use simulator::{Metrics, Time};
use simulator::UniqueId;

use crate::datastructures::block::Block;
use crate::datastructures::block::BlockType;
use crate::datastructures::hash::Hash;
use crate::simulation::Event;

#[derive(Debug, Clone)]
pub enum MetricsEventType {
    MessageEvent {
        own: usize,
        from: usize,
        event: Event,
    },
    MacroBlockAccepted(Block),
}

impl fmt::Display for MetricsEventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            MetricsEventType::MessageEvent { own, from, event } => {
                write!(f, "{} {} from {}", own, event, from)
            },
            MetricsEventType::MacroBlockAccepted(block) => {
                write!(f, "Macro block accepted {}", block)
            },
        }
    }
}

/// A default metrics implementation.
pub struct DefaultMetrics {
    pub block_ids: HashMap<u32, Hash>,
    pub block_types: HashMap<Hash, BlockType>,
    pub block_productions: HashMap<Hash, Time>,
    pub block_receives: HashMap<Hash, HashMap<UniqueId, Time>>,
    pub proposal_accepted: HashMap<Hash, Time>,
}

impl Metrics for DefaultMetrics {
    type EventType = MetricsEventType;

    fn note_event(&mut self, event: &MetricsEventType, time: Time) {
        trace!("Event {}", event);

        match event {
            MetricsEventType::MessageEvent { own, event, .. } => {
                match event {
                    Event::BlockProduced(ref block) => {
                        let hash = block.hash();
                        self.block_types.insert(hash.clone(), block.block_type());
                        self.block_productions.insert(hash.clone(), time);

                        // Overwrites ids.
                        self.block_ids.insert(block.block_number(), hash);
                    },
                    Event::BlockProcessed(ref block) => {
                        let hash = block.hash();
                        // Only note first receive.
                        self.block_receives.entry(hash)
                            .or_insert_with(HashMap::new)
                            .entry(*own)
                            .or_insert(time);
                    },
                    _ => {},
                }
            }
            MetricsEventType::MacroBlockAccepted(ref block) => {
                let hash = block.hash();
                // Overwrite and only store last accepted.
                self.proposal_accepted.insert(hash, time);
            },
        }
    }
}

impl DefaultMetrics {
    pub fn analyze(&self) {
        // Metrics of interest are:
        // - block propagation times (produced to last receive)
        // - macro block proposal to accept time
        // - micro block time (time between production of micro blocks)

        let propagation_times: Vec<Duration> = self.block_types.iter()
            .filter_map(|(hash, ty)| {
                if *ty == BlockType::Micro {
                    self.block_propagation_time(hash)
                } else {
                    None
                }
            })
            .collect();

        if !propagation_times.is_empty() {
            let min = propagation_times.iter().min().unwrap();
            let max = propagation_times.iter().max().unwrap();
            let avg = propagation_times.iter()
                .fold(Duration::default(), |a, b| a + *b).div(propagation_times.len() as u32);

            info!("Micro block propagation time [min/avg/max]: {:?} {:?} {:?}", min, avg, max);
        } else {
            warn!("Empty propagation times!");
        }

        let macro_accept_times: Vec<Duration> = self.block_types.iter().filter_map(|(hash, ty)| {
            if *ty == BlockType::Macro {
                self.macro_accept_time(hash)
            } else {
                None
            }
        }).collect();

        if !macro_accept_times.is_empty() {
            let min = macro_accept_times.iter().min().unwrap();
            let max = macro_accept_times.iter().max().unwrap();
            let avg = macro_accept_times.iter()
                .fold(Duration::default(), |a, b| a + *b).div(macro_accept_times.len() as u32);

            info!("Macro block accept time [min/avg/max]: {:?} {:?} {:?}", min, avg, max);
        } else {
            warn!("Empty macro accept times!");
        }

        let micro_production_times = self.sorted_micro_production_times();
        let mut micro_production_windows = Vec::new();
        for i in 1..micro_production_times.len() {
            micro_production_windows.push(micro_production_times[i] - micro_production_times[i - 1]);
        }

        if !micro_production_windows.is_empty() {
            let min = micro_production_windows.iter().min().unwrap();
            let max = micro_production_windows.iter().max().unwrap();
            let avg = micro_production_windows.iter()
                .fold(Duration::default(), |a, b| a + *b).div(micro_production_windows.len() as u32);

            info!("Micro block time [min/avg/max]: {:?} {:?} {:?}", min, avg, max);
        } else {
            warn!("Empty micro block times!");
        }
    }

    fn block_propagation_time(&self, hash: &Hash) -> Option<Duration> {
        let produced = self.block_productions.get(hash)?;
        let last_receive = self.block_receives.get(hash)?.values().max()?;
        Some(*last_receive - *produced)
    }

    fn sorted_micro_production_times(&self) -> Vec<Time> {
        let mut times: Vec<Time> = self.block_productions.iter().filter_map(|(hash, time)| {
            match self.block_types.get(hash).unwrap() {
                BlockType::Micro => Some(time.clone()),
                _ => None,
            }
        }).collect();
        times.sort();
        times
    }

    fn macro_accept_time(&self, hash: &Hash) -> Option<Duration> {
        let produced = self.block_productions.get(hash)?;
        let last_receive = self.proposal_accepted.get(hash)?;
        Some(*last_receive - *produced)
    }
}

impl Default for DefaultMetrics {
    fn default() -> Self {
        DefaultMetrics {
            block_ids: HashMap::new(),
            block_types: HashMap::new(),
            block_productions: HashMap::new(),
            block_receives: HashMap::new(),
            proposal_accepted: HashMap::new(),
        }
    }
}
