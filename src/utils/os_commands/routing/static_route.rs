use std::collections::HashMap;
use crate::models::network_stack::NetworkStack;
use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::OsCommand;
use crate::models::routes::static_route::StaticRoute;
use crate::utils::os_commands::utils::format_command;

pub fn router_add_static_route_commands(os: &OperatingSystem, network_stack: &NetworkStack, static_route: &StaticRoute) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let interface_index = os.interfaces_start_at + static_route.interface;

    let to_replace = HashMap::from([
        ("{DISTANT_NETWORK}", static_route.distant_network.to_string()),
        ("{GATEWAY}", static_route.gateway.to_string()),
        ("{INTERFACE_PREFIX}", os.interface_prefix.clone()),
        ("{INTERFACE}", interface_index.to_string()),
    ]);

    for add_static_route in &network_stack.add_static_route {
        commands.push(OsCommand::new(&os.input_ready, format_command(add_static_route, &to_replace)));
    }

    Ok(commands)
}