use std::net::Ipv4Addr;
use cidr::Ipv4Inet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgpConfig {
    pub local_as: u32,
    pub neighbors: Vec<BgpNeighbor>,
    pub networks_to_advertise: Vec<Ipv4Inet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgpNeighbor {
    pub address: Ipv4Addr,
    pub remote_as: u32,
}