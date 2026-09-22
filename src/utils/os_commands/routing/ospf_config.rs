use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::OsCommand;
use crate::models::routes::ospf_config::OspfConfig;
use crate::models::routing_stack::{OspfCommands, RoutingStack};
use cidr::Ipv4Inet;
use std::collections::HashMap;

pub fn router_configure_ospf_commands(os: &OperatingSystem, routing_stack: &RoutingStack, ospf_config: &OspfConfig) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    let Some(ospf_commands) = &routing_stack.ospf else {
        return commands
    };

    let set_router_id_commands = router_set_ospf_router_id_commands(&os, &ospf_commands, &ospf_config.router_id);
    commands.extend(set_router_id_commands);

    for (network_to_add, area) in &ospf_config.networks_to_add {
        let add_networks_commands = router_add_ospf_network_commands(&os, &ospf_commands, &network_to_add, *area);
        commands.extend(add_networks_commands);
    }

    commands
}

fn router_set_ospf_router_id_commands(os: &OperatingSystem, ospf_commands: &OspfCommands, router_id: &str) -> Vec<OsCommand> {
    let mut commands = Vec::new();


    let to_replace = HashMap::from([
        ("{ROUTER_ID}", router_id.to_string()),
    ]);

    for set_router_id in &ospf_commands.set_router_id {
        if let Some(command) = set_router_id.to_os_command(&os, Some(&to_replace)) {
            commands.push(command);
        }
    }

    commands
}

fn router_add_ospf_network_commands(os: &OperatingSystem, ospf_commands: &OspfCommands, distant_network: &Ipv4Inet, area: u8) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    let to_replace = HashMap::from([
        ("{DISTANT_NETWORK}", distant_network.to_string()),
        ("{AREA}", area.to_string()),
    ]);

    for add_network in &ospf_commands.add_network {
        if let Some(command) = add_network.to_os_command(&os, Some(&to_replace)) {
            commands.push(command);
        }
    }

    commands
}