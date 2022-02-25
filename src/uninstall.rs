#[derive(Parser, Clone)]
pub struct Uninstall {
    pkg_name: String,
    #[clap(
        short = 'y',
        long = "yes",
        help = "Accept the changes and perform the uninstallation"
    )]
    accept_changes: bool,
    #[clap(short = 'f', long = "force", help = "Force the uninstallation")]
    force: bool,
}
