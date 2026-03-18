use clap::{Parser, Subcommand};
use strum::Display;
use crate::models::hardware_resources::HardwareResources;
use crate::models::nic::NicType;
use crate::models::protocol::RoutingProtocol;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command
}

#[derive(Debug, Clone, Subcommand, Display)]
pub enum Command {
    /// Generate experiment files
    Generate(GenerateCommand),
    /// Run experiment files
    Run(RunCommand)
}

#[derive(Debug, Clone, clap::Args)]
pub struct GenerateCommand {
    #[clap(flatten)]
    pub experiment_selection: ExperimentSelectionArgs,

    #[clap(subcommand)]
    pub command: GenerateSubcommand
}

#[derive(Debug, Clone, clap::Args)]
pub struct ExperimentSelectionArgs {
    /// OS to use. All if absent
    #[arg(short, long)]
    pub os: Option<String>,

    /// Test to use. All if absent
    #[arg(long)]
    pub test_batch: Option<String>,

    /// Resources to use. All if absent
    #[arg(short, long)]
    pub resources: Option<HardwareResources>,

    /// NIC to use. All if absent
    #[arg(short, long)]
    pub nic: Option<NicType>,

    /// Topology to use. All if absent
    #[arg(short, long)]
    pub topology: Option<String>,

    /// Routing protocol to use. All if absent
    #[arg(short, long)]
    pub protocol: Option<RoutingProtocol>,
}

#[derive(Debug, Clone, Subcommand, Display)]
pub enum GenerateSubcommand {
    /// Run the following generated experiment files
    Run(RunCommandArgs),
    /// Generate experiment files
    Files {
        /// Override existing experiment files
        #[arg(long, name = "override", default_value_t = true)]
        override_: bool,
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct RunCommand {
    #[clap(flatten)]
    pub experiment_selection: ExperimentSelectionArgs,

    #[clap(flatten)]
    pub run_command: RunCommandArgs,
}

#[derive(Debug, Clone, clap::Args)]
pub struct RunCommandArgs {
    /// Only run the first experiment
    #[arg(long)]
    pub first_only: bool,

    /// Do not stop GNS3 nodes after the end of the experiment
    #[arg(long)]
    pub no_stop: bool,
}