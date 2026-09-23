use crate::models::nic::NicType;
use crate::models::os_command::{CommandContext, OsCommand};
use crate::models::routes::ospf_config::OspfConfig;
use crate::models::routing_stack::OspfCommands;
use anyhow::anyhow;
use cidr::Ipv4Inet;
use std::str::FromStr;

pub fn router_configure_ospf_commands(command_context: &CommandContext, ospf_config: &OspfConfig) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let Some(ospf_commands) = &command_context.routing_stack.unwrap().ospf else {
        return Ok(commands)
    };

    let start_ospf_commands = router_start_ospf_commands(&command_context, &ospf_commands)?;
    commands.extend(start_ospf_commands);

    for (area, interfaces) in &ospf_config.areas_to_add {
        for interface in interfaces {
            let nic = command_context.router.nics.get(interface.to_string().as_str()).ok_or_else(|| anyhow!("NIC index {} found in router \"{}\"", &interface, &command_context.router_name))?;
            let add_area_commands = router_add_ospf_area_commands(&command_context, &ospf_commands, u8::from_str(area)?, *interface, &nic.nic_type)?;
            commands.extend(add_area_commands);
        }
    }

    for network_to_add in &ospf_config.networks_to_add {
        let add_network_commands = router_add_ospf_network_commands(&command_context, &ospf_commands, &network_to_add.network, network_to_add.area)?;
        commands.extend(add_network_commands);
    }

    let stop_ospf_commands = router_stop_ospf_commands(&command_context, &ospf_commands)?;
    commands.extend(stop_ospf_commands);

    commands.push(OsCommand::new_line(&command_context.os.input_ready));

    Ok(commands)
}

fn router_start_ospf_commands(command_context: &CommandContext, ospf_commands: &OspfCommands) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for start_ospf in &ospf_commands.start {
        if let Some(command) = start_ospf.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn router_add_ospf_area_commands(command_context: &CommandContext, ospf_commands: &OspfCommands, area: u8, interface_index: u8, nic_type: &NicType) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let interface_index =  command_context.os.gap_between_interfaces * (command_context.os.interfaces_start_at + interface_index as i16) as u16;

    let mut to_replace = command_context.to_replace_map(Some(&nic_type))?;
    to_replace.insert(String::from("{AREA}"), area.to_string());
    to_replace.insert(String::from("{INTERFACE}"), interface_index.to_string());

    for add_area in &ospf_commands.add_area {
        if let Some(command) = add_area.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn router_add_ospf_network_commands(command_context: &CommandContext, ospf_commands: &OspfCommands, distant_network: &Ipv4Inet, area: u8) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let mut to_replace = command_context.to_replace_map(None)?;
    to_replace.insert(String::from("{DISTANT_NETWORK}"), distant_network.to_string());
    to_replace.insert(String::from("{AREA}"), area.to_string());

    for add_network in &ospf_commands.add_network {
        if let Some(command) = add_network.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn router_stop_ospf_commands(command_context: &CommandContext, ospf_commands: &OspfCommands) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for stop_ospf in &ospf_commands.stop {
        if let Some(command) = stop_ospf.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}