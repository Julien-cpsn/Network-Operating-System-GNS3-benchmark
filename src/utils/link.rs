use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};
use crate::models::gns3::connector::Gns3Connector;
use crate::models::gns3::link::Gns3Link;
use crate::models::link::Link;
use crate::models::nodes::node::Node;

const TARGET: &str = "link";

pub fn create_link(
    gns3: &Gns3Connector,
    project_id: &str,
    node_a_name: &str,
    node_b_name: &str,
    node_a: &Node,
    node_b: &Node,
    adapter_a: u32,
    adapter_b: u32
) -> anyhow::Result<Gns3Link> {
    let link = gns3.create_link(project_id, node_a, node_b, adapter_a, adapter_b)?;
    link.create()?;

    debug!(target: TARGET, "Linked {} and {}", node_a_name, node_b_name);
    
    Ok(link)
}

pub async fn link_failure(link: Link, gns3_link: Gns3Link, happens_at: u64) -> anyhow::Result<()> {
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

    gns3_link.delete()?;

    Ok(())
}