use serde::Deserialize;

#[derive(Parser, Deserialize, Clone)]
pub struct DirsConfig {
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
}
