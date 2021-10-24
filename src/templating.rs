use std::{fs, path::Path};

use color_eyre::eyre::{ensure, Context, ContextCompat, Result};

use crate::Dirs;

pub struct Templating {
    pub contents: String,
}

impl Templating {
    pub fn new(source: &Path) -> Result<Self> {
        Ok(Self {
            contents: fs::read_to_string(&source)
                .with_context(|| format!("unable to read file {:?}", source))?,
        })
    }

    pub fn apply(
        &mut self,
        dirs: &Dirs,
    ) -> Result<()> {
        macro_rules! replace_impl {
            ( $dir:expr, $needle:literal ) => {
                self.contents = self.contents.replace(
                    $needle,
                    $dir.as_os_str()
                        .to_str()
                        .with_context(|| format!("unable to convert {:?} to String", $dir))?,
                );
            };
        }

        macro_rules! replace {
            ( $dir:ident, $needle:literal ) => {
                replace_impl!(&dirs.$dir, $needle);
            };
        }

        macro_rules! replace_when_some {
            ( $dir:ident, $needle:literal ) => {
                if let Some($dir) = &dirs.$dir {
                    replace_impl!($dir, $needle);
                } else {
                    // TODO: Is this needed?
                    ensure!(
                        !self.contents.contains($needle),
                        "tried replacing {} when its value is none",
                        $needle
                    );
                }
            };
        }

        replace_when_some!(prefix, "@prefix@");
        replace_when_some!(exec_prefix, "@exec_prefix@");
        replace!(bindir, "@bindir@");
        replace!(libdir, "@libdir@");
        replace!(datarootdir, "@datarootdir@");
        replace!(datadir, "@datadir@");
        replace!(sysconfdir, "@sysconfdir@");
        replace!(localstatedir, "@localstatedir@");
        replace!(runstatedir, "@runstatedir@");
        replace_when_some!(includedir, "@includedir@");
        replace_when_some!(docdir, "@docdir@");
        replace_when_some!(mandir, "@mandir@");
        replace_when_some!(pam_modulesdir, "@pam_moduledirs@");
        replace!(systemd_unitsdir, "@systemd_unitsdir@");

        Ok(())
    }
}
