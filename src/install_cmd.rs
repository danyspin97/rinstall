#[derive(Args, Clone)]
pub struct InstallCmd {
    #[clap(
        short,
        long,
        help = "Path to the rinstall.yml configuration",
        from_global
    )]
    pub config: Option<String>,
    #[clap(
        long = "system",
        help = "Perform a system-wide installation",
        global = true
    )]
    pub system: bool,
    #[clap(
        short = 'y',
        long = "yes",
        help = "Accept the changes and perform the installation"
    )]
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
    #[clap(
        short = 'D',
        long,
        requires = "system",
        help = "Install all the files relative to this directory",
        env
    )]
    pub destdir: Option<String>,
    #[clap(
        long = "skip-pkginfo",
        help = "Skip the installation of rinstall pkginfo, used for uninstallation"
    )]
    pub skip_pkg_info: bool,
    #[clap(
        short = 'P',
        long,
        help = "Path to the directory containing the project to install",
        default_value_os_t = std::env::current_dir()
            .expect("unable to get current directory"),
    )]
    pub package_dir: std::path::PathBuf,
    #[clap(
        short = 'p',
        long = "pkgs",
        help = "List of packages to install, separated by a comma"
    )]
    pub packages: Vec<String>,
    #[clap(
        short = 'U',
        long = "update",
        help = "Update the current installed package"
    )]
    pub update: bool,
    #[clap(flatten, next_help_heading = "DIRECTORIES")]
    pub dirs: DirsConfig,
}
