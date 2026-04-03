use crate::models::nodes::node::DistantNetwork;
use crate::models::os_command::OsCommand;
use crate::models::test::Test;
use std::net::Ipv4Addr;

pub const GUEST_INPUT_READY: &str = ":~";
pub const GUEST_NIC_INDEX: u32 = 4;

pub fn guest_config_commands(ip_address: Ipv4Addr) -> Vec<OsCommand> {
    vec![
        OsCommand::new_text("ogin:", "root", true, false),
        OsCommand::new_text("assword:", "debian", true, false),
        OsCommand::new_text(GUEST_INPUT_READY, "ip link set ens4 up", true, false),
        OsCommand::new_text(GUEST_INPUT_READY, format!("ip address add {}/24 dev ens4", &ip_address), true, false),
    ]
}

pub fn guest_add_route_commands(distant_network: &DistantNetwork) -> Vec<OsCommand> {
    vec![
        OsCommand::new_text(GUEST_INPUT_READY, format!("ip route add {} via {} dev ens4", &distant_network.network, &distant_network.gateway), true, false),
    ]
}

pub fn guest_test_commands(experiment_name: &str, test: &Test, server_ip: Ipv4Addr) -> Vec<OsCommand> {
    vec![
        OsCommand::new_text(GUEST_INPUT_READY, "mkdir /mnt/shared", true, false),
        OsCommand::new_text(GUEST_INPUT_READY, "mount -t 9p -o trans=virtio,version=9p2000.L shared_folder /mnt/shared", true, false),
        OsCommand::new_text(GUEST_INPUT_READY, format!("flent {} -t \"{}\" -l {} -H {}", test.test, experiment_name, test.duration, server_ip), true, false),
        OsCommand::new_text(GUEST_INPUT_READY, "cp *.flent.gz /mnt/shared", true, false),
        OsCommand::new_line(GUEST_INPUT_READY),
    ]
}