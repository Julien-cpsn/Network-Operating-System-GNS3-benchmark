use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Test {
    pub name: String,
    pub test: String,
    pub fire_at: u64,
    pub duration: u64,
    pub from: String,
    pub to: String,
}