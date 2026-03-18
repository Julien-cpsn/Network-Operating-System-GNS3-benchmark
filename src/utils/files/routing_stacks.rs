use std::{env, fs};
use std::path::{PathBuf};
use std::process::exit;
use indexmap::IndexMap;
use log::{error, info};
use once_cell::sync::Lazy;
use crate::models::routing_stack::RoutingStack;

const TARGET: &str = "files";

pub static ROUTING_STACK_LIST_PATH: Lazy<PathBuf> = Lazy::new(|| env::current_dir().unwrap().join("experimentation").join("routing_stacks.toml"));


pub fn parse_routing_stack_list_file() -> anyhow::Result<IndexMap<String, RoutingStack>> {
    if !ROUTING_STACK_LIST_PATH.exists() {
        error!(target: TARGET, "Could not find file \"{}\"", ROUTING_STACK_LIST_PATH.display());
        exit(1);
    }

    let routing_stack_list_content = fs::read_to_string(&*ROUTING_STACK_LIST_PATH)?;
    let routing_stacks: IndexMap<String, RoutingStack> = toml::from_str(&routing_stack_list_content)?;

    if routing_stacks.is_empty() {
        info!(target: TARGET, "Routing stack list is empty");
    }
    else {
        info!(target: TARGET, "Found routing stack list:");

        for key in routing_stacks.keys() {
            info!(target: TARGET, "- {}", key);
        }
    }

    Ok(routing_stacks)
}