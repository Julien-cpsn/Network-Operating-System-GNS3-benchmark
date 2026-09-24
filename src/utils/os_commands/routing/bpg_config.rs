use crate::models::os_command::{CommandContext, OsCommand};
use crate::models::routes::bgp_config::{BgpConfig, BgpNeighbor};
use crate::models::routing_stack::BgpCommands;
use cidr::Ipv4Inet;

pub fn router_configure_bgp_commands(command_context: &CommandContext, bgp_config: &BgpConfig) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let Some(bgp_commands) = &command_context.routing_stack.unwrap().bgp else {
        return Ok(commands)
    };

    let start_bgp_commands = router_start_bgp_commands(&command_context, &bgp_commands, bgp_config.local_as)?;
    commands.extend(start_bgp_commands);

    for bgp_neighbor in &bgp_config.neighbors {
        let add_neighbor_commands = router_add_bgp_neighbor_commands(&command_context, &bgp_commands, &bgp_neighbor)?;
        commands.extend(add_neighbor_commands);
    }

    for network_to_advertise in &bgp_config.networks_to_advertise {
        let add_network_to_advertise_commands = router_add_bgp_network_to_advertise_commands(&command_context, &bgp_commands, &network_to_advertise)?;
        commands.extend(add_network_to_advertise_commands);
    }

    let stop_bgp_commands = router_stop_bgp_commands(&command_context, &bgp_commands)?;
    commands.extend(stop_bgp_commands);
    
    Ok(commands)
}

fn router_start_bgp_commands(command_context: &CommandContext, bgp_commands: &BgpCommands, local_as: u32) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let mut to_replace = command_context.to_replace_map(None)?;
    to_replace.insert(String::from("{LOCAL_AS}"), local_as.to_string());

    for start_bgp in &bgp_commands.start {
        if let Some(command) = start_bgp.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn router_add_bgp_neighbor_commands(command_context: &CommandContext, bgp_commands: &BgpCommands, bgp_neighbor: &BgpNeighbor) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let mut to_replace = command_context.to_replace_map(None)?;
    to_replace.insert(String::from("{NEIGHBOR_ADDRESS}"), bgp_neighbor.address.to_string());
    to_replace.insert(String::from("{REMOTE_AS}"), bgp_neighbor.remote_as.to_string());

    for add_neighbor in &bgp_commands.add_neighbor {
        if let Some(command) = add_neighbor.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn router_add_bgp_network_to_advertise_commands(command_context: &CommandContext, bgp_commands: &BgpCommands, network: &Ipv4Inet) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let mut to_replace = command_context.to_replace_map(None)?;
    to_replace.insert(String::from("{DISTANT_NETWORK}"), network.to_string());

    for add_network_to_advertise in &bgp_commands.add_network_to_advertise {
        if let Some(command) = add_network_to_advertise.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn router_stop_bgp_commands(command_context: &CommandContext, bgp_commands: &BgpCommands) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for stop_bgp in &bgp_commands.stop {
        if let Some(command) = stop_bgp.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}