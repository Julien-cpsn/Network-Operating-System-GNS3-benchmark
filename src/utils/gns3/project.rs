use std::sync::Arc;
use gns3fy_rs::{Gns3Connector, Project};
use tracing::debug;
use crate::GNS3_PROJECT_PREFIX;

const TARGET: &str = "project";

pub async fn find_and_delete_projects(gns3: Arc<Gns3Connector>) -> anyhow::Result<()> {
    let projects = gns3.get_projects().await?;

    let mut project_deleted = false;

    for project in projects {
        if let Some(name) = project.name && name.starts_with(GNS3_PROJECT_PREFIX.get().unwrap()) {
            debug!(target: TARGET, "Deleting old project: {}", name);
            gns3.delete_project(project.project_id.unwrap().as_str()).await?;

            project_deleted = true;
        }
    }

    if !project_deleted {
        debug!(target: TARGET, "No projects deleted");
    }

    Ok(())
}

pub async fn create_project(gns3: Arc<Gns3Connector>, experiment_name: &str) -> anyhow::Result<Project> {
    let project_name = format!("{}.{}", GNS3_PROJECT_PREFIX.get().unwrap(), experiment_name);
    let mut project = Project::with_connector(gns3)
        .with_name(&project_name);

    project.create().await?;

    debug!(target: TARGET, "Created project: {}", project_name);
    
    Ok(project)
}