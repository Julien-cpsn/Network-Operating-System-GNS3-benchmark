use crate::models::network::Network;
use crate::models::test::Test;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Experiment {
    pub experiment_name: String,
    pub plot_legend_when_merged: String,
    pub network: Network,
    pub test_batch: Vec<Test>,
}