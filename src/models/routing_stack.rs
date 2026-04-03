use serde::Deserialize;
use crate::models::os_command::DeserializedOsCommandType;

#[derive(Debug, Deserialize)]
pub struct RoutingStack {
    pub start: Vec<DeserializedOsCommandType>,
    pub stop: Vec<DeserializedOsCommandType>,
}