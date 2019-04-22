use crate::simulation::settings::Settings;
use crate::distributions::piecewise_constant::*;
use rand::distributions::{WeightedIndex, WeightedError, Pareto, Distribution};
use rand::Rng;

pub struct AdvancedTopologyHelper<'a> {
    pub min_connections_per_node: usize,
    pub max_connections_per_node: usize,
    pub min_connections_per_validator: usize,
    pub max_connections_per_validator: usize,
    latency_pareto_shape_divider: Option<f64>,
    pub nodes_distribution: WeightedIndex<f64>,
    pub connections_distribution: PiecewiseConstant<u64, usize, usize>,
    pub regions: Vec<Region<'a>>,
}

pub struct Region<'a> {
    pub name: String,
    pub latencies: Vec<f64>,
    pub download_speed: f64,
    pub upload_speed: f64,
    pub download_bandwidth_distribution: PiecewiseConstant<u64, f64, &'a f64>,
    pub upload_bandwidth_distribution: PiecewiseConstant<u64, f64, &'a f64>,
}

impl<'a> AdvancedTopologyHelper<'a> {
    pub(crate) fn from_settings(settings: &'a mut Settings) -> Result<Self, Error> {
        let nodes_distribution = WeightedIndex::new(&settings.main.region_distribution)?;
        let connections_distribution = PiecewiseConstant::new(settings.main.connections_distribution_weights.clone(), settings.main.connections_distribution_intervals.clone())?;

        let mut regions = Vec::new();
        for region_name in settings.main.regions.clone() {
            // Existence is checked by the invariants of Settings.
            let region_settings = settings.regions.remove(&region_name).unwrap();

            let download_bandwidth_distribution = PiecewiseConstant::new(region_settings.download_bandwidth_weights, &settings.main.download_bandwidth_intervals)?;
            let upload_bandwidth_distribution = PiecewiseConstant::new(region_settings.upload_bandwidth_weights, &settings.main.upload_bandwidth_intervals)?;

            regions.push(Region {
                name: region_name,
                latencies: region_settings.latencies,
                download_speed: region_settings.download_speed,
                upload_speed: region_settings.upload_speed,
                download_bandwidth_distribution,
                upload_bandwidth_distribution,
            });
        }

        Ok(AdvancedTopologyHelper {
            min_connections_per_node: settings.main.min_connections_per_node,
            max_connections_per_node: settings.main.max_connections_per_node,
            min_connections_per_validator: settings.main.min_connections_per_validator,
            max_connections_per_validator: settings.main.max_connections_per_validator,
            latency_pareto_shape_divider: Some(settings.main.latency_pareto_shape_divider),
            nodes_distribution,
            connections_distribution,
            regions,
        })
    }

    pub fn get_latency<R: Rng + ?Sized>(&self, region1: usize, region2: usize, rng: &mut R) -> f64 {
        let latency = self.regions[region1].latencies[region2];
        if let Some(ref pareto_shape_divider) = self.latency_pareto_shape_divider {
            Pareto::new(latency, latency / pareto_shape_divider).sample(rng)
        } else {
            latency
        }
    }
}

#[derive(Debug)]
pub enum Error {
    WeightedIndexError(WeightedError),
    PiecewiseConstantError(PiecewiseConstantError),
}

impl From<WeightedError> for Error {
    fn from(e: WeightedError) -> Self {
        Error::WeightedIndexError(e)
    }
}

impl From<PiecewiseConstantError> for Error {
    fn from(e: PiecewiseConstantError) -> Self {
        Error::PiecewiseConstantError(e)
    }
}
