use clap::Subcommand;
use strum::Display;
use crate::args::args::ExperimentSelectionArgs;
use crate::args::run::RunCommandArgs;

#[derive(Debug, Clone, clap::Args)]
pub struct GenerateCommand {
    #[clap(flatten)]
    pub experiment_selection: ExperimentSelectionArgs,

    #[clap(subcommand)]
    pub command: GenerateSubcommand
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