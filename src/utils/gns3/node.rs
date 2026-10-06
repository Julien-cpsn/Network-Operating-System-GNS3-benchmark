use std::sync::Arc;
use gns3fy_rs::{Gns3Connector, Node};
use tracing::debug;

use crate::utils::gns3::template::template_name;

const TARGET: &str = "node";

pub async fn create_node(gns3: Arc<Gns3Connector>, project_id: &str, node_name: &str, x: i32, y: i32) -> anyhow::Result<Node> {
    let mut node = Node::with_connector(gns3)
        .with_project_id(project_id)
        .with_name(node_name)
        .with_template(template_name(&node_name));

    node.x = Some(x as i64);
    node.y = Some(y as i64);

    node.create().await?;

    debug!(target: TARGET, "Generated node: {}", node_name);

    Ok(node)
}