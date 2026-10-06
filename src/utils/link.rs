use std::sync::Arc;
use std::time::Duration;
use gns3fy_rs::{Gns3Connector, LinkEndpoint};
use tokio::time::sleep;
use tracing::{debug, info};
use crate::models::link::Link;
use crate::models::nodes::node::Node;

const TARGET: &str = "link";

pub async fn create_link(gns3: Arc<Gns3Connector>, project_id: &str, node_a_name: &str, node_b_name: &str, node_a: &Node, node_b: &Node, adapter_a: u32, adapter_b: u32) -> anyhow::Result<gns3fy_rs::Link> {
    let mut link = gns3fy_rs::Link::with_connector(gns3).with_project_id(project_id);

    let endpoint_a = LinkEndpoint {
        node_id: node_a.gns3_node.as_ref().unwrap().node_id.as_ref().unwrap().to_string(),
        adapter_number: adapter_a as i64,
        port_number: 0,
        label: None,
    };

    let endpoint_b = LinkEndpoint {
        node_id: node_b.gns3_node.as_ref().unwrap().node_id.as_ref().unwrap().to_string(),
        adapter_number: adapter_b as i64,
        port_number: 0,
        label: None,
    };

    link.nodes = Some(vec![endpoint_a, endpoint_b]);

    link.create().await?;

    debug!(target: TARGET, "Linked {} and {}", node_a_name, node_b_name);
    
    Ok(link)
}

pub async fn link_failure(link: Link, mut gns3_link: gns3fy_rs::Link, happens_at: u64) -> anyhow::Result<()> {
    info!(
        target: TARGET,
        "Waiting {} seconds before programmed link failure between {} (adapter {}) and {} (adapter {})",
        happens_at,
        link.node_a,
        link.adapter_a,
        link.node_b,
        link.adapter_b
    );

    sleep(Duration::from_secs(happens_at)).await;

    info!(
        target: TARGET,
        "Waited {} seconds, programmed link failure between {} (adapter {}) and {} (adapter {})",
        happens_at,
        link.node_a,
        link.adapter_a,
        link.node_b,
        link.adapter_b
    );

    gns3_link.delete().await?;

    Ok(())
}