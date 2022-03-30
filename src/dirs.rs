use std::{
    env,
    path::{Path, PathBuf},
};

use color_eyre::Result;

use crate::DirsConfig;

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
    pub fn new(
        dirs_config: DirsConfig,
        system: bool,
    ) -> Result<Self> {
        let mut dirs = Self {
            prefix: dirs_config.prefix.map(PathBuf::from),
            exec_prefix: dirs_config.exec_prefix.map(PathBuf::from),
            bindir: PathBuf::from(dirs_config.bindir.unwrap()),
            sbindir: dirs_config.sbindir.map(PathBuf::from),
            libdir: PathBuf::from(dirs_config.libdir.unwrap()),
            libexecdir: PathBuf::from(dirs_config.libexecdir.unwrap()),
            datarootdir: PathBuf::from(dirs_config.datarootdir.unwrap()),
            datadir: PathBuf::from(dirs_config.datadir.unwrap()),
            sysconfdir: PathBuf::from(dirs_config.sysconfdir.unwrap()),
            localstatedir: PathBuf::from(dirs_config.localstatedir.unwrap()),
            runstatedir: PathBuf::from(dirs_config.runstatedir.unwrap()),
            includedir: dirs_config.includedir.map(PathBuf::from),
            docdir: dirs_config.docdir.map(PathBuf::from),
            mandir: dirs_config.mandir.map(PathBuf::from),
            pam_modulesdir: dirs_config.pam_modulesdir.map(PathBuf::from),
            systemd_unitsdir: PathBuf::from(dirs_config.systemd_unitsdir.unwrap()),
        };

        if system {
            dirs.check_absolute_paths()?;
        } else {
            dirs.append_home();
        }

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
