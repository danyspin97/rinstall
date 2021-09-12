use std::{
    env,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;

use crate::Config;

pub struct Dirs {
    pub prefix: Option<PathBuf>,
    pub exec_prefix: Option<PathBuf>,
    pub bindir: PathBuf,
    pub sbindir: Option<PathBuf>,
    pub libdir: PathBuf,
    pub libexecdir: PathBuf,
    pub datarootdir: PathBuf,
    pub datadir: PathBuf,
    pub sysconfdir: PathBuf,
    pub localstatedir: PathBuf,
    pub runstatedir: PathBuf,
    pub includedir: Option<PathBuf>,
    pub docdir: Option<PathBuf>,
    pub mandir: Option<PathBuf>,
    pub pam_modulesdir: Option<PathBuf>,
    pub systemd_unitsdir: PathBuf,
}

impl Dirs {
    pub fn new(config: Config) -> Result<Dirs> {
        let mut dirs = Self {
            prefix: config.prefix.map(PathBuf::from),
            exec_prefix: config.exec_prefix.map(PathBuf::from),
            bindir: PathBuf::from(config.bindir.unwrap()),
            sbindir: config.sbindir.map(PathBuf::from),
            libdir: PathBuf::from(config.libdir.unwrap()),
            libexecdir: PathBuf::from(config.libexecdir.unwrap()),
            datarootdir: PathBuf::from(config.datarootdir.unwrap()),
            datadir: PathBuf::from(config.datadir.unwrap()),
            sysconfdir: PathBuf::from(config.sysconfdir.unwrap()),
            localstatedir: PathBuf::from(config.localstatedir.unwrap()),
            runstatedir: PathBuf::from(config.runstatedir.unwrap()),
            includedir: config.includedir.map(PathBuf::from),
            docdir: config.docdir.map(PathBuf::from),
            mandir: config.mandir.map(PathBuf::from),
            pam_modulesdir: config.pam_modulesdir.map(PathBuf::from),
            systemd_unitsdir: PathBuf::from(config.systemd_unitsdir.unwrap()),
        };

        if !config.system {
            dirs.append_home();
        }
        dirs.check_absolute_paths()?;

        Ok(dirs)
    }

    fn append_home(&mut self) {
        macro_rules! append_home_to {
            ( $($var:ident),* ) => {
                $(
                    if self.$var.is_relative() {
                        self.$var = Path::new(&env::var("HOME").unwrap()).join(&self.$var);
                    }
                )*
            };
        }

        append_home_to!(
            bindir,
            libdir,
            libexecdir,
            datarootdir,
            datadir,
            sysconfdir,
            localstatedir,
            runstatedir
        );
    }

    fn check_absolute_paths(&self) -> Result<()> {
        Ok(())
    }
}
