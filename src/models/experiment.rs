use std::collections::HashMap;
use crate::models::network::Network;
use crate::models::test::Test;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Experiment {
    pub experiment_name: String,
    pub network: Network,
    pub test_batch: Vec<Test>,
}

impl Experiment {
    pub fn test_batch_by_node(&self) -> HashMap<String, Vec<Test>> {
        let mut result: HashMap<String, Vec<Test>> = HashMap::new();

        for test in &self.test_batch {
            result.insert(test.from.to_string(), Vec::new());
        }

        for (key, value) in result.iter_mut() {
            for test in &self.test_batch {
                if &test.from != key {
                    continue;
                }

                value.push(test.clone());
            }
        }

        result
    }
}