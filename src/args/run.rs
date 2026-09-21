use crate::args::args::ExperimentSelectionArgs;

#[derive(Debug, Clone, clap::Args)]
pub struct RunCommand {
    #[clap(flatten)]
    pub experiment_selection: ExperimentSelectionArgs,

    #[clap(flatten)]
    pub run_command: RunCommandArgs,
}

#[derive(Debug, Clone, clap::Args)]
pub struct RunCommandArgs {
    /// No sleep after starting the nodes and before starting the experiment
    #[arg(long)]
    pub no_sleep: bool,

    /// Do not run test in the experiment
    #[arg(long)]
    pub no_test: bool,

    /// Only run the first experiment
    #[arg(long)]
    pub first_only: bool,

    /// Do not stop GNS3 nodes after the end of the experiment
    #[arg(long)]
    pub no_stop: bool,
    
    /// Does not input any command inside the router in order to set it up and test commands yourself
    #[arg(long)]
    pub os_setup: bool
}