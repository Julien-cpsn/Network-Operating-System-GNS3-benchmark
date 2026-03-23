use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;
use anyhow::anyhow;
use indexmap::IndexMap;
use log::{debug, error, info, warn};
use pyo3::Python;
use tokio::task::JoinSet;
use tokio::time::sleep;
use crate::args::run::RunCommand;
use crate::GUEST_IMAGE_PATH;
use crate::models::experiment::Experiment;
use crate::models::gns3::connector::Gns3Connector;
use crate::models::network_stack::NetworkStack;
use crate::models::nodes::node::{Node, NodeType};
use crate::models::operating_system::OperatingSystem;
use crate::models::os_command::OsCommand;
use crate::models::routes::route::Route;
use crate::models::routing_stack::RoutingStack;
use crate::utils::env::harvest_env_variables;
use crate::utils::files::experiments::parse_experiments_files;
use crate::utils::files::network_stacks::parse_network_stack_list_file;
use crate::utils::files::oses::parse_os_list_file;
use crate::utils::files::results_dir::RESULT_DIR_PATH;
use crate::utils::files::routing_stacks::parse_routing_stack_list_file;
use crate::utils::files::shared_dir::SHARED_DIR_PATH;
use crate::utils::gns3::config::get_gns3_images_path;
use crate::utils::gns3::image::find_or_upload_image;
use crate::utils::gns3::node::create_node;
use crate::utils::gns3::project::{create_project, find_and_delete_projects};
use crate::utils::gns3::template::{find_and_delete_templates, generate_and_create_guest_template, generate_and_create_router_template};
use crate::utils::link::create_link;
use crate::utils::os_commands::execute::{execute_commands_from_node};
use crate::utils::os_commands::guest::{guest_add_route_commands, guest_config_commands, GUEST_INPUT_READY};
use crate::utils::os_commands::router::{router_add_ip_address_commands, router_login_commands, router_start_network_stack_commands, router_start_routing_stack_commands, router_stop_network_stack_commands, router_stop_routing_stack_commands};
use crate::utils::os_commands::routing::static_route::router_add_static_route_commands;
use crate::utils::route::generate_distant_network_from_test;
use crate::utils::test::test_task;

const TARGET: &str = "run";

