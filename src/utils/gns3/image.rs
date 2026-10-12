use std::path::PathBuf;
use gns3fy_rs::Gns3Connector;
use tracing::{debug};


const TARGET: &str = "image";

pub async fn find_or_upload_images(gns3: &Gns3Connector, images_path: &PathBuf, image_path_list: &Vec<PathBuf>) -> anyhow::Result<()> {
    for image_path in image_path_list {
        let image_name = image_path.file_name().unwrap().to_str().unwrap().to_string();

        if images_path.join("QEMU").join(&image_name).exists() {
            debug!(target: TARGET, "Found image: {}", image_name);
        } else {
            debug!(target: TARGET, "Image \"{}\" not found in GNS3, uploading...", image_name);
            gns3.upload_compute_image("qemu", image_path.to_str().unwrap(), "local").await?;
            debug!(target: TARGET, "Image successfully uploaded to GNS3");
        }
    }
    
    Ok(())
}