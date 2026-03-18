use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, ValueEnum, Display)]
pub enum RoutingProtocol {
    #[clap(name = "static")]
    #[strum(to_string = "static")]
    #[serde(alias = "static", alias = "STATIC")]
    Static,
    #[clap(name = "RIP")]
    #[strum(to_string = "RIP")]
    #[serde(alias = "rip", alias = "RIP")]
    Rip,
    #[clap(name = "OSPF")]
    #[strum(to_string = "OSPF")]
    #[serde(alias = "ospf", alias = "OSPF")]
    Ospf,
    #[clap(name = "BGP")]
    #[strum(to_string = "BGP")]
    #[serde(alias = "bgp", alias = "BGP")]
    Bgp,
    #[clap(name = "MPLS")]
    #[strum(to_string = "MPLS")]
    #[serde(alias = "mpls", alias = "MPLS")]
    Mpls
}