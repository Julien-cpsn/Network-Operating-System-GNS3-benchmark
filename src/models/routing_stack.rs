use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingStack {
    pub start: Vec<String>,
    pub stop: Vec<String>,
}