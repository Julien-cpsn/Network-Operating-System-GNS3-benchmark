use crate::models::nic::NicType;
use crate::models::os_command::{CommandContext, OsCommand};
use crate::models::routes::rip_config::RipConfig;
use crate::models::routing_stack::RipCommands;
use anyhow::anyhow;
use cidr::Ipv4Inet;

pub fn router_configure_rip_commands(command_context: &CommandContext, rip_config: &RipConfig) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let Some(rip_commands) = &command_context.routing_stack.unwrap().rip else {
        return Ok(commands)
    };
    
    let start_rip_commands = router_start_rip_commands(&command_context, &rip_commands)?;
    commands.extend(start_rip_commands);
    
    for interface_to_enable in &rip_config.interfaces_to_enable {
        let nic = command_context.router.nics.get(interface_to_enable.to_string().as_str()).ok_or_else(|| anyhow!("NIC index {} found in router \"{}\"", &interface_to_enable, &command_context.router_name))?;
        let enable_interfaces_commands = router_enable_rip_interface_commands(&command_context, &rip_commands, *interface_to_enable, &nic.nic_type)?;
        commands.extend(enable_interfaces_commands);
    }
    
    for network_to_add in &rip_config.networks_to_add {
        let add_networks_commands = router_add_rip_network_commands(&command_context, &rip_commands, &network_to_add)?;
        commands.extend(add_networks_commands);
    }

    let stop_rip_commands = router_stop_rip_commands(&command_context, &rip_commands)?;
    commands.extend(stop_rip_commands);
    
    commands.push(OsCommand::new_line(&command_context.os.input_ready));
    
    Ok(commands)
}

fn router_start_rip_commands(command_context: &CommandContext, rip_commands: &RipCommands) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();
    
    let to_replace = command_context.to_replace_map(None)?;

    for start_rip in &rip_commands.start {
        if let Some(command) = start_rip.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn router_enable_rip_interface_commands(command_context: &CommandContext, rip_commands: &RipCommands, interface_to_enable: u16, nic_type: &NicType) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();
    
    let interface_index = command_context.os.gap_between_interfaces * (command_context.os.interfaces_start_at + interface_to_enable as i16) as u16;

    let mut to_replace = command_context.to_replace_map(Some(nic_type))?;
    to_replace.insert(String::from("{INTERFACE}"), interface_index.to_string());

    for enable_interface in &rip_commands.enable_interface {
        if let Some(command) = enable_interface.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }
    
    Ok(commands)
}

fn router_add_rip_network_commands(command_context: &CommandContext, rip_commands: &RipCommands, distant_network: &Ipv4Inet) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let mut to_replace = command_context.to_replace_map(None)?;
    to_replace.insert(String::from("{DISTANT_NETWORK}"), distant_network.to_string());

    for add_network in &rip_commands.add_network {
        if let Some(command) = add_network.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }
    
    Ok(commands)
}

fn router_stop_rip_commands(command_context: &CommandContext, rip_commands: &RipCommands) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for stop_rip in &rip_commands.stop {
        if let Some(command) = stop_rip.to_os_command(&command_context, Some(&to_replace)) {
            commands.push(command);
        }
    }

    Ok(commands)
}
