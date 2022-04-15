#[derive(Parser)]
#[clap(
    version,
    author,
    about,
    long_about = "A helper tool that installs software and additional data into the system",
    global_setting(clap::AppSettings::DeriveDisplayOrder)
)]
pub struct Config {
    #[clap(
        short,
        long,
        help = "Path to the rinstall.yml configuration",
        global = true
    )]
    pub config: Option<String>,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    #[clap(about = "Install the packages into the system")]
    Install(InstallCmd),
    #[clap(about = "Uninstall the packages from the system")]
    Uninstall(Uninstall),
    #[clap(
        name = "rpm-files",
        about = "Generate the list of rpm files that would be installed by the package"
    )]
    GenerateRpmFiles(GenerateRpmFiles),
}
