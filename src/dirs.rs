use color_eyre::eyre::Result;
use std::path::PathBuf;

use crate::Config;

pub struct Dirs {
    pub user: bool,
    pub dry_run: bool,
    pub prefix: Option<PathBuf>,
    pub exec_prefix: Option<PathBuf>,
    pub bindir: PathBuf,
    pub libdir: PathBuf,
    pub datarootdir: PathBuf,
    pub datadir: PathBuf,
    pub sysconfdir: PathBuf,
    pub localstatedir: PathBuf,
    pub runstatedir: PathBuf,
    pub includedir: Option<PathBuf>,
    pub docdir: Option<PathBuf>,
    pub mandir: Option<PathBuf>,
}

impl Dirs {
    pub fn new(config: Config) -> Result<Dirs> {
        let mut dirs = Self {
            user: config.user,
            dry_run: config.dry_run,
            prefix: config.prefix.map(PathBuf::from),
            exec_prefix: config.exec_prefix.map(PathBuf::from),
            bindir: PathBuf::from(config.bindir.unwrap()),
            libdir: PathBuf::from(config.libdir.unwrap()),
            datarootdir: PathBuf::from(config.datarootdir.unwrap()),
            datadir: PathBuf::from(config.datadir.unwrap()),
            sysconfdir: PathBuf::from(config.sysconfdir.unwrap()),
            localstatedir: PathBuf::from(config.localstatedir.unwrap()),
            runstatedir: PathBuf::from(config.runstatedir.unwrap()),
            includedir: config.includedir.map(PathBuf::from),
            docdir: config.docdir.map(PathBuf::from),
            mandir: config.mandir.map(PathBuf::from),
        };

        if dirs.user {
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
                        self.$var = PathBuf::from(std::env::var("HOME").unwrap()).join(&self.$var);
                    }
                )*
            };
        }

        append_home_to!(
            bindir,
            libdir,
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
