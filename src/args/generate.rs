use crate::args::args::ExperimentSelectionArgs;
use crate::args::run::RunCommand;
use clap::Subcommand;
use strum::Display;

#[derive(Debug, Clone, clap::Args)]
pub struct GenerateCommand {
    #[clap(subcommand)]
    pub command: GenerateSubcommand,
}

#[derive(Debug, Clone, Subcommand, Display)]
pub enum GenerateSubcommand {
    /// Run the following generated experiment files
    Run(RunCommand),
    /// Generate experiment files
    Files {
        /// Override existing experiment files
        #[arg(long, name = "override", default_value_t = false)]
        override_: bool,

        #[clap(flatten)]
        experiment_selection: ExperimentSelectionArgs,
    }
}