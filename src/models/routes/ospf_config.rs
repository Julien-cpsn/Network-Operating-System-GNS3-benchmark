use cidr::Ipv4Inet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OspfConfig {
    pub router_id: String,
    pub networks_to_add: Vec<(Ipv4Inet, u8)>,
}