use cidr::Ipv4Inet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipConfig {
    pub interfaces_to_enable: Vec<u16>,
    pub networks_to_add: Vec<Ipv4Inet>,
}