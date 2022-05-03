use std::env;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::ensure, Result};

use crate::DirsConfig;

pub struct Dirs {
    pub prefix: Option<Utf8PathBuf>,
    pub exec_prefix: Option<Utf8PathBuf>,
    pub bindir: Utf8PathBuf,
    pub sbindir: Option<Utf8PathBuf>,
    pub libdir: Utf8PathBuf,
    pub libexecdir: Utf8PathBuf,
    pub datarootdir: Utf8PathBuf,
    pub datadir: Utf8PathBuf,
    pub sysconfdir: Utf8PathBuf,
    pub localstatedir: Utf8PathBuf,
    pub runstatedir: Utf8PathBuf,
    pub includedir: Option<Utf8PathBuf>,
    pub docdir: Option<Utf8PathBuf>,
    pub mandir: Option<Utf8PathBuf>,
    pub pam_modulesdir: Option<Utf8PathBuf>,
    pub systemd_unitsdir: Utf8PathBuf,
}

impl Dirs {
    pub fn new(
        dirs_config: DirsConfig,
        system: bool,
    ) -> Result<Self> {
        let mut dirs = Self {
            prefix: dirs_config.prefix.map(Utf8PathBuf::from),
            exec_prefix: dirs_config.exec_prefix.map(Utf8PathBuf::from),
            bindir: Utf8PathBuf::from(dirs_config.bindir.unwrap()),
            sbindir: dirs_config.sbindir.map(Utf8PathBuf::from),
            libdir: Utf8PathBuf::from(dirs_config.libdir.unwrap()),
            libexecdir: Utf8PathBuf::from(dirs_config.libexecdir.unwrap()),
            datarootdir: Utf8PathBuf::from(dirs_config.datarootdir.unwrap()),
            datadir: Utf8PathBuf::from(dirs_config.datadir.unwrap()),
            sysconfdir: Utf8PathBuf::from(dirs_config.sysconfdir.unwrap()),
            localstatedir: Utf8PathBuf::from(dirs_config.localstatedir.unwrap()),
            runstatedir: Utf8PathBuf::from(dirs_config.runstatedir.unwrap()),
            includedir: dirs_config.includedir.map(Utf8PathBuf::from),
            docdir: dirs_config.docdir.map(Utf8PathBuf::from),
            mandir: dirs_config.mandir.map(Utf8PathBuf::from),
            pam_modulesdir: dirs_config.pam_modulesdir.map(Utf8PathBuf::from),
            systemd_unitsdir: Utf8PathBuf::from(dirs_config.systemd_unitsdir.unwrap()),
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
                        self.$var = Utf8Path::new(home).join(&self.$var);
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
            runstatedir,
            systemd_unitsdir
        );
    }

    /// Check that all paths are absolute
    fn check_absolute_paths(&self) -> Result<()> {
        macro_rules! check_abs_path_impl {
            ($var:expr, $name:tt) => {
                ensure!(
                    $var.is_absolute(),
                    "{}, with path '{}', is not an absolute path",
                    $name,
                    $var
                );
            };
        }
        macro_rules! check_abs_path {
            ( $($var:ident, $name:tt),* ) => {
                $(
                    check_abs_path_impl!(self.$var, $name);
                )*
            };
        }
        macro_rules! check_abs_path_opt {
            ( $($var:ident, $name:tt),* ) => {
                $(
                    if let Some(ref path) = self.$var {
                        check_abs_path_impl!(path, $name);
                    }
                )*
            };
        }

        check_abs_path!(
            bindir,
            "bindir",
            libdir,
            "libdir",
            libexecdir,
            "libexecdir",
            datarootdir,
            "datarootdir",
            datadir,
            "datadir",
            sysconfdir,
            "sysconfdir",
            localstatedir,
            "localstatedir",
            runstatedir,
            "runstatedir",
            systemd_unitsdir,
            "systemd-unitsdir"
        );

        check_abs_path_opt!(
            prefix,
            "prefix",
            exec_prefix,
            "exec_prefix",
            includedir,
            "includedir",
            docdir,
            "docdir",
            mandir,
            "mandir",
            pam_modulesdir,
            "pam_modulesdir"
        );

        Ok(())
    }
}
