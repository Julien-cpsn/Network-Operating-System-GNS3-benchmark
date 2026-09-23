use crate::models::network_stack::NetworkStack;
use crate::models::nic::{NicIndex, NicType};
use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::{CommandContext, OsCommand};
use crate::models::routing_stack::RoutingStack;
use cidr::Ipv4Inet;

pub fn router_login_commands(os: &OperatingSystem) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    if let Some(trigger_sequence) = &os.trigger_sequence {
        commands.push(OsCommand::new_text(trigger_sequence, "\n", true, false));
    }

    if let Some(login) = &os.login {
        commands.push(OsCommand::new_text("ogin:", login, true, false));
    }

    if let Some(password) = &os.password {
        commands.push(OsCommand::new_text("assword:", password, true, false));
    }

    commands.push(OsCommand::new_line(&os.input_ready));

    commands
}

pub fn router_start_network_stack_commands(command_context: &CommandContext, network_stack: &NetworkStack) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for start in &network_stack.start {
        if let Some(command) = start.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }

    Ok(commands)
}

pub fn router_stop_network_stack_commands(command_context: &CommandContext, network_stack: &NetworkStack) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for stop in &network_stack.stop {
        if let Some(command) = stop.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }

    Ok(commands)
}

pub fn router_add_ip_address_commands(command_context: &CommandContext, nic_index: &NicIndex, nic_type: &NicType, ip_address: &Ipv4Inet) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let interface_index = command_context.os.gap_between_interfaces * (command_context.os.interfaces_start_at + nic_index.to_u16()? as i16) as u16;

    let mut to_replace = command_context.to_replace_map(Some(&nic_type))?;
    to_replace.insert(String::from("{IP_ADDRESS}"), ip_address.address().to_string());
    to_replace.insert(String::from("{INTERFACE}"), interface_index.to_string());
    to_replace.insert(String::from("{MASK}"), ip_address.mask().to_string());
    to_replace.insert(String::from("{NETWORK_LENGTH}"), ip_address.network_length().to_string());

    let network_stack = command_context.network_stack.as_ref().unwrap();
    for add_ip_address in &network_stack.add_ip_address {
        if let Some(command) = add_ip_address.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }

    Ok(commands)
}

pub fn router_start_routing_stack_commands(command_context: &CommandContext, routing_stack: &RoutingStack) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for start in &routing_stack.start {
        if let Some(command) = start.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }

    Ok(commands)
}

pub fn router_stop_routing_stack_commands(command_context: &CommandContext, routing_stack: &RoutingStack) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let to_replace = command_context.to_replace_map(None)?;

    for stop in &routing_stack.stop {
        if let Some(command) = stop.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }

    Ok(commands)
}