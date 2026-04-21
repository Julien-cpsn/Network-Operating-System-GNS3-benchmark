use anyhow::anyhow;
use cidr::Ipv4Inet;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use strum::{Display, VariantArray};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nic {
    pub nic_type: NicType,
    pub ip_address: Ipv4Inet
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct NicIndex(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ValueEnum, VariantArray, Display)]
pub enum NicType {
    #[strum(to_string = "RTL8139")]
    #[serde(rename = "RTL8139")]
    Rtl8139,
    #[strum(to_string = "E1000")]
    #[serde(rename = "E1000")]
    E1000,
    #[strum(to_string = "VirtIO")]
    #[serde(rename = "VirtIO")]
    VirtIO
}

impl NicIndex {
    pub fn to_u16(&self) -> anyhow::Result<u16> {
        match self.0.parse::<u16>() {
            Ok(i) => Ok(i),
            Err(err) => Err(anyhow!(err))
        }
    }
}

impl Borrow<str> for NicIndex {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

impl NicType {
    pub fn to_qemu_name(&self) -> String {
        match self {
            NicType::Rtl8139 => String::from("rtl8139"),
            NicType::E1000 => String::from("e1000"),
            NicType::VirtIO => String::from("virtio-net-pci")
        }
    }
}