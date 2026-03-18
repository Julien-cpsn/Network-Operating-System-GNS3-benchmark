use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystem {
    pub input_ready: String,
    pub trigger_sequence: Option<String>,
    pub login: Option<String>,
    pub password: Option<String>,
    pub network_stack: String,
    pub routing_stack: Option<String>,
    pub interface_prefix: String,
    pub interfaces_start_at: u32,
    pub image_path: PathBuf,
}

impl OperatingSystem {
    pub fn image_name(&self) -> String {
        self.image_path.file_name().unwrap().to_str().unwrap().to_string()
    }
}