use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum G5kCommands {
    Deploy(DeployG5kCommand),
    // Todo: HERE
}

#[derive(Debug, Clone, clap::Args)]
pub struct DeployG5kCommand {
    #[clap(flatten)]
    pub login: G5kLogin,

    /// Operating system name, as listed in "experimentation/operating_systems.toml"
    pub os: String,

    /// Grid'5000 queue to submit the job to
    #[arg(long, default_value = "abaca")]
    pub queue: String,

    /// Grid'5000 cluster to reserve a node on
    #[arg(long, default_value = "grappe")]
    pub cluster: String,

    /// Reservation walltime (ex: "3" or "3:30"). Site default if absent
    #[arg(long)]
    pub walltime: Option<String>,

    /// Do not wait for the job to terminate after submitting it
    #[arg(long)]
    pub no_wait: bool,
}

#[derive(Debug, Clone, clap::Args)]
pub struct G5kLogin {
    /// Grid'5000 username
    pub user: String,

    /// Grid'5000 site (ex: nancy)
    pub site: String,
}