use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStack {
    pub start: Vec<String>,
    pub add_ip_address: Vec<String>,
    pub add_static_route: Vec<String>,
    pub stop: Vec<String>,
}