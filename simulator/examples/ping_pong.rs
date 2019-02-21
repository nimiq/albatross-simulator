use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use simulator::{Environment, Event, Metrics, NetworkConfig, Node, Simulator, Timer};

use crate::example_metrics::DefaultMetrics;
use std::borrow::Cow;

mod example_metrics;

#[derive(Clone)]
pub enum PingPongEvent {
    Init,
    Ping(u8),
    Pong(u8),
}

#[derive(Clone)]
pub enum PingPongMetrics {
    Init,
    Ping(u8),
    Pong(u8, usize),
}

pub struct PingPong {
    counter: u8,
    sleep: bool,
}

impl Node for PingPong {
    type EventType = PingPongEvent;
    type MetricsEventType = PingPongMetrics;

    fn run(&mut self, event: Event<PingPongEvent>,
           mut env: Environment<PingPongEvent, PingPongMetrics>) -> bool {
        // Start local processing timer.
        let mut timer = if self.sleep { Timer::new() } else { Timer::new_stopped() };
        match event.inner() {
            PingPongEvent::Init => {
                // Send first Ping event to all peers.
                let peers = env.peers().into_owned();
                for &peer in peers.iter() {
                    // Note that event has been sent out.
                    env.note_event(&PingPongMetrics::Init, env.time());

                    env.send_to(peer, PingPongEvent::Ping(self.counter));
                    self.counter += 1;
                }
                true
            },
            PingPongEvent::Ping(i) => {
                // Do something for 200ms.
                if self.sleep {
                    sleep(Duration::from_millis(200));
                } else {
                    timer.advance(Duration::from_millis(200));
                }

                // Received Ping, will reply with Pong.
                env.advance_time(timer.elapsed());
                env.send_to(event.from(), PingPongEvent::Pong(*i));
                true
            },
            PingPongEvent::Pong(i) => {
                // Stop after sending 50 messages.
                if self.counter >= 50 {
                    return false;
                }

                // Note that event has been received.
                env.note_event(&PingPongMetrics::Pong(*i, event.from()), env.time());

                // Do something for 200ms.
                if self.sleep {
                    sleep(Duration::from_millis(200));
                } else {
                    timer.advance(Duration::from_millis(200));
                }

                // Received Pong, will reply with Ping.
                env.advance_time(timer.elapsed());

                // Note that event has been sent out.
                env.note_event(&PingPongMetrics::Ping(self.counter), env.time());
                env.send_to(event.from(), PingPongEvent::Ping(self.counter));

                self.counter += 1;
                true
            },
        }
    }
}

pub struct Network {
    network: [[Option<u64>; 3]; 3],
    adjacency: Vec<Vec<usize>>,
    sleep: bool,
}

impl Network {
    pub fn new(sleep: bool) -> Self {
        Network {
            network: [
                [None, Some(200), Some(400)],
                [Some(200), None, None],
                [Some(400), None, None],
            ],
            adjacency: Vec::new(),
            sleep,
        }
    }

    pub fn init(&mut self) {
        for latencies in self.network.iter() {
            self.adjacency.push(latencies.iter().enumerate()
                .filter_map(|(i, &connected)| {
                    if connected.is_some() {
                        Some(i)
                    } else {
                        None
                    }
                }).collect::<Vec<usize>>());
        }
    }
}

impl NetworkConfig for Network {
    type EventType = PingPongEvent;
    type MetricsEventType = PingPongMetrics;

    fn num_nodes(&self) -> usize {
        self.network.len()
    }

    fn adjacent(&self, from: usize) -> Cow<Vec<usize>> {
        Cow::Borrowed(&self.adjacency[from])
    }

    fn transmission_delay(&self, from: usize, to: usize, _event: &PingPongEvent) -> Option<Duration> {
        self.network.get(from)?.get(to)?.map(Duration::from_millis)
    }

    fn node(&self, _id: usize) -> Box<Node<EventType=Self::EventType, MetricsEventType=Self::MetricsEventType>> {
        Box::new(PingPong {
            counter: 0,
            sleep: self.sleep,
        })
    }
}

fn duration_to_millis(duration: Duration) -> u64 {
    duration.as_secs() * 1_000u64 + duration.subsec_millis() as u64
}

fn main() {
    println!("Simulating three parties and ping pong!");
    let metrics = DefaultMetrics::default();
    // Setting this to true will cause the program to take much longer with the same result.
    let mut network = Network::new(false);
    network.init();
    let mut simulator = Simulator::new(network, metrics);

    simulator.build();

    simulator.initial_event(0, PingPongEvent::Init);

    simulator.run();

    println!("Simulation ended, analyzing metrics.");

    let events = &simulator.metrics().events;

    let mut pings = HashMap::new();
    let mut pongs = HashMap::new();
    for (event, time) in events.iter() {
        match event {
            PingPongMetrics::Ping(i) => {
                pings.insert(*i, *time);
            },
            PingPongMetrics::Pong(i, from) => {
                pongs.insert(*i, (*time, *from));
            },
            _ => (),
        }
    }

    // Average round trip time.
    let mut sorted = HashMap::new();
    for (i, ping) in pings.iter() {
        let pong = pongs.get(i);
        if let Some((pong, from)) = pong {
            let duration = *pong - *ping;
            sorted.entry(from)
                .or_insert_with(Vec::new)
                .push(duration);
        }
    }

    let mut total_duration = 0;
    let mut total_count = 0;
    for (k, v) in sorted.iter() {
        let duration: Duration = v.iter().sum();
        let duration = duration_to_millis(duration);
        total_duration += duration;
        total_count += v.len();
        println!("Average round trip time for 0 - {} was: {}ms", k, (duration as f64 / v.len() as f64));
    }

    println!("Average round trip time was: {}ms", (total_duration as f64 / total_count as f64));
}
