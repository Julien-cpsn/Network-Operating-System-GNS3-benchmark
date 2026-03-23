use clap::{Parser, Subcommand};
use strum::Display;
use crate::args::generate::GenerateCommand;
use crate::args::plot::PlotCommand;
use crate::args::run::RunCommand;
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
    Run(RunCommand),
    /// Plot experiment results
    Plot(PlotCommand)
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