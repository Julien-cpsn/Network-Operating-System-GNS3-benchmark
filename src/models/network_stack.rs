use serde::Deserialize;
use crate::models::os_command::DeserializedOsCommandType;

#[derive(Debug, Deserialize)]
pub struct NetworkStack {
    pub start: Vec<DeserializedOsCommandType>,
    pub add_ip_address: Vec<DeserializedOsCommandType>,
    pub add_static_route: Vec<DeserializedOsCommandType>,
    pub stop: Vec<DeserializedOsCommandType>,
}