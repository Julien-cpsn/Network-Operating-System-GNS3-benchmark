use std::path::{Path};
use std::process::Command;
use anyhow::{bail, Context};
use tracing::{debug, info};
use crate::args::compute_results::ComputeResultsCommand;
use crate::utils::flent::merge::{write_merged_results, MergeStat};
use crate::utils::flent::summary::{write_runs_summary, PlanningConfig};
use crate::utils::results::{extract_experiments_and_results, ExperimentAndResults};

const TARGET: &str = "results";


pub fn compute_results(compute_results_command: ComputeResultsCommand) -> anyhow::Result<()> {
    let experiment_results = extract_experiments_and_results(&compute_results_command.experiment_selection, false)?;

    for experiment in experiment_results {
        info!(target: TARGET, "{}", &experiment.experiment.experiment_name);

        add_flent_stats(&experiment)?;
        add_runs_summaries(&experiment)?;
        add_merged_results(&experiment)?;
    }

    Ok(())
}

fn add_flent_stats(experiment: &ExperimentAndResults) -> anyhow::Result<()> {
    for (_, result_path_per_run) in &experiment.result_paths_per_run_per_test {
        for (test, result_paths_per_test) in result_path_per_run {
            for result_path in result_paths_per_test {
                let stats_path = result_path.with_extension("stats");

                if !stats_path.exists() {
                    export_with_flent("stats", result_path, &stats_path)?;
                }

                let stats_csv_path = result_path.with_extension("csv");

                if !stats_csv_path.exists() {
                    export_with_flent("csv", result_path, &stats_csv_path)?;
                }
            }
            debug!(target: TARGET, "\tAdded Flent stats for results of {}", test);
        }
    }

    Ok(())
}

fn export_with_flent(format: &str, input: &Path, output: &Path) -> anyhow::Result<()> {
    let status = Command::new("flent")
        .args(["-f", format, "-i"])
        .arg(input)
        .arg("-o")
        .arg(output)
        .status()
        .context("cannot run flent")?;

    if !status.success() {
        bail!("flent -f {format} failed for {}", input.display());
    }

    Ok(())
}
fn add_runs_summaries(experiment: &ExperimentAndResults) -> anyhow::Result<()> {
    let config = PlanningConfig::default();

    write_runs_summary(&experiment.result_paths_per_run_per_test, &config)?;

    Ok(())
}

fn add_merged_results(experiment: &ExperimentAndResults) -> anyhow::Result<()> {
    write_merged_results(&experiment.result_paths_per_run_per_test, MergeStat::default())?;

    Ok(())
}