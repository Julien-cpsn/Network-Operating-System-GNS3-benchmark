use crate::args::args::ExperimentSelectionArgs;

#[derive(Debug, Clone, clap::Args)]
pub struct PlotCommand {
    #[clap(flatten)]
    pub experiment_selection: ExperimentSelectionArgs,

    #[clap(flatten)]
    pub plot_command: PlotCommandArgs,
}

#[derive(Debug, Clone, clap::Args)]
pub struct PlotCommandArgs {
    /// Plot without title
    #[clap(long)]
    pub no_title: bool,

    /// Remove common legend words
    #[clap(long, default_value = "true")]
    pub remove_common_legend: bool,

    /// Hide foot node
    #[clap(long)]
    pub hide_note: bool,

    /// Plot with log10 scale
    #[clap(long)]
    pub log_scale: bool
}