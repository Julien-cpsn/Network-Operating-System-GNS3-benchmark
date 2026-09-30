use crate::models::network::GenericNetwork;
use serde::{Deserialize, Serialize};
use crate::models::failure_event::FailureEvent;
use crate::models::protocol::RoutingProtocol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub supported_routing_protocols: Vec<RoutingProtocol>,
    pub failure_events: Vec<FailureEvent>,
    pub network: GenericNetwork,
}