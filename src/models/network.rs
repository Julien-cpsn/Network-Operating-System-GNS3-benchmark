use crate::models::link::Link;
use crate::models::nodes::node::{GenericNode, Node};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::models::failure_event::FailureEvent;

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    pub nodes: IndexMap<String, Node>,
    pub physical_links: Vec<Link>,
    pub failure_events: Vec<FailureEvent>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericNetwork {
    pub nodes: IndexMap<String, GenericNode>,
    pub physical_links: Vec<Link>,
}