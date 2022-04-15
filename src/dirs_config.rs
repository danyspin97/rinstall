#[derive(Args, Deserialize, Clone)]
pub struct DirsConfig {
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("A prefix used in constructing the default values of the directories",
                       " listed below. (system only)",
                       " [default: /usr/local]")
    )]
    pub prefix: Option<String>,
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("A prefix used in constructing the default values of some of the",
                       " variables listed below. (system only)",
                       " [default: @prefix@])]")
    )]
    pub exec_prefix: Option<String>,
    #[clap(
        long,
        env,
        help = concat!("The directory for installing executable programs that users can run.",
                       " [system default: @exec_prefix@/bin] [user default: $HOME/.local/bin]")
    )]
    pub bindir: Option<String>,
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("The directory for installing executable programs that can be run from the",
                       " shell, but are only generally useful to system administrators.",
                       " (system only)",
                       " [default: @exec_prefix@/sbin]")
    )]
    pub sbindir: Option<String>,
    #[clap(
        long,
        env,
        help = concat!("The directory for object files and libraries of object code.",
                       " [system default: @exec_prefix@/lib] [user default: $HOME/.local/lib]")
    )]
    pub libdir: Option<String>,
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("The directory for installing executable programs to be run by other",
                        " programs rather than by users. (system only)",
                        " [default: @exec_prefix@/libexec]")
    )]
    pub libexecdir: Option<String>,
    #[clap(
        long,
        env,
        help = concat!("The root of the directory tree for read-only architecture-independent",
                       " data files.",
                       " [system default: @prefix@/share] [user default: @XDG_DATA_HOME@]")
    )]
    pub datarootdir: Option<String>,
    #[clap(
        long,
        env,
        help = concat!("The directory for installing idiosyncratic read-only architecture-independent",
               " data files for this program.",
               " [default: @datarootdir@]")
    )]
    pub datadir: Option<String>,
    #[clap(
        long,
        env,
        help = concat!("The directory for installing read-only data files that pertain to a single",
                       " machine–that is to say, files for configuring a host.",
                       " [system default: @prefix@/etc] [user default: @XDG_CONFIG_HOME@]")
    )]
    pub sysconfdir: Option<String>,
    #[clap(
        long,
        env,
        help = concat!("The directory for installing data files which the programs modify while they run,",
                       " and that pertain to one specific machine.",
                       " [system default: @prefix@/var] [user default: @XDG_DATA_HOME@]")
    )]
    pub localstatedir: Option<String>,
    #[clap(
        long,
        env,
        help = concat!("The directory for installing data files which the programs modify while",
                       " they run and which need not persist longer than the execution of the",
                       " program.",
                       " [system default: @localstatedir@/run] [user default: @XDG_RUNTIME_DIR@]")
    )]
    pub runstatedir: Option<String>,
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("The directory for installing header files to be included by user programs",
                       " with the C ‘#include’ preprocessor directive. (system only)",
                       " [default: @prefix@/include]")
    )]
    pub includedir: Option<String>,
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("The directory for installing documentation files (other than Info)",
                       " The package name will be appendend automatically. (system only)",
                       " [default: @datarootdir@/doc]")
    )]
    pub docdir: Option<String>,
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("The top-level directory for installing the man pages (if any)",
                       " (system only)",
                       " [default: @datarootdir@/man]")
    )]
    pub mandir: Option<String>,
    #[clap(
        long,
        requires = "system",
        env,
        help = concat!("The directory for installing the pam modules for this package. (system only)",
                       "[default: @libdir@/security]")
    )]
    pub pam_modulesdir: Option<String>,
    #[clap(
        long,
        env, 
        help = concat!("The directory for installing the systemd unit files for this package.",
                       " (system only)",
                       "[default: @libdir@/systemd]")
    )]
    pub systemd_unitsdir: Option<String>,
}