pub async fn run(run_command: RunCommand) -> anyhow::Result<()> {
    harvest_env_variables();

    if !SHARED_DIR_PATH.exists() {
        fs::create_dir(&*SHARED_DIR_PATH)?;
    }

    if !RESULT_DIR_PATH.exists() {
        fs::create_dir(&*RESULT_DIR_PATH)?;
    }

    info!(target: TARGET, "Initializing Python interpreter");
    Python::initialize();
    info!(target: TARGET, "Python interpreter initialized!");

    let experiments = parse_experiments_files(&run_command.experiment_selection)?;

    if experiments.is_empty() {
        warn!(target: TARGET, "No experiment found");
        exit(0);
    }

    let gns3 = Python::attach(|py| Gns3Connector::new(py))?;

    let images_path = get_gns3_images_path()?;
    let network_stack_list = parse_network_stack_list_file()?;
    let routing_stack_list = parse_routing_stack_list_file()?;
    let os_list = parse_os_list_file(network_stack_list.keys().collect(), routing_stack_list.keys().collect())?;

    find_or_upload_image(&gns3, &images_path, &GUEST_IMAGE_PATH.get().unwrap())?;

    for experiment in experiments {
        if let Err(error) = run_experiment(&run_command, &gns3, &os_list, &network_stack_list, &routing_stack_list, &images_path, experiment).await {
            error!(target: TARGET, "{}", error);
            exit(1);
        }

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

pub async fn run_experiment(
    run_command: &RunCommand,
    gns3: &Gns3Connector,
    oses: &IndexMap<String, OperatingSystem>,
    network_stacks: &IndexMap<String, NetworkStack>,
    routing_stacks: &IndexMap<String, RoutingStack>,
    images_path: &PathBuf,
    mut experiment: Experiment
) -> anyhow::Result<()> {
    const TARGET: &str = "experiment";

    info!(target: TARGET, "----- Running experiment: {} -----", experiment.experiment_name);

    /* INITIALIZATION */

    find_and_delete_projects(&gns3)?;
    find_and_delete_templates(&gns3)?;
    
    for (guest_name, node) in filter_guests(&mut experiment.network.nodes) {
        generate_and_create_guest_template(&gns3, &guest_name, &node)?;
    }

    for (router_name, node) in filter_routers(&mut experiment.network.nodes) {
        let router = node.unwrap_router();
        debug!(target: TARGET, "Ensuring \"{}\" operating ({}) is uploaded", router_name, router.os_name);

        let operating_system = oses.get(&router.os_name).ok_or_else(|| anyhow!("No operating system found for {}", router.os_name))?;
        find_or_upload_image(&gns3, images_path, &operating_system.image_path)?;
    }

    /* SETUP */

    let project = create_project(&gns3, &experiment.experiment_name)?;

    for (router_name, node) in filter_routers(&mut experiment.network.nodes) {
        let router = node.unwrap_router();
        let os = oses.get(&router.os_name).ok_or_else(|| anyhow!("No operating system {} found for {}", router.os_name, router_name))?;

        generate_and_create_router_template(&gns3, &router_name, &node, os.image_name())?;

        let gns3_node = create_node(&gns3, &project.project_id, router_name, node.x, node.y)?;
        node.gns3_node = Some(gns3_node);
    }

    for (guest_name, node) in filter_guests(&mut experiment.network.nodes) {
        let gns3_node = create_node(&gns3, &project.project_id, guest_name, node.x, node.y)?;
        node.gns3_node = Some(gns3_node);
    }

    for (index, link) in experiment.network.physical_links.iter().enumerate() {
        let (node_a_name, node_a) = experiment.network.nodes.get_key_value(&link.node_a).ok_or_else(|| anyhow!("Node a \"{}\" not found in link {}", &link.node_a, index))?;
        let (node_b_name, node_b) = experiment.network.nodes.get_key_value(&link.node_b).ok_or_else(|| anyhow!("Node b \"{}\" not found in link {}", &link.node_b, index))?;

        create_link(&gns3, &project.project_id, node_a_name, node_b_name, node_a, node_b, link.adapter_a, link.adapter_b)?;
    }

    for test in &experiment.test_batch {
        let to_node = experiment.network.nodes.get(&test.to).ok_or_else(|| anyhow!("Node \"{}\" not found in test {}", &test.to, &test.name))?;
        let to_node_network = to_node.unwrap_guest().ip.network();

        let from_node = experiment.network.nodes.get(&test.from).ok_or_else(|| anyhow!("Node \"{}\" not found in test {}", test.from, &test.name))?;
        let from_node_network = from_node.unwrap_guest().ip.network();

        generate_distant_network_from_test(&test.from, experiment.network.nodes.get_mut(&test.from).unwrap(), to_node_network, from_node_network)?;
        generate_distant_network_from_test(&test.to, experiment.network.nodes.get_mut(&test.to).unwrap(), from_node_network, to_node_network)?;
    }

    /* START */

    for (guest_name, node) in filter_guests(&mut experiment.network.nodes) {
        info!(target: TARGET, "Starting guest: {}", guest_name);
        let gns3_node = node.gns3_node.as_ref().ok_or_else(|| anyhow!("No GNS3 node was attached to the guest"))?;
        gns3_node.start()?;
    }

    /* CONFIG */

    for (guest_name, node) in filter_guests(&mut experiment.network.nodes) {
        let guest = node.unwrap_guest();
        let mut config_commands = guest_config_commands(guest.ip.address());

        for distant_network in &node.distant_networks {
            let add_route_command = guest_add_route_commands(&distant_network);
            config_commands.extend(add_route_command);
        }

        config_commands.push(OsCommand::new(GUEST_INPUT_READY, ""));

        execute_commands_from_node(&guest_name, node.gns3_node.as_ref().unwrap(), config_commands)?;
    }

    for (router_name, node) in filter_routers(&mut experiment.network.nodes) {
        info!(target: TARGET, "Starting router: {}", router_name);
        let gns3_node = node.gns3_node.as_ref().ok_or_else(|| anyhow!("No GNS3 node was attached to the guest"))?;
        gns3_node.start()?;

        // Network stack setup

        let router = node.unwrap_router();
        let os = oses.get(&router.os_name).ok_or_else(|| anyhow!("No operating system {} found for {}", router.os_name, router_name))?;
        let network_stack = network_stacks.get(&os.network_stack).ok_or_else(|| anyhow!("No network stack found for {}", os.network_stack))?;

        let login_commands = router_login_commands(&os);
        let start_network_stack_commands = router_start_network_stack_commands(&os, &network_stack);
        let stop_network_stack_commands = router_stop_network_stack_commands(&os, &network_stack);
        let mut add_ip_address_commands = Vec::new();

        for (index, nic) in &router.nics {
            add_ip_address_commands.extend(router_add_ip_address_commands(&os, &network_stack, &index, nic.ip_address.address())?);
        }

        add_ip_address_commands.push(OsCommand::new(&os.input_ready, ""));

        // Routing stack setup

        let (start_routing_stack_commands, stop_routing_stack_commands) = match &os.routing_stack {
            Some(routing_stack) => {
                let routing_stack = routing_stacks.get(routing_stack.as_str()).ok_or_else(|| anyhow!("No routing stack found for {}", os.network_stack))?;

                (
                    router_start_routing_stack_commands(&os, &routing_stack),
                    router_stop_routing_stack_commands(&os, &routing_stack),
                )
            }
            None => (Vec::new(), Vec::new()),
        };

        // Routing protocols (static, RIP, OSPF, BGP, MPLS)

        let mut routing_commands = Vec::new();

        for route in &router.routes {
            let commands = match route {
                Route::Static(static_route) => router_add_static_route_commands(&os, &network_stack, &static_route)?,
                Route::Rip(_rip_route) => Vec::new(),
                Route::Ospf => Vec::new(),
                Route::Bgp => Vec::new(),
                Route::Mpls => Vec::new()
            };

            routing_commands.extend(commands);
        }

        routing_commands.push(OsCommand::new(&os.input_ready, ""));

        // Send commands

        let commands = vec![
            login_commands,
            start_network_stack_commands,
            start_routing_stack_commands,
            add_ip_address_commands,
            routing_commands,
            stop_routing_stack_commands,
            stop_network_stack_commands
        ]
            .concat();

        execute_commands_from_node(&router_name, &gns3_node, commands)?;
    }

    /* RUN */

    let mut threads = JoinSet::new();

    for test in experiment.test_batch {
        let (from_node_name, from_node) = experiment.network.nodes.get_key_value(&test.from).ok_or_else(|| anyhow!("Node \"{}\" not found in test {}", test.from, &test.name))?;
        let to_node = experiment.network.nodes.get(&test.to).ok_or_else(|| anyhow!("Node \"{}\" not found in test {}", &test.to, &test.name))?;

        if matches!(from_node.node_type, NodeType::Router(..)) || matches!(to_node.node_type, NodeType::Router(..)) {
            return Err(anyhow!("Cannot test from/to router nodes, only guests"))
        }

        let gns3_node = from_node.gns3_node.as_ref().unwrap();
        let to_node_ip = to_node.unwrap_guest().ip.address();

        threads.spawn(test_task(
            experiment.experiment_name.clone(),
            test.clone(),
            from_node_name.to_owned(),
            gns3_node.console_host(),
            gns3_node.console(),
            to_node_ip
        ));
    }

    threads.join_all().await;

    /* STOP */

    if !run_command.run_command.no_stop {
        for (node_name, node) in &experiment.network.nodes {
            debug!(target: TARGET, "Stoping node: {}", node_name);
            node.gns3_node.as_ref().unwrap().stop()?;
        }
    }

    /* END */

    info!(target: TARGET, "Experiment finished");
    info!(target: TARGET, "");

    if run_command.run_command.first_only {
        exit(0);
    }

    Ok(())
}

pub fn filter_guests(nodes: &mut IndexMap<String, Node>) -> IndexMap<&String, &mut Node> {
    nodes
        .iter_mut()
        .filter(|(_, n)| match n.node_type {
            NodeType::Guest(_) => true,
            _ => false,
        })
        .collect()
}

pub fn filter_routers(nodes: &mut IndexMap<String, Node>) -> IndexMap<&String, &mut Node> {
    nodes
        .iter_mut()
        .filter(|(_, n)| match n.node_type {
            NodeType::Router(_) => true,
            _ => false,
        })
        .collect()
}