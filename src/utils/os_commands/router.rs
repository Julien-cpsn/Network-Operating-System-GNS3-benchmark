use crate::models::network_stack::NetworkStack;
use crate::models::nic::{NicIndex, NicType};
use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::OsCommand;
use crate::models::routing_stack::RoutingStack;
use cidr::Ipv4Inet;
use std::collections::HashMap;

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

pub fn router_start_network_stack_commands(os: &OperatingSystem, network_stack: &NetworkStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for start in &network_stack.start {
        let command = start.to_os_command(&os, None);
        commands.push(command)
    }

    commands.push(OsCommand::new_line(&os.input_ready));

    commands
}

pub fn router_stop_network_stack_commands(os: &OperatingSystem, network_stack: &NetworkStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for stop in &network_stack.stop {
        let command = stop.to_os_command(&os, None);
        commands.push(command)
    }

    commands.push(OsCommand::new_line(&os.input_ready));

    commands
}

pub fn router_add_ip_address_commands(os: &OperatingSystem, network_stack: &NetworkStack, nic_index: &NicIndex, nic_type: &NicType, ip_address: &Ipv4Inet) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let interface_index =  os.gap_between_interfaces * (os.interfaces_start_at + nic_index.to_u16()? as i16) as u16;

    let to_replace = HashMap::from([
        ("{IP_ADDRESS}", ip_address.address().to_string()),
        ("{INTERFACE_PREFIX}", os.interface_prefix(&nic_type)?),
        ("{INTERFACE}", interface_index.to_string()),
        ("{MASK}", ip_address.mask().to_string()),
        ("{NETWORK_LENGTH}", ip_address.network_length().to_string()),
    ]);

    for add_ip_address in &network_stack.add_ip_address {
        let command = add_ip_address.to_os_command(&os, Some(&to_replace));
        commands.push(command)
    }

    commands.push(OsCommand::new_line(&os.input_ready));

    Ok(commands)
}

pub fn router_start_routing_stack_commands(os: &OperatingSystem, routing_stack: &RoutingStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for start in &routing_stack.start {
        let command = start.to_os_command(&os, None);
        commands.push(command)
    }

    commands.push(OsCommand::new_line(&os.input_ready));

    commands
}

pub fn router_stop_routing_stack_commands(os: &OperatingSystem, routing_stack: &RoutingStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for stop in &routing_stack.stop {
        let command = stop.to_os_command(&os, None);
        commands.push(command)
    }

    commands.push(OsCommand::new_line(&os.input_ready));

    commands
}