use crate::models::nic::NicType;
use crate::models::os_command::{CommandContext, OsCommand};
use crate::models::routes::static_route::StaticRoute;

pub fn router_add_static_route_commands(command_context: &CommandContext, static_route: &StaticRoute, nic_type: &NicType) -> anyhow::Result<Vec<OsCommand>> {
    let mut commands = Vec::new();

    let interface_index =  command_context.os.gap_between_interfaces * (command_context.os.interfaces_start_at + static_route.interface as i16) as u16;

    let mut to_replace = command_context.to_replace_map(Some(&nic_type))?;
    to_replace.insert(String::from("{DISTANT_NETWORK}"), static_route.distant_network.to_string());
    to_replace.insert(String::from("{GATEWAY}"), static_route.gateway.to_string());
    to_replace.insert(String::from("{INTERFACE}"), interface_index.to_string());

    let network_stack = command_context.network_stack.as_ref().unwrap();
    for add_static_route in &network_stack.add_static_route {
        if let Some(command) = add_static_route.to_os_command(&command_context, Some(&to_replace)) {
        commands.push(command);
        }
    }

    commands.push(OsCommand::new_line(&command_context.os.input_ready));

    Ok(commands)
}