use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use log::debug;
use rexpect::session::Options;
use crate::models::gns3::node::Gns3Node;
use crate::models::os_command::OsCommand;

const TARGET: &str = "telnet";


pub fn execute_commands_from_node(node_name: &str, gns3_node: &Gns3Node, commands: Vec<OsCommand>) -> anyhow::Result<()> {
    execute_commands(&node_name, &gns3_node.console_host(), gns3_node.console(), commands)
}

pub fn execute_commands(node_name: &str, console_host: &str, console: u32, commands: Vec<OsCommand>) -> anyhow::Result<()> {
    let mut command = Command::new("telnet");
    command.args(vec![console_host, console.to_string().as_str()]);
    let mut telnet = rexpect::spawn_with_options(
        command,
        Options::new()
            .timeout_ms(Some(180_000))
            .strip_ansi_escape_codes(true)
    )?;

    debug!(target: TARGET, "Executing commands for {}", node_name);

    telnet.send_line("")?;

    for command in &commands {
        // Cause we never know
        sleep(Duration::from_millis(100));

        let _ = telnet.exp_string(&command.expect);
        let _ = telnet.send_line(&command.send);

        if !command.send.is_empty() {
            debug!(target: TARGET, "> {}", &command.send);
        }
    }

    let _ = telnet.send_line("");
    let _ = telnet.flush();

    telnet.process_mut().exit()?;

    Ok(())
}