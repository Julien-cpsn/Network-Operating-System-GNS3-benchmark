use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use gns3fy_rs::Gns3Connector;
use indexmap::IndexMap;
use tracing::{debug, info};
use crate::models::operating_system::OperatingSystem;


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

/// Removes from the GNS3 images directory every image listed in `operating_systems.toml`.
///
/// GNS3 v2.2 has no API to delete an image, so the files are removed from the filesystem:
/// `<images_path>/QEMU/<image name>`, along with the `<image name>.md5sum` file GNS3 creates next to it.
/// Images that are not in GNS3 are skipped. Returns the number of removed images.
pub fn delete_os_images(images_path: &Path, os_list: &IndexMap<String, OperatingSystem>) -> anyhow::Result<usize> {
    // Several OSes can share the same image (ex: BSDRP-BIRD2 and BSDRP-FRR)
    let mut seen = HashSet::new();
    let image_path_list: Vec<&PathBuf> = os_list
        .values()
        .flat_map(|os| os.images_path.iter())
        .filter(|image_path| seen.insert(image_path.as_path()))
        .collect();

    delete_images(images_path, image_path_list)
}

/// Removes the given images from the GNS3 images directory (see [`delete_os_images`]).
pub fn delete_images<'a>(images_path: &Path, image_path_list: impl IntoIterator<Item = &'a PathBuf>) -> anyhow::Result<usize> {
    let qemu_images_path = images_path.join("QEMU");
    let mut deleted_images = HashSet::new();
    let mut deleted = 0;

    for image_path in image_path_list {
        // Only the file name is used, so the path inside the GNS3 directory can never escape it
        let Some(image_name) = image_path.file_name() else {
            anyhow::bail!("Invalid image path \"{}\"", image_path.display());
        };

        if !deleted_images.insert(image_name.to_owned()) {
            continue;
        }

        let mut md5sum_name: OsString = image_name.to_owned();
        md5sum_name.push(".md5sum");

        if remove_file_if_exists(&qemu_images_path.join(image_name))? {
            info!(target: TARGET, "Deleted image \"{}\" from GNS3", image_name.to_string_lossy());
            deleted += 1;
        } else {
            debug!(target: TARGET, "Image \"{}\" not found in GNS3, skipping", image_name.to_string_lossy());
        }

        // The checksum may be left over even if the image itself is gone
        remove_file_if_exists(&qemu_images_path.join(md5sum_name))?;
    }

    Ok(deleted)
}

fn remove_file_if_exists(path: &Path) -> anyhow::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Could not delete \"{}\": {e}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_images_removes_images_and_checksums_only() {
        let root = std::env::temp_dir().join(format!("nos-gns3-image-test-{}", std::process::id()));
        let qemu = root.join("QEMU");
        fs::create_dir_all(&qemu).unwrap();

        for name in ["a.qcow2", "a.qcow2.md5sum", "b.img", "keep.qcow2", "keep.qcow2.md5sum"] {
            fs::write(qemu.join(name), "x").unwrap();
        }

        let images = vec![
            PathBuf::from("/some/where/a.qcow2"),
            PathBuf::from("/other/a.qcow2"), // same name: only counted once
            PathBuf::from("/some/where/b.img"),
            PathBuf::from("/some/where/missing.iso"),
        ];

        let deleted = delete_images(&root, &images).unwrap();

        assert_eq!(deleted, 2);
        assert!(!qemu.join("a.qcow2").exists());
        assert!(!qemu.join("a.qcow2.md5sum").exists());
        assert!(!qemu.join("b.img").exists());
        assert!(qemu.join("keep.qcow2").exists());
        assert!(qemu.join("keep.qcow2.md5sum").exists());

        fs::remove_dir_all(&root).unwrap();
    }
}
