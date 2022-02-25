#[derive(Parser, Clone)]
pub struct InstallCmd {
    #[clap(from_global)]
    pub system: bool,
    #[clap(from_global)]
    pub accept_changes: bool,
    #[clap(
        short = 'f',
        long = "force",
        help = "Force the installation by overwriting (non-config) files",
        conflicts_with = "destdir"
    )]
    pub force: bool,
    #[clap(
        long = "update-config",
        help = "Overwrite the existing configurations of the package",
        conflicts_with = "destdir"
    )]
    pub update_config: bool,
    #[clap(
        long,
        help = "Use the generated binaries and libraries from the debug profile (only effective for rust projects)"
    )]
    pub rust_debug_target: bool,
    #[clap(short = 'D', long, requires = "system")]
    pub destdir: Option<String>,
    #[clap(long = "disable-uninstall")]
    pub disable_uninstall: bool,
    #[clap(from_global)]
    pub package_dir: Option<String>,
    #[clap(from_global)]
    pub packages: Vec<String>,
}
