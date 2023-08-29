#[derive(Parser)]
#[clap(
    version,
    author,
    about,
    long_about = "A helper tool that installs software and additional data into the system"
)]
pub struct Opts {
    #[clap(
        short,
        long,
        help = "Path to the rinstall.yml configuration",
        global = true
    )]
    pub config: Option<String>,
    #[clap(
        short,
        long,
        help = concat!("Do not print anything on the stdout. Warnings and",
                       " errors will still be print on the stderr")
    )]
    pub quiet: bool,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    #[clap(about = "Install the packages into the system")]
    Install(Box<InstallCmd>),
    #[clap(about = "Uninstall the packages from the system")]
    Uninstall(Uninstall),
    #[clap(about = "Create a tarball of the package")]
    Tarball(Box<TarballCmd>),
}
