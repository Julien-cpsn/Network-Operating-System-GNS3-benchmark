use serde::Deserialize;
use crate::models::os_command::DeserializedOsCommandType;

#[derive(Debug, Deserialize)]
pub struct RoutingStack {
    pub start: Vec<DeserializedOsCommandType>,
    #[serde(alias = "RIP")]
    pub rip: Option<RipCommands>,
    pub stop: Vec<DeserializedOsCommandType>,
}

#[derive(Debug, Deserialize)]
pub struct RipCommands {
    pub enable_interface: Vec<DeserializedOsCommandType>,
    pub add_network: Vec<DeserializedOsCommandType>,
}