use serde::Deserialize;
use crate::models::os_command::DeserializedOsCommandType;

#[derive(Debug, Deserialize)]
pub struct RoutingStack {
    pub start: Vec<DeserializedOsCommandType>,
    #[serde(alias = "RIP")]
    pub rip: Option<RipCommands>,
    #[serde(alias = "OSPF")]
    pub ospf: Option<OspfCommands>,
    pub stop: Vec<DeserializedOsCommandType>,
}

#[derive(Debug, Deserialize)]
pub struct RipCommands {
    pub start: Vec<DeserializedOsCommandType>,
    pub enable_interface: Vec<DeserializedOsCommandType>,
    pub add_network: Vec<DeserializedOsCommandType>,
    pub stop: Vec<DeserializedOsCommandType>,
}

#[derive(Debug, Deserialize)]
pub struct OspfCommands {
    pub start: Vec<DeserializedOsCommandType>,
    pub add_area: Vec<DeserializedOsCommandType>,
    pub add_network: Vec<DeserializedOsCommandType>,
    pub stop: Vec<DeserializedOsCommandType>,
}