use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;
use indexmap::IndexMap;
use tracing::{error, info, warn};
use strum::VariantArray;
use crate::args::generate::{GenerateCommand, GenerateSubcommand};
use crate::models::experiment::Experiment;
use crate::models::hardware_resources::HardwareResources;
use crate::models::network::Network;
use crate::models::nic::{Nic, NicType};
use crate::models::nodes::guest::Guest;
use crate::models::nodes::node::{GenericNodeType, Node, NodeType};
use crate::models::nodes::router::Router;
use crate::models::operating_system::OperatingSystem;
use crate::models::protocol::RoutingProtocol;
use crate::models::test::Test;
use crate::models::topology::Topology;
use crate::utils::files::dir::create_dir_if_does_not_exist;
use crate::utils::files::experiments::EXPERIMENTS_PATH;
use crate::utils::files::network_stacks::parse_network_stack_list_file;
use crate::utils::files::oses::{parse_os_list_file, OS_LIST_PATH};
use crate::utils::files::routing_stacks::parse_routing_stack_list_file;
use crate::utils::files::test_batches::{parse_test_batch_list_file, TEST_BATCH_LIST_PATH};
use crate::utils::files::topologies::{parse_topologies_list_file, TOPOLOGIES_LIST_PATH};

const TARGET: &str = "generate";


