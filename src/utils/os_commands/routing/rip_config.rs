use crate::models::nic::{Nic, NicIndex, NicType};
use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::OsCommand;
use crate::models::routes::rip_config::RipConfig;
use crate::models::routing_stack::{RipCommands, RoutingStack};
use cidr::Ipv4Inet;
use std::collections::HashMap;
use anyhow::anyhow;
use indexmap::IndexMap;

pub fn router_configure_rip_commands(router_name: &str, os: &OperatingSystem, routing_stack: &RoutingStack, rip_config: &RipConfig, nics: &IndexMap<NicIndex, Nic>) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let Some(rip_commands) = &routing_stack.rip else {
        return Ok(commands)
    };
    
    for interface_to_enable in &rip_config.interfaces_to_enable {
        let nic = nics.get(interface_to_enable.to_string().as_str()).ok_or_else(|| anyhow!("NIC index {} found in router \"{}\"", &interface_to_enable, &router_name))?;
        let enable_interfaces_commands = router_enable_rip_interface_commands(&os, &rip_commands, *interface_to_enable, &nic.nic_type)?;
        commands.extend(enable_interfaces_commands);
    }
    
    for network_to_add in &rip_config.networks_to_add {
        let add_networks_commands = router_add_rip_network_commands(&os, &rip_commands, &network_to_add);
        commands.extend(add_networks_commands);
    }

    Ok(commands)
}

fn router_enable_rip_interface_commands(os: &OperatingSystem, rip_commands: &RipCommands, interface_to_enable: u16, nic_type: &NicType) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();
    
    let interface_index =  os.gap_between_interfaces * (os.interfaces_start_at + interface_to_enable as i16) as u16;

    let to_replace = HashMap::from([
        ("{INTERFACE_PREFIX}", os.interface_prefix(&nic_type)?),
        ("{INTERFACE}", interface_index.to_string()),
    ]);

    for enable_interface in &rip_commands.enable_interface {
        let command = enable_interface.to_os_command(&os, Some(&to_replace));
        commands.push(command);
    }
    
    Ok(commands)
}

fn router_add_rip_network_commands(os: &OperatingSystem, rip_commands: &RipCommands, distant_network: &Ipv4Inet) -> Vec<OsCommand> {
    let mut commands = Vec::new();
    
    let to_replace = HashMap::from([
        ("{DISTANT_NETWORK}", distant_network.to_string()),
    ]);

    for add_network in &rip_commands.add_network {
        let command = add_network.to_os_command(&os, Some(&to_replace));
        commands.push(command);
    }
    
    commands
}