use std::fs;
use std::path::PathBuf;
use std::process::{Command};
use tracing::{info, warn};
use crate::args::args::ExperimentSelectionArgs;
use crate::args::plot::PlotCommand;
use crate::models::experiment::Experiment;
use crate::utils::files::experiments::parse_experiments_files;
use crate::utils::files::plot_dir::PLOT_DIR_PATH;
use crate::utils::files::results_dir::RESULT_DIR_PATH;
use crate::utils::utils::{extract_and_sort_common_parts, filter_routers};

const TARGET: &str = "plot";

const PLOTS: [(&str, &str, u32); 2] = [
    ("box_totals", "Box plot of totals", 0),
    ("icmp_cdf", "ICMP CDF", 15)
];

const EXTENSION: [&str; 2] = ["png", "svg"];

#[derive(Debug)]
struct ExperimentAndResults {
    pub experiment: Experiment,
    pub experiment_keywords: Vec<String>,
    pub result_paths: Vec<PathBuf>,
}

pub fn plot(plot_command: PlotCommand) -> anyhow::Result<()> {
    let experiment_results = extract_experiments_and_results(&plot_command.experiment_selection)?;

    let plot_legends: Vec<&Vec<String>> = experiment_results.iter().map(|e| &e.experiment_keywords).collect();

    let (common_words, non_common_words) = extract_and_sort_common_parts(plot_legends);

    let plot_output_directory_name = format!("merged_{}_{}", non_common_words.join("-"), common_words.join("-"));
    let plot_output_directory_path = PLOT_DIR_PATH.join(plot_output_directory_name);

    if !plot_output_directory_path.exists() {
        fs::create_dir(&plot_output_directory_path)?;
    }

    plot_flent(&plot_command, &plot_output_directory_path, &experiment_results, common_words)?;
    plot_resources(&plot_output_directory_path, &experiment_results)?;

    Ok(())
}

fn plot_flent(plot_command: &PlotCommand, plot_output_directory_path: &PathBuf, experiment_results: &Vec<ExperimentAndResults>, common_words: Vec<String>) -> anyhow::Result<()> {
    let notes: Vec<&str> = match plot_command.plot_command.hide_note {
        false => common_words.iter().map(|s| s.as_str()).collect(),
        true => Vec::new(),
    };

    let mut flent_input_files_args = Vec::new();
    let mut flent_additional_args = Vec::new();

    let flent_legends_to_remove: Vec<String> = common_words
        .iter()
        .map(|s| [String::from("--filter-regexp"), format!("{}\\s?", s.as_str())])
        .flatten()
        .collect();
    let flent_legends_to_remove: Vec<&str> = flent_legends_to_remove.iter().map(|s| s.as_str()).collect();

    for experiment in experiment_results {
        for result_path in &experiment.result_paths {
            flent_input_files_args.push("-i");
            flent_input_files_args.push(result_path.to_str().unwrap());
        }
    }

    if plot_command.plot_command.log_scale {
        flent_additional_args.push("--log-scale-y");
        flent_additional_args.push("log10");
    }

    if plot_command.plot_command.no_title {
        flent_additional_args.push("--no-title");
    }

    for (plot_type, plot, adjustment) in PLOTS {
        let flent_notes = adjust_note(&notes, adjustment);

        for extension in EXTENSION {
            let plot_path = plot_output_directory_path.join(format!("{plot}.{extension}"));

            let flent_args = [
                vec![
                    "-o", plot_path.to_str().unwrap(),
                    "-p", plot_type,
                    "--skip-missing-series",
                    "--filter-regexp", ",",
                    "--filter-regexp", "_",
                    "--filter-regexp", "Ping \\(ms\\) --",
                    "--filter-regexp", "ICMP - ",
                    "--filter-regexp", "(?:[0-9]{1,3}\\.){3}[0-9]{1,3}",
                    "--no-annotation",
                    "--no-markers",
                    "--no-hover-highlight",
                    "--fallback-layout"
                ],
                flent_input_files_args.clone(),
                flent_legends_to_remove.clone(),
                flent_additional_args.clone(),
                vec![
                    "--figure-note",
                    &flent_notes
                ]
            ]
                .concat();


            let mut command = Command::new("flent");
            let command = command.args(flent_args);
            command.spawn()?;

            info!(target: TARGET, "Plotted {}", plot_path.display());
        }
    }

    Ok(())
}

fn plot_resources(plot_output_directory_path: &PathBuf, experiment_results: &Vec<ExperimentAndResults>) -> anyhow::Result<()> {
    let comparison_mode = experiment_results.len() > 1;

    let output_path = plot_output_directory_path.join("resources.svg");
    let mut args = vec![
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/utils/resource_plot.py").to_string(),
        output_path.to_str().unwrap().to_string()
    ];

    for experiment_result in experiment_results {
        for (router_name, node) in filter_routers(&experiment_result.experiment.network.nodes) {
            let router_log_file_path = RESULT_DIR_PATH.join(&experiment_result.experiment.experiment_name).join(format!("{router_name}.log"));

            if !router_log_file_path.exists() {
                continue;
            }

            if comparison_mode {
                let router = node.unwrap_router();
                args.push(format!("{}:{}", router.os_name, router_log_file_path.display()));
            }
            else {
                args.push(router_log_file_path.display().to_string());
            }
        }
    }

    Command::new("python").args(&args).spawn()?;

    Ok(())
}

fn extract_experiments_and_results(experiment_selection: &ExperimentSelectionArgs) -> anyhow::Result<Vec<ExperimentAndResults>> {
    let experiments = parse_experiments_files(&experiment_selection)?;

    let mut experiment_results: Vec<ExperimentAndResults> = Vec::new();

    for experiment in experiments {
        let mut result_paths = Vec::new();

        for test in &experiment.test_batch {
            let result_dir_path = RESULT_DIR_PATH.join(&experiment.experiment_name).join(&test.name);

            if !result_dir_path.exists() {
                warn!(target: TARGET, "Result file for \"{} {}\" does not exist", &experiment.experiment_name, &test.name);
                continue;
            }

            for file_path in result_dir_path.read_dir()? {
                let file_path = file_path?;

                if file_path.file_type()?.is_dir() {
                    continue;
                }

                if let Some(extension) = file_path.path().to_str() && extension.ends_with(".flent.gz") {
                    result_paths.push(file_path.path());
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
            result_paths,
        };

        experiment_results.push(experiment_and_results);
    }

    Ok(experiment_results)
}

fn adjust_note(notes: &Vec<&str>, adjustment: u32) -> String {
    let mut lines = Vec::new();

    for (index, note) in notes.iter().enumerate() {
        let mut spaces_count = 196 - adjustment as usize - lines.len() - note.len() + (count_capital_letters(&note) / 2);

        if index == 0 {
            spaces_count -= 1;
        }

        lines.push(format!("{}{}\n", " ".repeat(spaces_count), note));
    }

    lines.join("")
}

fn count_capital_letters(string: &str) -> usize {
    let mut count = 0;

    for c in string.chars() {
        if c.is_uppercase() {
            count += 1;
        }
    }

    count
}