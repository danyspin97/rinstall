use std::{
    env,
    path::{Path, PathBuf},
};

use color_eyre::Result;

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
    pub fn new(config: &Config) -> Result<Dirs> {
        let mut dirs = Self {
            prefix: config.prefix.as_ref().map(PathBuf::from),
            exec_prefix: config.exec_prefix.as_ref().map(PathBuf::from),
            bindir: PathBuf::from(config.bindir.as_ref().unwrap()),
            sbindir: config.sbindir.as_ref().map(PathBuf::from),
            libdir: PathBuf::from(config.libdir.as_ref().unwrap()),
            libexecdir: PathBuf::from(config.libexecdir.as_ref().unwrap()),
            datarootdir: PathBuf::from(config.datarootdir.as_ref().unwrap()),
            datadir: PathBuf::from(config.datadir.as_ref().unwrap()),
            sysconfdir: PathBuf::from(config.sysconfdir.as_ref().unwrap()),
            localstatedir: PathBuf::from(config.localstatedir.as_ref().unwrap()),
            runstatedir: PathBuf::from(config.runstatedir.as_ref().unwrap()),
            includedir: config.includedir.as_ref().map(PathBuf::from),
            docdir: config.docdir.as_ref().map(PathBuf::from),
            mandir: config.mandir.as_ref().map(PathBuf::from),
            pam_modulesdir: config.pam_modulesdir.as_ref().map(PathBuf::from),
            systemd_unitsdir: PathBuf::from(config.systemd_unitsdir.as_ref().unwrap()),
        };

        if !config.system {
            dirs.append_home();
        }
        dirs.check_absolute_paths()?;

        Ok(dirs)
    }

    fn append_home(&mut self) {
        let home = &env::var("HOME").unwrap();
        macro_rules! append_home_to {
            ( $($var:ident),* ) => {
                $(
                    if self.$var.is_relative() {
                        self.$var = Path::new(home).join(&self.$var);
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
