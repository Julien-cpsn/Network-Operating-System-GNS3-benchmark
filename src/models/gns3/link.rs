use pyo3::prelude::*;

#[derive(Debug)]
pub struct Gns3Link(pub Py<PyAny>);

impl Gns3Link {
    pub fn create(&self) -> anyhow::Result<()> {
        Python::attach(|py| {
            let gns3_link = self.0.bind(py);
            gns3_link.call_method0("create")?;

            Ok(())
        })
    }
}