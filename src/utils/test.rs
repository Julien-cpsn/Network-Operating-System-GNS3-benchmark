use crate::models::test::Test;
use crate::utils::files::results_dir::RESULT_DIR_PATH;
use crate::utils::files::shared_dir::SHARED_DIR_PATH;
use crate::utils::os_commands::execute::execute_commands;
use std::fs;
use tracing::{error, info};
use walkdir::{DirEntry, WalkDir};
use crate::models::os_command::OsCommand;

const TARGET: &str = "test";

pub async fn test_task(
    experiment_name: String,
    from_node_name: String,
    console_host: String,
    console: u32,
    test_commands: Vec<OsCommand>,
) -> anyhow::Result<()> {
    execute_commands(
        &experiment_name,
        &from_node_name,
        &console_host,
        console,
        test_commands,
        Some(180_000),
        None
    )?;

    Ok(())
}

pub fn harvest_results(experiment_name: &str, test: &Test) -> anyhow::Result<()> {
    info!(target: TARGET, "Harvesting results...");

    let result_path = RESULT_DIR_PATH.join(&experiment_name).join(&test.name);
    fs::create_dir_all(&result_path)?;

    let walk_result_files = WalkDir::new(SHARED_DIR_PATH.as_path().join(&test.name));
    let result_files: Vec<DirEntry> = walk_result_files
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().is_file())
        .collect();

    if result_files.is_empty() {
        error!(target: TARGET, "No test result files found");
    }
    else {
        for file in result_files {
            let output_path = result_path.join(file.file_name());

            info!(target: TARGET, "Retrieved experiment result file: {}", file.file_name().display());
            info!(target: TARGET, "Moved to: {}", output_path.display());

            fs::rename(file.path(), output_path)?;
        }
    }

    Ok(())
}

pub fn clear_shared_dir() -> anyhow::Result<()> {
    info!(target: TARGET, "Clearing shared directory");

    let shared_dir = SHARED_DIR_PATH.as_path();
    for entry in fs::read_dir(shared_dir)?.filter_map(Result::ok) {
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(entry.path())?;
        }
        else {
            fs::remove_file(entry.path())?;
        }
    }

    Ok(())
}