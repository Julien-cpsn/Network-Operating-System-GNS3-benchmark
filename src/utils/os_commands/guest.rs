use crate::models::nodes::node::DistantNetwork;
use crate::models::os_command::OsCommand;
use crate::models::test::Test;
use std::net::Ipv4Addr;

pub const GUEST_INPUT_READY: &str = ":~";
pub const GUEST_NIC_INDEX: u32 = 4;

pub fn guest_config_commands(ip_address: Ipv4Addr) -> Vec<OsCommand> {
    vec![
        OsCommand::new("ogin:", "root"),
        OsCommand::new("assword:", "debian"),
        OsCommand::new(":~", "ip link set ens4 up"),
        OsCommand::new(":~", format!("ip address add {}/24 dev ens4", &ip_address)),
    ]
}

pub fn guest_add_route_commands(distant_network: &DistantNetwork) -> Vec<OsCommand> {
    vec![
        OsCommand::new(":~", format!("ip route add {} via {} dev ens4", &distant_network.network, &distant_network.gateway)),
    ]
}

pub fn guest_test_commands(experiment_name: &str, test: &Test, server_ip: Ipv4Addr) -> Vec<OsCommand> {
    vec![
        OsCommand::new(":~", "mkdir /mnt/shared"),
        OsCommand::new(":~", "mount -t 9p -o trans=virtio,version=9p2000.L shared_folder /mnt/shared"),
        OsCommand::new(":~", format!("flent {} -t \"{}\" -l {} -H {}", test.test, experiment_name, test.duration, server_ip)),
        OsCommand::new(":~", "cp *.flent.gz /mnt/shared"),
        OsCommand::new(":~", ""),
    ]
}