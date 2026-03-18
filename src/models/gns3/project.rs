use pyo3::pyclass;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Serialize, Deserialize)]
pub struct Gns3Project {
    pub auto_close: bool,
    pub auto_open: bool,
    pub auto_start: bool,
    pub drawing_grid_size: u32,
    pub filename: String,
    pub grid_size: u32,
    pub name: String,
    pub path: String,
    pub project_id: String,
    pub scene_height: u32,
    pub scene_width: u32,
    pub show_grid: bool,
    pub show_interface_labels: bool,
    pub show_layers: bool,
    pub snap_to_grid: bool,
    pub status: String,
    pub supplier: Option<String>,
    pub variables: Option<String>,
    pub zoom: u32,
}