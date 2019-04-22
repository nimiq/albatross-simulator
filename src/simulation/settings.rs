use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub main: MainSettings,
    pub regions: HashMap<String, RegionSettings>,
}

impl Settings {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Settings, Error> {
        let settings: Settings = toml::from_str(read_to_string(path)?.as_ref())?;

        // Check settings for consistency.
        // That means:
        // 1. |main.regions| = |main.region_distribution|
        if settings.main.regions.len() != settings.main.region_distribution.len() {
            return Err(Error::SizeMismatch("|main.regions| != |main.region_distribution|".to_string()));
        }

        // 2. |regions| = |main.regions|
        if settings.main.regions.len() != settings.regions.len() {
            return Err(Error::SizeMismatch("|regions| != |main.regions|".to_string()));
        }

        // 3. |main.connections_distribution_intervals| = |main.connections_distribution_weights| + 1
        if settings.main.connections_distribution_intervals.len() != settings.main.connections_distribution_weights.len() + 1 {
            return Err(Error::SizeMismatch("|main.connections_distribution_intervals| != |main.connections_distribution_weights| + 1".to_string()));
        }

        for region_name in settings.main.regions.iter() {
            // 4. Region exists
            let region = settings.regions.get(region_name)
                .ok_or_else(|| Error::RegionMissing(region_name.clone()))?;

            // 5. |r.latencies| = |main.regions|
            if settings.main.regions.len() != region.latencies.len() {
                return Err(Error::SizeMismatch(format!("|{}.latencies| != |main.regions|", region_name)));
            }

            // 6. |main.download_bandwidth_intervals| = |r.download_bandwidth_weights| + 1
            if settings.main.download_bandwidth_intervals.len() != region.download_bandwidth_weights.len() + 1 {
                return Err(Error::SizeMismatch(format!("|main.download_bandwidth_intervals| != |{}.download_bandwidth_weights| + 1", region_name)));
            }

            // 7. |main.upload_bandwidth_intervals| = |r.upload_bandwidth_weights| + 1
            if settings.main.upload_bandwidth_intervals.len() != region.upload_bandwidth_weights.len() + 1 {
                return Err(Error::SizeMismatch(format!("|main.upload_bandwidth_intervals| != |{}.upload_bandwidth_weights| + 1", region_name)));
            }
        }

        Ok(settings)
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct RegionSettings {
    pub latencies: Vec<f64>,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub download_bandwidth_weights: Vec<u64>,
    pub upload_bandwidth_weights: Vec<u64>,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct MainSettings {
    pub regions: Vec<String>,
    pub region_distribution: Vec<f64>,
    pub connections_distribution_intervals: Vec<usize>,
    pub connections_distribution_weights: Vec<u64>,
    pub download_bandwidth_intervals: Vec<f64>,
    pub upload_bandwidth_intervals: Vec<f64>,

    pub min_connections_per_node: usize,
    pub max_connections_per_node: usize,
    pub min_connections_per_validator: usize,
    pub max_connections_per_validator: usize,
    pub latency_pareto_shape_divider: f64,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct SignatureTimingSettings {
    pub signing: u64,
    pub verification: u64,
    pub batch_verification: u64,
    pub generate_aggregate_signature_same_message: u64,
    pub generate_aggregate_public_key: u64,
    pub verify_aggregate_signature_same_message: u64,
    pub generate_aggregate_signature_distinct_message: u64,
    pub verify_aggregate_signature_distinct_message: u64,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct ProtocolSettings {
    pub micro_block_timeout: u64,
    pub macro_block_timeout: u64,

    pub num_micro_blocks: u32,
}

impl ProtocolSettings {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<ProtocolSettings, Error> {
        let settings: ProtocolSettings = toml::from_str(read_to_string(path)?.as_ref())?;
        Ok(settings)
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub(crate) struct TimingSettings {
    pub signatures: SignatureTimingSettings,
}

impl TimingSettings {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<TimingSettings, Error> {
        let settings: TimingSettings = toml::from_str(read_to_string(path)?.as_ref())?;
        Ok(settings)
    }
}

#[derive(Debug)]
pub(crate) enum Error {
    Toml(toml::de::Error),
    Io(std::io::Error),
    SizeMismatch(String),
    RegionMissing(String),
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Toml(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}
