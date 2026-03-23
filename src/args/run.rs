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
    /// Only run the first experiment
    #[arg(long)]
    pub first_only: bool,

    /// Do not stop GNS3 nodes after the end of the experiment
    #[arg(long)]
    pub no_stop: bool,
}