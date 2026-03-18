use log::debug;
use crate::models::gns3::connector::Gns3Connector;
use crate::models::gns3::node::Gns3Node;
use crate::utils::gns3::template::template_name;

const TARGET: &str = "node";

pub fn create_node(gns3: &Gns3Connector, project_id: &str, node_name: &str, x: i32, y: i32) -> anyhow::Result<Gns3Node> {
    let node = gns3.create_node(
        project_id,
        node_name,
        &template_name(&node_name),
        x,
        y
    )?;

    node.create()?;

    debug!(target: TARGET, "Generated node: {}", node_name);

    Ok(node)
}