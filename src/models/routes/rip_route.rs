use cidr::Ipv4Cidr;
use serde::{Deserialize, Serialize};
use crate::models::nic::NicIndex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipRoute {
    pub networks: Vec<Ipv4Cidr>,
    pub interfaces_to_enable: Vec<NicIndex>
}