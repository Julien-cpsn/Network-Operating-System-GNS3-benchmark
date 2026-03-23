use clap::ValueEnum;
use strum::{Display, VariantArray};

#[derive(Debug, Clone, ValueEnum, VariantArray, Display)]
pub enum HardwareResources {
    /// Low resources (1 vCPU, 1024 MB RAM)
    #[clap(name = "lr")]
    #[strum(to_string = "Low Resources")]
    LowResources,
    /// Medium resources (2 vCPU, 2048 MB RAM)
    #[clap(name = "mr")]
    #[strum(to_string = "Medium Resources")]
    MediumResources,
    /// High resources (4 vCPU, 8096 MB RAM)
    #[clap(name = "hr")]
    #[strum(to_string = "High Resources")]
    HighResources,
}

impl HardwareResources {
    pub fn to_vcpu_and_ram(&self) -> (u32, u32) {
        match self {
            HardwareResources::LowResources => (1, 1024),
            HardwareResources::MediumResources => (2, 2048),
            HardwareResources::HighResources => (4, 8096)
        }
    }
}