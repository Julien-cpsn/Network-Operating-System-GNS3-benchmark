use crate::models::nic::NicType;
use crate::models::os_command::DeserializedOsCommandType;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystem {
    pub input_ready: String,
    pub trigger_sequence: Option<String>,
    pub login: Option<String>,
    pub password: Option<String>,
    pub network_stack: String,
    pub routing_stack: Option<String>,
    pub interface_prefix: InterfacePrefix,
    #[serde(default)]
    pub interfaces_start_at: i16,
    #[serde(default = "gap_between_interfaces")]
    pub gap_between_interfaces: u16,
    pub image_path: PathBuf,
    pub resources_monitor_commands: Option<Vec<DeserializedOsCommandType>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterfacePrefix {
    Simple(String),
    Custom(Vec<(NicType, String)>)
}

impl OperatingSystem {
    pub fn image_name(&self) -> String {
        self.image_path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn interface_prefix(&self, nic_type: &NicType) -> anyhow::Result<String> {
        match &self.interface_prefix {
            InterfacePrefix::Simple(prefix) => Ok(prefix.to_owned()),
            InterfacePrefix::Custom(custom) => match custom.iter().find(|(nt, _)| nt == nic_type) {
                Some((_, prefix)) => Ok(prefix.to_owned()),
                None => Err(anyhow!("Interface type \"{}\" does not exist", nic_type))?,
            }
        }
    }
}

fn gap_between_interfaces() -> u16 {
    1
}