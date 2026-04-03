use std::net::Ipv4Addr;
use cidr::Ipv4Cidr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticRoute {
    pub distant_network: Ipv4Cidr,
    pub gateway: Ipv4Addr,
    pub interface: u16,
}