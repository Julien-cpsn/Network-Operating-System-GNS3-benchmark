pub(crate) mod models;
pub(crate) mod utils;
pub(crate) mod args;
pub(crate) mod commands;

use crate::commands::generate::generate;
use crate::commands::run::run;
use tracing::{info};
use once_cell::sync::{Lazy, OnceCell};
use std::path::PathBuf;
use clap::Parser;
use futures::{FutureExt};
use crate::args::args::{Args, Command};
use crate::args::generate::GenerateSubcommand;
use crate::commands::plot::plot;
use crate::utils::log::setup_global_logger;

pub static ARGS: Lazy<Args> = Lazy::new(|| Args::parse());

pub static GNS3_SERVER_URL: OnceCell<String> = OnceCell::new();
pub static GNS3_SERVER_USERNAME: OnceCell<String> = OnceCell::new();
pub static GNS3_SERVER_PASSWORD: OnceCell<String> = OnceCell::new();
pub static GNS3_PROJECT_PREFIX: OnceCell<String> = OnceCell::new();
pub static GNS3_TEMPLATE_PREFIX: OnceCell<String> = OnceCell::new();
pub static GUEST_IMAGE_PATH: OnceCell<PathBuf> = OnceCell::new();

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    setup_global_logger()?;

    handle_command(&ARGS.command).await?;

    Ok(())
}

async fn handle_command(command: &Command) -> anyhow::Result<()> {
    info!(target: "main", "===== {} command =====", command);

    match &command {
        Command::Generate(generate_command) => {
            generate(generate_command.clone())?;

            if let GenerateSubcommand::Run(run_command) = &generate_command.command {
                handle_command(&Command::Run(run_command.clone())).boxed_local().await?;
            }
        },
        Command::Run(run_command) => run(run_command.clone()).await?,
        Command::Plot(plot_command) => plot(plot_command.clone())?,
    }

    Ok(())
}
