use crate::models::network_stack::NetworkStack;
use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::OsCommand;
use crate::models::routes::static_route::StaticRoute;
use std::collections::HashMap;
use crate::models::nic::NicType;

pub fn router_add_static_route_commands(os: &OperatingSystem, network_stack: &NetworkStack, static_route: &StaticRoute, nic_type: &NicType) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let interface_index =  os.gap_between_interfaces * (os.interfaces_start_at + static_route.interface as i16) as u16;
    
    let to_replace = HashMap::from([
        ("{DISTANT_NETWORK}", static_route.distant_network.to_string()),
        ("{GATEWAY}", static_route.gateway.to_string()),
        ("{INTERFACE_PREFIX}", os.interface_prefix(&nic_type)?),
        ("{INTERFACE}", interface_index.to_string()),
    ]);

    for add_static_route in &network_stack.add_static_route {
        let command = add_static_route.to_os_command(&os, Some(&to_replace));
        commands.push(command);
    }

    commands.push(OsCommand::new_line(&os.input_ready));

    Ok(commands)
}