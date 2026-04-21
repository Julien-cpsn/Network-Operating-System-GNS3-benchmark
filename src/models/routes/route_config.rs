use serde::{Deserialize, Serialize};
use crate::models::routes::rip_config::RipConfig;
use crate::models::routes::static_route::StaticRoute;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RouteConfig {
    Static(Vec<StaticRoute>),
    Rip(RipConfig),
    Ospf,
    Bgp,
    Mpls
}