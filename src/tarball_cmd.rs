#[derive(Args, Clone)]
pub struct TarballCmd {
    #[clap(
        long,
        help = concat!("Use the generated binaries and libraries from the",
                       " debug profile (only effective for rust projects)")
    )]
    pub rust_debug_target: bool,
    #[clap(
        long,
        help = concat!("Use the generated binaries and libraries from this",
                       " target triple (only effective for rust projects)")
    )]
    pub rust_target_triple: Option<String>,
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
        long,
        help = "Name of the created tarball (default to the basename of package-dir and the version)"
    )]
    pub tarball_name: Option<String>,
}
