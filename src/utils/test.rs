use crate::models::test::Test;
use crate::utils::files::results_dir::RESULT_DIR_PATH;
use crate::utils::files::shared_dir::SHARED_DIR_PATH;
use crate::utils::os_commands::execute::execute_commands;
use crate::utils::os_commands::guest::{guest_test_commands, GUEST_INPUT_READY};
use std::fs;
use std::net::Ipv4Addr;
use std::time::Duration;
use log::{info, warn};
use tokio::time::sleep;
use walkdir::{DirEntry, WalkDir};
use crate::models::os_command::OsCommand;

const TARGET: &str = "test";

pub async fn test_task(experiment_name: String, test: Test, from_node_name: String, console_host: String, console: u32, to_node_ip: Ipv4Addr) -> anyhow::Result<()> {
    sleep(Duration::from_secs(test.fire_at)).await;

    info!(target: TARGET, "Running test: {} (duration: {})", &test.name, &test.duration);

    let mut test_commands = guest_test_commands(&test, to_node_ip);
    test_commands.push(OsCommand::new(GUEST_INPUT_READY, ""));

    execute_commands(&from_node_name, &console_host, console, test_commands)?;

    let result_path = RESULT_DIR_PATH.join(&experiment_name).join(&test.name);

    fs::create_dir_all(&result_path)?;

    let walk_result_files = WalkDir::new(SHARED_DIR_PATH.as_path());
    let result_files: Vec<DirEntry> = walk_result_files.into_iter().filter_map(|f| f.ok()).collect();

    if result_files.is_empty() {
        warn!(target: TARGET, "No test result files found");
    }
    else {
        for file in result_files {
            if file.file_type().is_dir() {
                continue;
            }

            let output_path = result_path.join(file.file_name());

            info!(target: TARGET, "Retrieved experiment result file: {}", file.file_name().display());
            info!(target: TARGET, "Moved to : {}", output_path.display());

            fs::rename(file.path(), output_path)?;
        }
    }

    Ok(())
}