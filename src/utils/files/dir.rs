use std::fs;
use std::path::PathBuf;

pub fn create_dir_if_does_not_exist(path: PathBuf) -> anyhow::Result<PathBuf> {
    if !path.exists() {
        fs::create_dir(&path)?;
    }

    Ok(path)
}