use pyo3::prelude::*;

#[derive(Debug)]
pub struct Gns3Node(pub Py<PyAny>);

impl Gns3Node {
    pub fn node_id(&self) -> String {
        Python::attach(|py| {
            let gns3_node = self.0.bind(py);
            gns3_node.getattr("node_id").unwrap().extract().unwrap()
        })
    }

    pub fn console_host(&self) -> String {
        Python::attach(|py| {
            let gns3_node = self.0.bind(py);
            gns3_node.getattr("console_host").unwrap().extract().unwrap()
        })
    }

    pub fn console(&self) -> u32 {
        Python::attach(|py| {
            let gns3_node = self.0.bind(py);
            gns3_node.getattr("console").unwrap().extract().unwrap()
        })
    }

    pub fn create(&self) -> anyhow::Result<()> {
        Python::attach(|py| {
            let gns3_node = self.0.bind(py);
            gns3_node.call_method0("create")?;

            Ok(())
        })
    }

    pub fn start(&self) -> anyhow::Result<()> {
        Python::attach(|py| {
            let gns3_node = self.0.bind(py);
            gns3_node.call_method0("start")?;

            Ok(())
        })
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        Python::attach(|py| {
            let gns3_node = self.0.bind(py);
            gns3_node.call_method0("stop")?;

            Ok(())
        })
    }
}