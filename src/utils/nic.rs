use crate::models::nodes::node::{Node, NodeType};
use crate::models::operating_system::OperatingSystem;
use indexmap::IndexMap;

#[allow(unused)]
pub fn get_adapter_index_from_name(node: &Node, oses: &IndexMap<String, OperatingSystem>, adapter_index: u32) -> anyhow::Result<u32> {
    let adapter_index = match &node.node_type {
        NodeType::Guest(_) => adapter_index,
        NodeType::Router(router) => {
            let os = oses.get(&router.os_name).unwrap();

            adapter_index - os.interfaces_start_at
        },
    };

    Ok(adapter_index)
}