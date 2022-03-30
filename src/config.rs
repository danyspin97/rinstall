#[derive(Parser)]
#[clap(version = "0.1.0", author = "Danilo Spinella <oss@danyspin97.org>")]
pub struct Config {
    #[clap(short, long, help = "Path to the rinstall.yml configuration")]
    pub config: Option<String>,
    #[clap(
        short = 'p',
        long = "pkgs",
        help = "List of packages to install, separated by a comma",
        global = true
    )]
    pub packages: Vec<String>,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
    #[clap(
        short = 'P',
        long,
        help = "Path to the directory containing the project to install",
        global = true
    )]
    pub package_dir: Option<String>,
    #[clap(
        short = 'y',
        long = "yes",
        help = "Accept the changes and perform the installation",
        global = true
    )]
    pub accept_changes: bool,
    #[clap(
        long = "system",
        help = "Perform a system-wide installation",
        global = true
    )]
    pub system: bool,
    #[clap(flatten)]
    pub dirs: DirsConfig,
    #[clap(flatten)]
    pub install: InstallCmd,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Install(InstallCmd),
    Uninstall(Uninstall),
    #[clap(name = "rpm-files")]
    GenerateRpmFiles(GenerateRpmFiles),
}
