use log::debug;
use crate::GNS3_PROJECT_PREFIX;
use crate::models::gns3::connector::Gns3Connector;
use crate::models::gns3::project::Gns3Project;

const TARGET: &str = "project";

pub fn find_and_delete_projects(gns3: &Gns3Connector) -> anyhow::Result<()> {
    let projects = gns3.get_projects()?;

    for project in projects {
        if project.name.starts_with(GNS3_PROJECT_PREFIX.get().unwrap()) {
            debug!(target: TARGET, "Deleting old project: {}", project.name);
            gns3.delete_project(&project.project_id)?;
        }
    }

    Ok(())
}

pub fn create_project(gns3: &Gns3Connector, experiment_name: &str) -> anyhow::Result<Gns3Project> {
    let project_name = format!("{}.{}", GNS3_PROJECT_PREFIX.get().unwrap(), experiment_name);
    let project = gns3.create_project(&project_name)?;

    debug!(target: TARGET, "Created project: {}", project_name);
    
    Ok(project)
}