use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use dotenv::dotenv;
use log::error;
use crate::{GNS3_PROJECT_PREFIX, GNS3_SERVER_PASSWORD, GNS3_SERVER_URL, GNS3_SERVER_USERNAME, GNS3_TEMPLATE_PREFIX, GUEST_IMAGE_PATH};

pub fn harvest_env_variables() {
    dotenv().ok();

    for (key, value) in env::vars() {
        match key.as_str() {
            "GNS3_SERVER_URL" => { GNS3_SERVER_URL.get_or_init(|| value); },
            "GNS3_SERVER_USERNAME" => { GNS3_SERVER_USERNAME.get_or_init(|| value); },
            "GNS3_SERVER_PASSWORD" => { GNS3_SERVER_PASSWORD.get_or_init(|| value); },
            "GNS3_PROJECT_PREFIX" => { GNS3_PROJECT_PREFIX.get_or_init(|| value); },
            "GNS3_TEMPLATE_PREFIX" => { GNS3_TEMPLATE_PREFIX.get_or_init(|| value); },
            "GUEST_IMAGE_PATH" => { GUEST_IMAGE_PATH.get_or_init(|| PathBuf::from_str(&value).unwrap()); },
            _ => {}
        }
    }
    
    if GNS3_SERVER_URL.get().is_none() {
        error!("Environment variable GNS3_SERVER_URL not set");
        exit(1);
    }

    if GNS3_SERVER_USERNAME.get().is_none() {
        error!("Environment variable GNS3_SERVER_USERNAME not set");
        exit(1);
    }

    if GNS3_SERVER_PASSWORD.get().is_none() {
        error!("Environment variable GNS3_SERVER_PASSWORD not set");
        exit(1);
    }

    if GNS3_PROJECT_PREFIX.get().is_none() {
        error!("Environment variable GNS3_PROJECT_PREFIX not set");
        exit(1);
    }

    if GNS3_TEMPLATE_PREFIX.get().is_none() {
        error!("Environment variable GNS3_TEMPLATE_PREFIX not set");
        exit(1);
    }

    if let Some(guest_image_path) = GUEST_IMAGE_PATH.get() {
        if guest_image_path.is_dir() {
            error!("Environment variable GUEST_IMAGE_PATH is a directory");
            exit(1);
        }
    }
    else {
        error!("Environment variable GUEST_IMAGE_PATH not set");
        exit(1);
    }
}