pub fn generate(generate_command: GenerateCommand) -> anyhow::Result<()> {
    let experiment_selection = match &generate_command.command {
        GenerateSubcommand::Run(run_command) => &run_command.experiment_selection,
        GenerateSubcommand::Files { experiment_selection, .. } => &experiment_selection
    };

    let network_stack_list = parse_network_stack_list_file()?;
    let routing_stack_list = parse_routing_stack_list_file()?;
    let os_list = parse_os_list_file(network_stack_list.keys().collect(), routing_stack_list.keys().collect())?;
    let oses_to_use = get_oses_to_use(&experiment_selection.os, os_list);

    let test_batch_list = parse_test_batch_list_file()?;
    let test_batches_to_use = get_test_batches_to_use(&experiment_selection.test_batch, test_batch_list);

    let resources_to_use = match &experiment_selection.resources {
        None => HardwareResources::VARIANTS.to_vec(),
        Some(resources) => vec![resources.to_owned()],
    };

    let nic_types_to_use = match &experiment_selection.nic {
        None => NicType::VARIANTS.to_vec(),
        Some(nic_type) => vec![nic_type.to_owned()],
    };

    let topology_list = parse_topologies_list_file()?;
    let topologies_to_use = get_topologies_to_use(&experiment_selection.topology, topology_list, &experiment_selection.protocol);

    if oses_to_use.is_empty() || test_batches_to_use.is_empty() || topologies_to_use.is_empty() {
        warn!(target: TARGET, "At least of the lists is empty, no experiment will be generated");
        warn!(target: TARGET, "Exiting");
        exit(0);
    }

    // Experiment count
    {
        let mut experiment_count = oses_to_use.len() * test_batches_to_use.len() * resources_to_use.len() * nic_types_to_use.len();

        for topology in topologies_to_use.values() {
            experiment_count *= topology.supported_routing_protocols.len();
        }

        info!(target: TARGET, "Total of {} individual experiment",  experiment_count);
    }

    let override_ = match &generate_command.command {
        GenerateSubcommand::Files { override_, .. } => *override_,
        _ => false
    };

    if override_ == true {
        warn!(target: TARGET, "Override is set, existing experiment will be overwritten");
    }
    else {
        warn!(target: TARGET, "Override is not set, existing experiment will NOT be overwritten");
    }

    let experiment_dir = create_dir_if_does_not_exist(EXPERIMENTS_PATH.clone())?;

    for os_name in oses_to_use.keys() {
        let os_dir = create_dir_if_does_not_exist(experiment_dir.join(os_name))?;

        for (test_batch_name, test_batch) in &test_batches_to_use {
            let test_batch_dir = create_dir_if_does_not_exist(os_dir.join(test_batch_name))?;

            for resources in &resources_to_use {
                let resources_dir = create_dir_if_does_not_exist(test_batch_dir.join(resources.to_string()))?;

                for nic in &nic_types_to_use {
                    let nic_dir = create_dir_if_does_not_exist(resources_dir.join(nic.to_string()))?;

                    for (topology_name, topology) in &topologies_to_use {
                        let topology_dir = create_dir_if_does_not_exist(nic_dir.join(topology_name))?;

                        for routing_protocol in &topology.supported_routing_protocols {
                            let routing_protocol_dir = create_dir_if_does_not_exist(topology_dir.join(routing_protocol.to_string()))?;
                            let experiment_path = routing_protocol_dir.join("experiment.json");

                            info!(target: TARGET, "{}, {}, {}, {}, {}", os_name, test_batch_name, resources, topology_name, routing_protocol);

                            if experiment_path.exists() && override_ == false {
                                continue;
                            }

                            let mut nodes = IndexMap::new();

                            for (key, generic_node) in &topology.network.nodes {
                                let (vcpu, ram, node_type) = match &generic_node.node_type {
                                    GenericNodeType::Guest(guest) => {
                                        let node_type = NodeType::Guest(Guest {
                                            ip: guest.ip,
                                        });

                                        (guest.vcpu, guest.ram, node_type)
                                    },
                                    GenericNodeType::Router(router) => {
                                        let (vcpu, ram) = resources.to_vcpu_and_ram();
                                        let routes = match router.routes.get(routing_protocol) {
                                            Some(routes) => routes.clone(),
                                            None => Vec::new(),
                                        };

                                        let mut nics = IndexMap::new();

                                        for (index, ip) in &router.ips {
                                            let router_nic = Nic {
                                                nic_type: nic.clone(),
                                                ip_address: *ip,
                                            };

                                            nics.insert(index.clone(), router_nic);
                                        }

                                        let node_type = NodeType::Router(Router {
                                            os_name: os_name.to_owned(),
                                            number_nics: router.number_nics,
                                            nics,
                                            routes,
                                        });

                                        (vcpu, ram, node_type)
                                    }
                                };

                                let node = Node {
                                    vcpu,
                                    ram,
                                    x: generic_node.x,
                                    y: generic_node.y,
                                    node_type,
                                    gns3_node: None,
                                    distant_networks: vec![],
                                };

                                nodes.insert(key.to_owned(), node);
                            }

                            let experiment = Experiment {
                                experiment_name: format!("{topology_name},{os_name},{test_batch_name},{resources},{nic},{routing_protocol}"),
                                network: Network {
                                    nodes,
                                    physical_links: topology.network.physical_links.clone()
                                },
                                test_batch: test_batch.clone(),
                            };

                            let experiment_json = serde_json::to_string_pretty(&experiment)?;

                            let mut experiment_file = OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(experiment_path)?;

                            experiment_file.write_all(experiment_json.as_bytes())?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_oses_to_use(command_os: &Option<String>, mut os_list: IndexMap<String, OperatingSystem>) -> IndexMap<String, OperatingSystem> {
    if let Some(os_name) = &command_os {
        if !os_list.contains_key(os_name) {
            error!(target: TARGET, "{} was not found in \"{}\"", os_name, OS_LIST_PATH.display());
            exit(1);
        }

        os_list.retain(|o, _| o == os_name);
    }

    os_list
}

fn get_test_batches_to_use(command_test: &Option<String>, mut test_list: IndexMap<String, Vec<Test>>) -> IndexMap<String, Vec<Test>> {
   if let Some(test) = &command_test {
       if !test_list.contains_key(test) {
           error!(target: TARGET, "{} was not found in \"{}\"", test, TEST_BATCH_LIST_PATH.display());
           exit(1);
       }

       test_list.retain(|t, _| t == test);
   }

    test_list
}

fn get_topologies_to_use(command_topology: &Option<String>, topology_list: IndexMap<String, Topology>, command_routing_protocol: &Option<RoutingProtocol>) -> IndexMap<String, Topology> {
    let mut topology_list = match &command_topology {
        None => topology_list,
        Some(topology) => match topology_list.get_key_value(topology) {
            Some(topology) => IndexMap::from([(topology.0.to_owned(), topology.1.to_owned())]),
            None => {
                error!(target: TARGET, "{} was not found in \"{}\"", topology, TOPOLOGIES_LIST_PATH.display());
                exit(1);
            }
        }
    };

    if let Some(routing_protocol) = command_routing_protocol {
        for (key, topology) in topology_list.iter_mut() {
            if !topology.supported_routing_protocols.contains(&routing_protocol) {
                error!(target: TARGET, "{} virtual topology was not found in the topology \"{}\"", routing_protocol, key);
                exit(1);
            }

            topology.supported_routing_protocols.retain(|r| r == routing_protocol);
        }
    }

    topology_list
}