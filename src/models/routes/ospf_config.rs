use cidr::Ipv4Inet;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OspfConfig {
    pub areas_to_add: IndexMap<String, Vec<u8>>,
    pub networks_to_add: Vec<NetworkToAdd>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkToAdd {
    pub network: Ipv4Inet,
    pub area: u8,
}