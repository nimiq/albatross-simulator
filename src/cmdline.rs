use std::str::FromStr;
use std::time::Duration;

use clap::{App, Arg, Values};

#[derive(Debug, Copy, Clone)]
pub(crate) enum ParseError {
    NumNodes,
    NumMicroBlocks,
    Blocks,
    Iterations,
    MicroBlockTimeout,
    MacroBlockTimeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Options {
    pub num_nodes: Vec<usize>,
    pub num_micro_blocks: Option<u32>,
    pub blocks: u32,
    pub iterations: usize,
    pub network_settings: Option<String>,
    pub timing_settings: Option<String>,
    pub protocol_settings: Option<String>,
    pub trace_file: Option<String>,

    pub micro_block_timeout: Option<Duration>,
    pub macro_block_timeout: Option<Duration>,
}


impl Options {
    fn create_app<'a, 'b>() -> App<'a, 'b> {
        App::new("albatross-simulator")
            .version("0.1.0")
            .about("Albatross Simulator")
            .author("Pascal Berrang <mail@paberr.net>")
            // Configuration
            .arg(Arg::with_name("num_nodes")
                .value_name("NUM_NODES")
                .help("Number of nodes in the network (currently equal to number of validators).")
                .takes_value(true)
                .required(true)
                .use_delimiter(true))
            .arg(Arg::with_name("num_micro_blocks")
                .long("num_micro_blocks")
                .value_name("NUM_MICRO_BLOCKS")
                .help("Number of micro blocks between macro blocks.")
                .takes_value(true)
                .use_delimiter(true))
            .arg(Arg::with_name("blocks")
                .value_name("BLOCKS")
                .help("Number of blocks to be simulated")
                .required(true)
                .takes_value(true))
            // Options
            .arg(Arg::with_name("iterations")
                .long("iterations")
                .value_name("ITERATIONS")
                .help("Number of iterations of the simulation.")
                .default_value("1"))
            .arg(Arg::with_name("network_settings")
                .long("network_settings_file")
                .short("n")
                .value_name("NETWORK_SETTINGS_FILE")
                .help("Path to the network settings.")
                .default_value("./config/network-distributions.toml")
                .takes_value(true))
            .arg(Arg::with_name("timing_settings")
                .long("timing_settings_file")
                .short("t")
                .value_name("TIMING_SETTINGS_FILE")
                .help("Path to the timing settings.")
                .default_value("./config/timing.toml")
                .takes_value(true))
            .arg(Arg::with_name("protocol_settings")
                .long("protocol_settings_file")
                .short("p")
                .value_name("PROTOCOL_SETTINGS_FILE")
                .help("Path to the protocol settings.")
                .default_value("./config/protocol.toml")
                .takes_value(true))
            .arg(Arg::with_name("trace_file")
                .long("trace_file")
                .short("l")
                .value_name("TRACE_FILE")
                .help("Allows to store all events in a trace file (only useful for a single iteration and configuration only).")
                .takes_value(false))
            .arg(Arg::with_name("micro_block_timeout")
                .long("micro_block_timeout")
                .value_name("MICRO_BLOCK_TIMEOUT")
                .help("Allows to override the micro block timeout from the timing config.")
                .takes_value(true))
            .arg(Arg::with_name("macro_block_timeout")
                .long("macro_block_timeout")
                .value_name("MACRO_BLOCK_TIMEOUT")
                .help("Allows to override the macro block timeout from the timing config.")
                .takes_value(true))
    }

    /// Parses a command line option from a string into `T` and returns `error`, when parsing fails.
    fn parse_option<T: FromStr>(value: Option<&str>, error: ParseError) -> Result<Option<T>, ParseError> {
        match value {
            None => Ok(None),
            Some(s) => match T::from_str(s.trim()) {
                Err(_) => Err(error), // type of _: <T as FromStr>::Err
                Ok(v) => Ok(Some(v))
            }
        }
    }

    /// Parses a non-optional command line option from a string into `T` and returns `error`, when parsing fails.
    fn parse_value<T: FromStr>(value: Option<&str>, error: ParseError) -> Result<T, ParseError> {
        match value {
            None => Err(error),
            Some(s) => match T::from_str(s.trim()) {
                Err(_) => Err(error),
                Ok(v) => Ok(v)
            }
        }
    }

    /// Parses a command line option from a string into `Vec<T>` and returns `error`, when parsing fails.
    fn parse_values<T: FromStr>(values: Option<Values>, error: ParseError) -> Result<Vec<T>, ParseError> {
        match values {
            None => Ok(Vec::new()),
            Some(values) => values
                .map(|value|
                    Self::parse_value(Some(value), error)
                ).collect(),
        }
    }

    fn parse_option_string(value: Option<&str>) -> Option<String> {
        value.map(String::from)
    }

    pub fn parse() -> Result<Options, ParseError> {
        let app = Self::create_app();
        let matches = app.get_matches();

        Ok(Options {
            num_nodes: Self::parse_values::<usize>(matches.values_of("num_nodes"), ParseError::NumNodes)?,
            num_micro_blocks: Self::parse_option::<u32>(matches.value_of("num_micro_blocks"), ParseError::NumMicroBlocks)?,
            blocks: Self::parse_value::<u32>(matches.value_of("blocks"), ParseError::Blocks)?,
            iterations: Self::parse_value::<usize>(matches.value_of("iterations"), ParseError::Iterations)?,
            network_settings: Self::parse_option_string(matches.value_of("network_settings")),
            timing_settings: Self::parse_option_string(matches.value_of("timing_settings")),
            protocol_settings: Self::parse_option_string(matches.value_of("protocol_settings")),
            trace_file: Self::parse_option_string(matches.value_of("trace_file")),
            micro_block_timeout: Self::parse_option::<u64>(matches.value_of("micro_block_timeout"), ParseError::MicroBlockTimeout)?
                .map(Duration::from_micros),
            macro_block_timeout: Self::parse_option::<u64>(matches.value_of("macro_block_timeout"), ParseError::MacroBlockTimeout)?
                .map(Duration::from_micros),
        })
    }
}