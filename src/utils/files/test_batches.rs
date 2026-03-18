use std::{env, fs};
use std::path::PathBuf;
use std::process::exit;
use indexmap::IndexMap;
use log::{error, info};
use once_cell::sync::Lazy;
use crate::models::test::Test;

const TARGET: &str = "files";

pub static TEST_BATCH_LIST_PATH: Lazy<PathBuf> = Lazy::new(|| env::current_dir().unwrap().join("experimentation").join("test_batches.toml"));


pub fn parse_test_batch_list_file() -> anyhow::Result<IndexMap<String, Vec<Test>>> {
    if !TEST_BATCH_LIST_PATH.exists() {
        error!(target: TARGET, "Could not find file \"{}\"", TEST_BATCH_LIST_PATH.display());
        exit(1);
    }

    let test_batch_list_content = fs::read_to_string(&*TEST_BATCH_LIST_PATH)?;
    let test_batch_list: IndexMap<String, Vec<Test>> = toml::from_str(&test_batch_list_content)?;

    if test_batch_list.is_empty() {
        info!(target: TARGET, "Test list is empty");
    }
    else {
        info!(target: TARGET, "Found test batch list:");

        for (key, test_batch) in &test_batch_list {
            info!(target: TARGET, "\t- {}", key);
            for test in test_batch {
                info!(target: TARGET, "\t- {} (test: {}, fire at: {}, duration: {})", test.name, test.test, test.fire_at, test.duration);
            }
        }
    }

    Ok(test_batch_list)
}