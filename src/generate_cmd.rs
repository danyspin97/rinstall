#[derive(Args, Clone)]
pub struct GenerateCmd {
    #[clap(
        short = 'P',
        long,
        help = "Path to the directory containing the project to scan",
        default_value_os_t = std::env::current_dir()
            .expect("unable to get current directory"),
    )]
    pub package_dir: std::path::PathBuf,
    #[clap(
        long,
        help = "Name to use for the detected package (defaults to the directory name)"
    )]
    pub name: Option<String>,
    #[clap(
        short = 'o',
        long,
        help = "Path to write the generated install.yml to",
        default_value = "install.yml"
    )]
    pub output: std::path::PathBuf,
    #[clap(
        short = 'f',
        long,
        help = "Overwrite the output file if it already exists"
    )]
    pub force: bool,
}
