use std::{env, fs};
use std::path::PathBuf;
use std::process::exit;
use indexmap::IndexMap;
use log::{error, info};
use once_cell::sync::Lazy;
use crate::models::operating_system::OperatingSystem;
use crate::utils::files::network_stacks::NETWORK_STACK_LIST_PATH;
use crate::utils::files::routing_stacks::ROUTING_STACK_LIST_PATH;

const TARGET: &str = "files";

pub static OS_LIST_PATH: Lazy<PathBuf> = Lazy::new(|| env::current_dir().unwrap().join("experimentation").join("operating_systems.toml"));


pub fn parse_os_list_file(network_stack_list: Vec<&String>, routing_stack_list: Vec<&String>) -> anyhow::Result<IndexMap<String, OperatingSystem>> {
    if !OS_LIST_PATH.exists() {
        error!(target: TARGET, "Could not find file \"{}\"", OS_LIST_PATH.display());
        exit(1);
    }

    let os_list_content = fs::read_to_string(&*OS_LIST_PATH)?;
    let os_list: IndexMap<String, OperatingSystem> = toml::from_str(&os_list_content)?;

    if os_list.is_empty() {
        info!(target: TARGET, "Operating system list is empty");
    }
    else {
        info!(target: TARGET, "Found operating system list:");

        for (key, os) in &os_list {
            info!(target: TARGET, "- {}", key);

            if !network_stack_list.contains(&&os.network_stack) {
                error!(target: TARGET, "{} network stack \"{}\" does not exist in \"{}\"", key, os.network_stack, NETWORK_STACK_LIST_PATH.display());
                exit(1);
            }

            if let Some(routing_stack) = &os.routing_stack {
                if !routing_stack_list.contains(&routing_stack) {
                    error!(target: TARGET, "{} routing stack \"{}\" does not exist in \"{}\"", key, routing_stack, ROUTING_STACK_LIST_PATH.display());
                    exit(1);
                }
            }
        }
    }

    Ok(os_list)
}