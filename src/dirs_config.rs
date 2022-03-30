#[derive(Args, Deserialize, Clone)]
pub struct DirsConfig {
    #[clap(long, requires = "system", global = true)]
    pub prefix: Option<String>,
    #[clap(long, requires = "system", global = true)]
    pub exec_prefix: Option<String>,
    #[clap(long, global = true)]
    pub bindir: Option<String>,
    #[clap(long, requires = "system", global = true)]
    pub sbindir: Option<String>,
    #[clap(long, global = true)]
    pub libdir: Option<String>,
    #[clap(long, requires = "system", global = true)]
    pub libexecdir: Option<String>,
    #[clap(long, global = true)]
    pub datarootdir: Option<String>,
    #[clap(long, global = true)]
    pub datadir: Option<String>,
    #[clap(long, global = true)]
    pub sysconfdir: Option<String>,
    #[clap(long, global = true)]
    pub localstatedir: Option<String>,
    #[clap(long, global = true)]
    pub runstatedir: Option<String>,
    #[clap(long, requires = "system", global = true)]
    pub includedir: Option<String>,
    #[clap(long, requires = "system", global = true)]
    pub docdir: Option<String>,
    #[clap(long, requires = "system", global = true)]
    pub mandir: Option<String>,
    #[clap(long, requires = "system", global = true)]
    pub pam_modulesdir: Option<String>,
    #[clap(long, global = true)]
    pub systemd_unitsdir: Option<String>,
}
