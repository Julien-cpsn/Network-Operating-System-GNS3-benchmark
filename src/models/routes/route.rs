use serde::{Deserialize, Serialize};
use crate::models::routes::rip_route::RipRoute;
use crate::models::routes::static_route::StaticRoute;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Route {
    Static(StaticRoute),
    Rip(RipRoute),
    Ospf,
    Bgp,
    Mpls
}