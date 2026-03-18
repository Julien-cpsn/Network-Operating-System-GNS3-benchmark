use crate::models::link::Link;
use crate::models::nodes::node::{GenericNode, Node};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    pub nodes: IndexMap<String, Node>,
    pub physical_links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericNetwork {
    pub nodes: IndexMap<String, GenericNode>,
    pub physical_links: Vec<Link>,
}