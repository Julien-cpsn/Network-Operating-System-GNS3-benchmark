use crate::models::nic::{Nic, NicIndex};
use crate::models::protocol::RoutingProtocol;
use crate::models::routes::route::Route;
use cidr::Ipv4Inet;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Router {
    pub os_name: String,
    pub number_nics: u32,
    pub nics: IndexMap<NicIndex, Nic>,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericRouter {
    pub number_nics: u32,
    pub ips: IndexMap<NicIndex, Ipv4Inet>,
    /// E.g. RIP and its associated routes
    pub routes: IndexMap<RoutingProtocol, Vec<Route>>
}