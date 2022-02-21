#[derive(Parser, Deserialize, Clone)]
pub struct Uninstall {
    pkg_name: String,
    #[serde(skip_deserializing)]
    #[clap(
        short = 'y',
        long = "yes",
        help = "Accept the changes and perform the uninstallation"
    )]
    accept_changes: bool,
    #[clap(short = 'f', long = "force", help = "Force the uninstallation")]
    force: bool,
}
