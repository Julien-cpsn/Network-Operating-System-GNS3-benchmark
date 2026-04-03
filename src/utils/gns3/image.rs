use std::path::PathBuf;
use tracing::{debug};
use crate::models::gns3::connector::Gns3Connector;


const TARGET: &str = "image";

pub fn find_or_upload_image(gns3: &Gns3Connector, images_path: &PathBuf, image_path: &PathBuf) -> anyhow::Result<()> {
    let image_name = image_path.file_name().unwrap().to_str().unwrap().to_string();

    if images_path.join("QEMU").join(&image_name).exists() {
        debug!(target: TARGET, "Found image: {}", image_name);
    }
    else {
        debug!(target: TARGET, "Image \"{}\" not found in GNS3, uploading...", image_name);
        gns3.upload_compute_image("qemu", image_path.to_str().unwrap())?;
        debug!(target: TARGET, "Image successfully uploaded to GNS3");
    }
    
    Ok(())
}