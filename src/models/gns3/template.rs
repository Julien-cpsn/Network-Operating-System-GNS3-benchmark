use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Gns3Template {
    pub name: String,
    pub category: TemplateCategory,
    pub builtin: bool,
    pub template_id: String,
    pub symbol: String,
    pub default_name_format: String,
    pub compute_id: Option<String>,
    
    #[serde(flatten)]
    pub template_type: TemplateType,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    Guest,
    Switch,
    Router
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "template_type", rename_all = "snake_case")]
pub enum TemplateType {
    Qemu(Gns3QemuTemplate),
    Cloud,
    Nat,
    Vpcs(Gns3VpcsTemplate),
    EthernetSwitch,
    EthernetHub,
    FrameRelaySwitch,
    AtmSwitch
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gns3QemuTemplate {
    pub adapter_type: String,
    pub adapters: u32,
    pub custom_adapters: Vec<String>,
    pub cpus: u32,
    pub cpu_throttling: u32,
    pub ram: u32,
    pub legacy_networking: bool,
    pub replicate_network_connection_state: bool,
    pub mac_address: String,

    pub console_type: String,
    pub console_auto_start: bool,

    pub create_config_disk: bool,
    pub hda_disk_image: String,
    pub hda_disk_interface: String,
    pub hdb_disk_image: String,
    pub hdb_disk_interface: String,
    pub hdc_disk_image: String,
    pub hdc_disk_interface: String,
    pub hdd_disk_image: String,
    pub hdd_disk_interface: String,

    pub first_port_name: String,
    pub port_name_format: String,
    pub port_segment_size: u32,
    pub process_priority: String,
    pub qemu_path: String,
    pub platform: String,
    pub on_close: String,
    pub linked_clone: bool,

    pub kernel_image: String,
    pub kernel_command_line: String,
    pub cdrom_image: String,
    pub bios_image: String,
    pub boot_priority: String,
    pub initrd: String,
    pub tpm: bool,
    pub uefi: bool,

    pub options: String,
    pub usage: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gns3VpcsTemplate {
    pub properties: HashMap<String, String>,
}