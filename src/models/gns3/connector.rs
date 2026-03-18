use anyhow::anyhow;
use log::{info};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_pyobject::{from_pyobject, pydict, to_pyobject};
use crate::{GNS3_SERVER_PASSWORD, GNS3_SERVER_URL, GNS3_SERVER_USERNAME};
use crate::models::gns3::link::Gns3Link;
use crate::models::gns3::node::Gns3Node;
use crate::models::gns3::project::Gns3Project;
use crate::models::gns3::template::Gns3Template;
use crate::models::nodes::node::Node;

const TARGET: &str = "GNS3";

pub struct Gns3Connector(pub Py<PyModule>, pub Py<PyAny>);

impl Gns3Connector {
    pub fn new(py: Python) -> PyResult<Self> {
        let gns3fy = py.import("gns3fy")?;
        let gns3_connector = gns3fy.getattr("Gns3Connector")?;
        let connector = gns3_connector.call1((
            GNS3_SERVER_URL.get().unwrap(),
            GNS3_SERVER_USERNAME.get().unwrap(),
            GNS3_SERVER_PASSWORD.get().unwrap(),
        ))?;

        info!(target: TARGET, "GNS3 connector successfully created");

        Ok(Self(gns3fy.unbind(), connector.unbind()))
    }

    pub fn get_projects(&self) -> anyhow::Result<Vec<Gns3Project>> {
        Python::attach(|py| {
            let gns3_connector = self.1.bind(py);
            let data = gns3_connector.call_method0("get_projects")?;
            from_pyobject(data).map_err(|err| anyhow!(err))
        })
    }

    pub fn delete_project(&self, project_id: &str) -> anyhow::Result<()> {
        Python::attach(|py| {
            let gns3_connector = self.1.bind(py);
            gns3_connector.call_method1("delete_project", (project_id,))?;

            Ok(())
        })
    }

    pub fn create_project(&self, project_name: &str) -> anyhow::Result<Gns3Project> {
        Python::attach(|py| {
            let kargs = pydict! { py, "name" => project_name }?;
            let gns3_connector = self.1.bind(py);
            let project = gns3_connector.call_method("create_project", (), Some(&kargs))?;
            from_pyobject(project).map_err(|err| anyhow!(err))
        })
    }

    pub fn get_templates(&self) -> anyhow::Result<Vec<Gns3Template>> {
        Python::attach(|py| {
            let gns3_connector = self.1.bind(py);
            let data = gns3_connector.call_method0("get_templates")?;
            from_pyobject(data).map_err(|err| anyhow!(err))
        })
    }

    pub fn create_template(&self,template: &Gns3Template) -> anyhow::Result<()> {
        Python::attach(|py| -> anyhow::Result<()> {
            let py_dict: Bound<PyDict> = to_pyobject(py, &template)?.cast_into().unwrap();
            let gns3_connector = self.1.bind(py);
            gns3_connector.call_method("create_template", (), Some(&py_dict))?;

            Ok(())
        })
    }

    pub fn delete_template(&self, template_name: &str) -> anyhow::Result<()> {
        Python::attach(|py| {
            let gns3_connector = self.1.bind(py);
            gns3_connector.call_method1("delete_template", (template_name,))?;

            Ok(())
        })
    }

    pub fn upload_compute_image(&self, emulator: &str, file_path: &str) -> anyhow::Result<()> {
        Python::attach(|py| {
            let gns3_connector = self.1.bind(py);
            gns3_connector.call_method1("upload_compute_image", (emulator, file_path)).ok();

            Ok(())
        })
    }

    pub fn create_node(&self, project_id: &str, node_name: &str, template_name: &str, x: i32, y: i32) -> anyhow::Result<Gns3Node> {
        let node = Python::attach(|py| -> anyhow::Result<Py<PyAny>> {
            let gns3fy = self.0.bind(py);
            let gns3_connector = self.1.bind(py);
            let node_type = gns3fy.getattr("Node")?;

            let kwargs = pydict! {
                py,
                "project_id" => project_id,
                "connector" => gns3_connector.clone(),
                "name" => node_name,
                "template" => template_name,
                "x" => x,
                "y" => y
            }?;

            Ok(node_type.call((), Some(&kwargs))?.unbind())
        })?;

        Ok(Gns3Node(node))
    }

    pub fn create_link(&self, project_id: &str, node_a: &Node, node_b: &Node, adapter_a: u32, adapter_b: u32) -> anyhow::Result<Gns3Link> {
        let link = Python::attach(|py| -> anyhow::Result<Py<PyAny>> {
            let gns3fy = self.0.bind(py);
            let gns3_connector = self.1.bind(py);
            let link_type = gns3fy.getattr("Link")?;

            let nodes = [
                pydict! { py, "node_id" => node_a.gns3_node.as_ref().unwrap().node_id(), "adapter_number" => adapter_a, "port_number" => 0 }?,
                pydict! { py, "node_id" => node_b.gns3_node.as_ref().unwrap().node_id(), "adapter_number" => adapter_b, "port_number" => 0 }?,
            ];

            let kwargs = pydict! {
                py,
                "project_id" => project_id,
                "connector" => gns3_connector.clone(),
                "nodes" => nodes
            }?;


             Ok(link_type.call((), Some(&kwargs))?.unbind())
        })?;

        Ok(Gns3Link(link))
    }
}