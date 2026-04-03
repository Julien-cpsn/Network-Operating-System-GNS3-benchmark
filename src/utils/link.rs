use tracing::debug;
use crate::models::gns3::connector::Gns3Connector;
use crate::models::nodes::node::Node;

const TARGET: &str = "link";

pub fn create_link(gns3: &Gns3Connector, project_id: &str, node_a_name: &str, node_b_name: &str, node_a: &Node, node_b: &Node, adapter_a: u32, adapter_b: u32) -> anyhow::Result<()> {
    let link = gns3.create_link(project_id, node_a, node_b, adapter_a, adapter_b)?;
    link.create()?;

    debug!(target: TARGET, "Linked {} and {}", node_a_name, node_b_name);
    
    Ok(())
}