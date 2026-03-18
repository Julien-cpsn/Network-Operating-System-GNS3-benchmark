use crate::models::network_stack::NetworkStack;
use crate::models::nic::NicIndex;
use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::OsCommand;
use crate::utils::os_commands::utils::format_command;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use crate::models::routing_stack::RoutingStack;

pub fn router_login_commands(os: &OperatingSystem) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    if let Some(trigger_sequence) = &os.trigger_sequence {
        commands.push(OsCommand::new(trigger_sequence, "\n"));
    }

    if let Some(login) = &os.login {
        commands.push(OsCommand::new("ogin:", login));
    }

    if let Some(password) = &os.password {
        commands.push(OsCommand::new("assword:", password));
    }

    commands.push(OsCommand::new(&os.input_ready, ""));

    commands
}

pub fn router_start_network_stack_commands(os: &OperatingSystem, network_stack: &NetworkStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for start in &network_stack.start {
        commands.push(OsCommand::new(&os.input_ready, start));
    }

    commands.push(OsCommand::new(&os.input_ready, ""));

    commands
}

pub fn router_stop_network_stack_commands(os: &OperatingSystem, network_stack: &NetworkStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for start in &network_stack.stop {
        commands.push(OsCommand::new(&os.input_ready, start));
    }

    commands
}

pub fn router_add_ip_address_commands(os: &OperatingSystem, network_stack: &NetworkStack, nic_index: &NicIndex, ip_address: Ipv4Addr) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let interface_index = os.interfaces_start_at + nic_index.0.parse::<u32>()?;
    let to_replace = HashMap::from([
        ("{IP_ADDRESS}", ip_address.to_string()),
        ("{INTERFACE_PREFIX}", os.interface_prefix.clone()),
        ("{INTERFACE}", interface_index.to_string()),
    ]);

    for add_ip_address in &network_stack.add_ip_address {
        commands.push(OsCommand::new(&os.input_ready, format_command(add_ip_address, &to_replace)));
    }

    Ok(commands)
}

pub fn router_start_routing_stack_commands(os: &OperatingSystem, routing_stack: &RoutingStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for start in &routing_stack.start {
        commands.push(OsCommand::new(&os.input_ready, start));
    }

    commands
}

pub fn router_stop_routing_stack_commands(os: &OperatingSystem, routing_stack: &RoutingStack) -> Vec<OsCommand> {
    let mut commands = Vec::new();

    for start in &routing_stack.stop {
        commands.push(OsCommand::new(&os.input_ready, start));
    }

    commands
}