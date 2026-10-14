use std::collections::HashMap;
use std::path::PathBuf;
use tracing::warn;
use crate::args::args::ExperimentSelectionArgs;
use crate::models::experiment::Experiment;
use crate::utils::files::experiments::parse_experiments_files;
use crate::utils::files::results_dir::RESULT_DIR_PATH;

const TARGET: &str = "results";

#[derive(Debug)]
pub struct ExperimentAndResults {
    pub experiment: Experiment,
    pub experiment_keywords: Vec<String>,
    pub result_paths_per_test: HashMap<String, Vec<PathBuf>>,
}

pub fn extract_experiments_and_results(experiment_selection: &ExperimentSelectionArgs) -> anyhow::Result<Vec<ExperimentAndResults>> {
    let experiments = parse_experiments_files(&experiment_selection)?;

    let mut experiment_results: Vec<ExperimentAndResults> = Vec::new();

    for experiment in experiments {
        let mut result_paths = HashMap::new();

        for test in &experiment.test_batch {
            let result_dir_path = RESULT_DIR_PATH.join(&experiment.experiment_name).join(&test.name);

            if !result_dir_path.exists() {
                warn!(target: TARGET, "Result file for \"{}\" \"{}\" does not exist", &experiment.experiment_name, &test.name);
                continue;
            }

            for file_path in result_dir_path.read_dir()? {
                let file_path = file_path?;

                if file_path.file_type()?.is_dir() {
                    continue;
                }

                if let Some(extension) = file_path.path().to_str() && extension.ends_with(".flent.gz") {
                    let entry = result_paths.entry(test.test.clone()).or_insert(Vec::new());
                    entry.push(file_path.path())
                }
            }
        }

        if result_paths.is_empty() {
            continue;
        }

        let experiment_keywords = experiment.experiment_name.split(',').map(|k| k.to_string()).collect();

        let experiment_and_results = ExperimentAndResults {
            experiment,
            experiment_keywords,
            result_paths_per_test: result_paths,
        };

        experiment_results.push(experiment_and_results);
    }

    experiment_results.sort_by(|a, b| a.experiment.experiment_name.cmp(&b.experiment.experiment_name));

    Ok(experiment_results)
}