use std::fs;
use std::fs::OpenOptions;
use crate::utils::files::results_dir::RESULT_DIR_PATH;
use crate::ARGS;
use std::io::stdout;
use indexmap::IndexMap;
use tracing::{subscriber, Dispatch, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use crate::models::nodes::node::Node;

pub fn setup_global_logger() -> anyhow::Result<()> {
    let verbosity = match (ARGS.verbosity.is_present(), ARGS.verbosity.tracing_level()) {
        (true, Some(level)) => level,
        _ => Level::DEBUG
    };

    let subscriber = Registry::default()
        .with(
            Layer::new()
                .with_writer(stdout.with_max_level(verbosity))
        );

    subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub fn setup_experiment_logger(experiment_name: &str, name: &str) -> anyhow::Result<(Dispatch, WorkerGuard)> {
    let verbosity = match (ARGS.verbosity.is_present(), ARGS.verbosity.tracing_level()) {
        (true, Some(level)) => level,
        _ => Level::DEBUG
    };

    let log_dir_path = RESULT_DIR_PATH.join(&experiment_name).join(format!("{name}.log"));
    let log_dir_file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(&log_dir_path)?;

    let (log_file_writer, file_guard) = tracing_appender::non_blocking(log_dir_file);
    let log_file_writer = log_file_writer.with_max_level(Level::TRACE);

    let subscriber = Registry::default()
        .with(Layer::new()
            .with_writer(stdout.with_max_level(verbosity))
        )
        .with(
            Layer::new()
                .with_writer(log_file_writer)
                .with_ansi(false)
        );

    Ok((Dispatch::new(subscriber), file_guard))
}

pub fn find_and_delete_log_files(experiment_name: &str, nodes: &IndexMap<String, Node>) -> anyhow::Result<()> {
    let experiment_path = RESULT_DIR_PATH.join(&experiment_name);
    let experiment_log_path = experiment_path.join(format!("{experiment_name}.log"));

    if experiment_log_path.exists() {
        fs::remove_file(&experiment_log_path)?;
    }

    for node_name in nodes.keys() {
        let path = experiment_path.join(format!("{node_name}.log"));

        if path.exists() {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}
