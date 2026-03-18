use cidr::Ipv4Inet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guest {
    pub ip: Ipv4Inet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericGuest {
    pub vcpu: u32,
    pub ram: u32,
    pub ip: Ipv4Inet,
}