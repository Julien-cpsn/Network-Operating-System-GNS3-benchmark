use std::path::PathBuf;
use std::process::exit;
use expanduser::expanduser;
use ini::Ini;
use log::{debug, error};


const TARGET: &str = "GNS3 config";

pub fn find_gns3_config() -> (PathBuf, Ini) {
    debug!(target: TARGET, "Finding GNS3 Configuration file");
    let mut config = None;
    let mut config_path = None;

    let config_paths = if cfg!(unix) {
        vec![
            "~/.config/GNS3/2.2/gns3_server.conf",
            "~/.config/GNS3.conf",
            "/etc/xdg/GNS3/gns3_server.conf",
            "/etc/xdg/GNS3.conf",
        ]
    }
    else if cfg!(windows) {
        vec![
            "%APPDATA%/GNS3/gns3_server.ini",
            "%APPDATA%/Roaming/GNS3/gns3_server.ini",
            "%APPDATA%/GNS3.ini",
            "%COMMON_APPDATA%/GNS3/gns3_server.ini",
            "%COMMON_APPDATA%/GNS3.ini"
        ]
    }
    else {
        vec![
            "~/.config/GNS3/gns3_server.conf"
        ]
    };

    for path in config_paths {
        let path = expanduser(path).unwrap();
        if !path.exists() {
            continue;
        }

        let config_content = Ini::load_from_file(&path).expect("Failed to read config file");
        config = Some(config_content);
        config_path = Some(path);
    }

    if let Some(path) = config_path {
        (path, config.unwrap())
    }
    else {
        error!(target: TARGET, "Could not find GNS3 config file");
        exit(1);
    }
}

pub fn get_gns3_images_path() -> anyhow::Result<PathBuf> {
    debug!(target: TARGET, "Getting gns3 images path...");
    let (config_path, mut config) = find_gns3_config();

    let qemu = config.section_mut(Some("Qemu")).expect("\"Qemu\" key not found in GNS3 config");
    let qemu_unsafe_option = qemu.get("allow_unsafe_options").expect("\"allow_unsafe_options\" key not found in \"Qemu\" section of GNS3 config");

    if qemu_unsafe_option == "True" {
        debug!(target: TARGET, "Qemu allow_unsafe_options is already set to True");
    }
    else {
        debug!(target: TARGET, "Qemu allow_unsafe_options is not set to True");
        qemu.insert("allow_unsafe_options", "True");
        config.write_to_file(&config_path)?;
        debug!(target: TARGET, "Qemu allow_unsafe_options has been set to True");
    }

    let server = config.section(Some("Server")).expect("\"Server\" key not found in GNS3 config");
    let images_path = server.get("images_path").expect("\"images_path\" key not found \"Server\" section of GNS3 config");

    let images_path = PathBuf::from(images_path);

    if !images_path.exists() {
        error!(target: TARGET, "GNS3 images path \"{}\" does not exist", images_path.display());
        exit(1);
    }
    else {
        Ok(images_path)
    }
}