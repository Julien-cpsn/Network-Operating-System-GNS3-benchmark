use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub node_a: String,
    pub adapter_a: u32,
    pub node_b: String,
    pub adapter_b: u32,
}