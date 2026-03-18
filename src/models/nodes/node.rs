use std::net::Ipv4Addr;
use cidr::Ipv4Cidr;
use crate::models::gns3::node::Gns3Node;
use crate::models::nodes::guest::{GenericGuest, Guest};
use crate::models::nodes::router::{GenericRouter, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub vcpu: u32,
    pub ram: u32,

    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,

    #[serde(flatten)]
    pub node_type: NodeType,

    #[serde(skip)]
    pub gns3_node: Option<Gns3Node>,

    #[serde(skip)]
    pub distant_networks: Vec<DistantNetwork>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Guest(Guest),
    Router(Router),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistantNetwork {
    pub network: Ipv4Cidr,
    pub gateway: Ipv4Addr,
    pub adapter_index: u32
}

impl Node {
    pub fn unwrap_guest(&self) -> &Guest {
        match &self.node_type {
            NodeType::Guest(guest) => guest,
            NodeType::Router(_) => unreachable!()
        }
    }

    pub fn unwrap_router(&self) -> &Router {
        match &self.node_type {
            NodeType::Guest(_) => unreachable!(),
            NodeType::Router(router) => router
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericNode {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,

    #[serde(flatten)]
    pub node_type: GenericNodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GenericNodeType {
    Guest(GenericGuest),
    Router(GenericRouter),
}