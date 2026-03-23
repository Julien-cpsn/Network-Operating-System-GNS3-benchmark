use std::ffi::OsStr;
use std::{env, fs};
use std::path::PathBuf;
use std::process::exit;
use log::{error, info};
use once_cell::sync::Lazy;
use walkdir::WalkDir;
use crate::args::args::ExperimentSelectionArgs;
use crate::models::experiment::Experiment;

const TARGET: &str = "files";

pub static EXPERIMENTS_PATH: Lazy<PathBuf> = Lazy::new(|| env::current_dir().unwrap().join("experimentation").join("experiments"));

pub fn parse_experiments_files(experiment_selection: &ExperimentSelectionArgs) -> anyhow::Result<Vec<Experiment>> {
    if !EXPERIMENTS_PATH.exists() {
        error!(target: TARGET, "Could not find experiment dir \"{}\", make sure you have created or generated experiments first by using the `generate` command", EXPERIMENTS_PATH.display());
        exit(1);
    }

    let mut experiments = Vec::new();

    for file in WalkDir::new(&*EXPERIMENTS_PATH).sort_by_file_name() {
        let file = file?;

        if file.file_type().is_dir() || file.path().extension() != Some(OsStr::new("json")) {
            continue;
        }

        let experiment_content = fs::read_to_string(&file.path())?;
        let experiment: Experiment = serde_json::from_str(&experiment_content)?;

        // Exclude not selected experiment
        
        if let Some(topology_name) = &experiment_selection.topology && !experiment.experiment_name.contains(topology_name){
            continue;
        }

        if let Some(os) = &experiment_selection.os && !experiment.experiment_name.contains(os){
            continue;
        }

        if let Some(test_batch) = &experiment_selection.test_batch && !experiment.experiment_name.contains(test_batch){
            continue;
        }

        if let Some(resources) = &experiment_selection.resources && !experiment.experiment_name.contains(resources.to_string().as_str()){
            continue;
        }

        if let Some(nic) = &experiment_selection.nic && !experiment.experiment_name.contains(nic.to_string().as_str()){
            continue;
        }

        if let Some(protocol) = &experiment_selection.protocol && !experiment.experiment_name.contains(protocol.to_string().as_str()){
            continue;
        }
        
        experiments.push(experiment);
    }

    if experiments.is_empty() {
        info!(target: TARGET, "Experiment list is empty");
    }
    else {
        info!(target: TARGET, "Found experiment list:");

        for key in &experiments {
            info!(target: TARGET, "- {}", key.experiment_name);
        }
    }

    Ok(experiments)
}