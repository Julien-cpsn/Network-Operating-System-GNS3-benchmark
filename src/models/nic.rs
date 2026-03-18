use cidr::Ipv4Inet;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nic {
    pub nic_type: NicType,
    pub ip_address: Ipv4Inet
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct NicIndex(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum, VariantArray, Display)]
pub enum NicType {
    #[strum(serialize = "rtl8139")]
    Rtl8139,
    #[strum(serialize = "e1000")]
    E1000,
    #[strum(to_string = "virtio-net-pci")]
    VirtIO
}