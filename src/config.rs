#[derive(Parser)]
#[clap(version = "0.1.0", author = "Danilo Spinella <oss@danyspin97.org>")]
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
    #[clap(skip)]
    pub system: bool,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Install(InstallCmd),
    Uninstall(Uninstall),
    #[clap(name = "rpm-files")]
    GenerateRpmFiles(GenerateRpmFiles),
}
