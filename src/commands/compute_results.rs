use std::process::Command;
use tracing::info;
use crate::args::compute_results::ComputeResultsCommand;
use crate::utils::results::{extract_experiments_and_results, ExperimentAndResults};

const TARGET: &str = "results";

pub fn compute_results(compute_results_command: ComputeResultsCommand) -> anyhow::Result<()> {
    let experiment_results = extract_experiments_and_results(&compute_results_command.experiment_selection)?;

    add_flent_stats(&experiment_results)?;

    Ok(())
}

fn add_flent_stats(experiment_results: &Vec<ExperimentAndResults>) -> anyhow::Result<()> {
    for experiment in experiment_results {
        for (_, result_paths) in &experiment.result_paths_per_test {
            for result_path in result_paths {
                let stats_path = result_path.with_extension("stats");

                if !stats_path.exists() {
                    Command::new("flent")
                        .arg("-f")
                        .arg("stats")
                        .arg("-i")
                        .arg(result_path)
                        .arg("-o")
                        .arg(&stats_path)
                        .spawn()?;
                }

                let stats_csv_path = result_path.with_extension("csv");

                if !stats_csv_path.exists() {
                    Command::new("flent")
                        .arg("-f")
                        .arg("csv")
                        .arg("-i")
                        .arg(result_path)
                        .arg("-o")
                        .arg(&stats_csv_path)
                        .spawn()?;
                }

                info!(target: TARGET, "Added Flent stats for result {}", result_path.file_name().unwrap().to_str().unwrap());
            }
        }
    }

    Ok(())
}
