use serde::{Deserialize, Serialize};
use crate::models::protocol::RoutingProtocol;
use crate::models::routes::ospf_config::OspfConfig;
use crate::models::routes::rip_config::RipConfig;
use crate::models::routes::static_route::StaticRoute;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RouteConfig {
    Static(Vec<StaticRoute>),
    Rip(RipConfig),
    Ospf(OspfConfig),
    Bgp,
    Mpls
}

impl RouteConfig {
    pub fn to_protocol_name(&self) -> RoutingProtocol {
        match self {
            RouteConfig::Static(_) => RoutingProtocol::Static,
            RouteConfig::Rip(_) => RoutingProtocol::Rip,
            RouteConfig::Ospf(_) => RoutingProtocol::Ospf,
            RouteConfig::Bgp => RoutingProtocol::Bgp,
            RouteConfig::Mpls => RoutingProtocol::Mpls
        }
    }
}