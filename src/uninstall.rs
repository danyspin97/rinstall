#[derive(Parser, Clone)]
pub struct Uninstall {
    #[clap(
        short,
        long,
        help = "Path to the rinstall.yml configuration",
        global = true
    )]
    pub config: Option<String>,
    #[clap(
        short = 'y',
        long = "yes",
        help = "Accept the changes and perform the uninstallation"
    )]
    accept_changes: bool,
    #[clap(short = 'f', long = "force", help = "Force the uninstallation")]
    force: bool,
    #[clap(
        long = "system",
        help = "Perform a system-wide uninstallation",
        global = true
    )]
    pub system: bool,
    #[clap(
        long,
        env,
        requires = "system",
        global = true,
        help = concat!("A prefix used in constructing the default values of the directories",
                       " listed below. (system only)",
                       " [default: /usr/local]")
    )]
    pub prefix: Option<String>,
    #[clap(
        long,
        env,
        global = true, 
        help = concat!("A prefix used in constructing the default values of the directories",
                       " listed below. (system only)",
                       " [default: /usr/local]")
    )]
    pub localstatedir: Option<String>,
    #[clap(
        help = "The names or pkginfo files of the packages to remove",
        required = true,
    )]
    packages: Vec<String>,
}
