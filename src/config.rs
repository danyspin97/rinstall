use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Deserialize)]
#[clap(version = "0.1.0", author = "Danilo Spinella <oss@danyspin97.org>")]
pub struct Config {
    #[serde(skip_deserializing)]
    #[clap(short, long, help = "Path to the rinstall.yml configuration")]
    pub config: Option<String>,
    #[serde(skip_deserializing)]
    #[clap(long = "system", help = "Perform a system-wide installation")]
    pub system: bool,
    #[serde(skip_deserializing)]
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
    #[serde(skip_deserializing)]
    #[clap(
        short = 'P',
        long,
        help = "Path to the directory containing the project to install"
    )]
    pub package_dir: Option<String>,
    #[serde(skip_deserializing, default)]
    #[clap(
        short = 'p',
        long = "pkgs",
        help = "List of packages to install, separated by a comma"
    )]
    pub packages: Vec<String>,
    #[serde(skip_deserializing)]
    #[clap(long = "disable-uninstall")]
    pub disable_uninstall: bool,
    #[serde(skip_deserializing)]
    #[clap(short = 'D', long, requires = "system")]
    pub destdir: Option<String>,
    #[clap(long)]
    pub prefix: Option<String>,
    #[clap(long)]
    pub exec_prefix: Option<String>,
    #[clap(long)]
    pub bindir: Option<String>,
    #[clap(long, requires = "system")]
    pub sbindir: Option<String>,
    #[clap(long)]
    pub libdir: Option<String>,
    #[clap(long, requires = "system")]
    pub libexecdir: Option<String>,
    #[clap(long)]
    pub datarootdir: Option<String>,
    #[clap(long)]
    pub datadir: Option<String>,
    #[clap(long)]
    pub sysconfdir: Option<String>,
    #[clap(long)]
    pub localstatedir: Option<String>,
    #[clap(long)]
    pub runstatedir: Option<String>,
    #[clap(long, requires = "system")]
    pub includedir: Option<String>,
    #[clap(long, requires = "system")]
    pub docdir: Option<String>,
    #[clap(long, requires = "system")]
    pub mandir: Option<String>,
    #[clap(long, requires = "system")]
    pub pam_modulesdir: Option<String>,
    #[clap(long)]
    pub systemd_unitsdir: Option<String>,
    #[serde(skip_deserializing)]
    #[clap(
        long,
        help = "Use the generated binaries and libraries from the debug profile (only effective for rust projects)"
    )]
    pub rust_debug_target: bool,
    #[serde(skip_deserializing)]
    #[clap(subcommand)]
    pub subcmd: Option<SubCommand>,
}

#[derive(Parser, Clone)]
pub enum SubCommand {
    Uninstall(Uninstall),
    #[clap(name = "rpm-files")]
    GenerateRpmFiles,
}
