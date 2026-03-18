mod models;
mod utils;
mod args;
mod commands;

use crate::args::{Args, Command, GenerateSubcommand, RunCommand};
use crate::commands::generate::generate;
use crate::commands::run::run;
use clap::Parser;
use log::{info, LevelFilter};
use once_cell::sync::{Lazy, OnceCell};
use std::path::PathBuf;
use futures::{FutureExt};

pub static ARGS: Lazy<Args> = Lazy::new(|| Args::parse());

pub static GNS3_SERVER_URL: OnceCell<String> = OnceCell::new();
pub static GNS3_SERVER_USERNAME: OnceCell<String> = OnceCell::new();
pub static GNS3_SERVER_PASSWORD: OnceCell<String> = OnceCell::new();
pub static GNS3_PROJECT_PREFIX: OnceCell<String> = OnceCell::new();
pub static GNS3_TEMPLATE_PREFIX: OnceCell<String> = OnceCell::new();
pub static GUEST_IMAGE_PATH: OnceCell<PathBuf> = OnceCell::new();

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    handle_command(&ARGS.command).await?;

    Ok(())
}

async fn handle_command(command: &Command) -> anyhow::Result<()> {
    info!(target: "main", "===== {} command =====", command);

    match &command {
        Command::Generate(generate_command) => {
            generate(generate_command.clone())?;

            if let GenerateSubcommand::Run(run_command_args) = &generate_command.command {
                let run_command = RunCommand {
                    experiment_selection: generate_command.experiment_selection.clone(),
                    run_command: run_command_args.clone(),
                };

                handle_command(&Command::Run(run_command)).boxed_local().await?;
            }
        },
        Command::Run(run_command) => run(run_command.clone()).await?,
    }

    Ok(())
}
