#[derive(Parser)]
pub struct GenerateRpmFiles {
    #[clap(
        short,
        long,
        help = "Path to the rinstall.yml configuration",
        from_global
    )]
    pub config: Option<String>,
    #[clap(long)]
    pub system: bool,
    #[clap(from_global)]
    pub packages: Vec<String>,
    #[clap(flatten)]
    pub dirs: DirsConfig,
    #[clap(
        short = 'P',
        long,
        help = "Path to the directory containing the project to install",
        default_value_os_t = std::env::current_dir()
            .expect("unable to get current directory"),
    )]
    pub package_dir: std::path::PathBuf,
}
