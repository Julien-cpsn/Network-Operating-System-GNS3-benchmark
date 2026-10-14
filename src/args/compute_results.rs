use crate::args::args::ExperimentSelectionArgs;

#[derive(Debug, Clone, clap::Args)]
pub struct ComputeResultsCommand {
    #[clap(flatten)]
    pub experiment_selection: ExperimentSelectionArgs,
}