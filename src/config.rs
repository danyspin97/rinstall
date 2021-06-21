use std::path::PathBuf;

use clap::Clap;
use color_eyre::eyre::{ContextCompat, Result};
use serde::Deserialize;
use xdg::BaseDirectories;

#[derive(Clap, Deserialize)]
#[clap(version = "0.1", author = "Danilo Spinella <oss@danyspin97.org>")]
pub struct Config {
    #[serde(skip_deserializing)]
    #[clap(short, long)]
    pub config: Option<String>,
    #[serde(skip_deserializing)]
    #[clap(short, long)]
    pub user: bool,
    #[serde(skip_deserializing)]
    #[clap(short = 'n', long)]
    pub dry_run: bool,
    #[serde(skip_deserializing)]
    #[clap(short = 'p', long)]
    pub package_dir: Option<String>,
    #[clap(long)]
    pub prefix: Option<String>,
    #[clap(long)]
    pub exec_prefix: Option<String>,
    #[clap(long)]
    pub bindir: Option<String>,
    #[clap(long)]
    pub libdir: Option<String>,
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
    #[clap(long)]
    pub includedir: Option<String>,
    #[clap(long)]
    pub docdir: Option<String>,
    #[clap(long)]
    pub mandir: Option<String>,
}

impl Config {
    pub fn new_default_root() -> Self {
        Self {
            config: None,
            user: false,
            dry_run: false,
            package_dir: None,
            prefix: Some("/usr/local".to_string()),
            exec_prefix: Some("@prefix@".to_string()),
            bindir: Some("@exec_prefix@/bin".to_string()),
            libdir: Some("@exec_prefix@/lib".to_string()),
            datarootdir: Some("@prefix@/share".to_string()),
            datadir: Some("@prefix@/share".to_string()),
            sysconfdir: Some("@prefix@/etc".to_string()),
            localstatedir: Some("@prefix@/var".to_string()),
            runstatedir: Some("@localstatedir@/run".to_string()),
            includedir: Some("@prefix@/include".to_string()),
            docdir: Some("@datarootdir@/doc".to_string()),
            mandir: Some("@datarootdir@/man".to_string()),
        }
    }

    pub fn new_default_user() -> Self {
        Self {
            config: None,
            user: true,
            dry_run: false,
            package_dir: None,
            prefix: None,
            exec_prefix: None,
            bindir: Some(".local/bin".to_string()),
            libdir: Some(".local/lib".to_string()),
            datarootdir: Some("@XDG_DATA_HOME@".to_string()),
            datadir: Some("@XDG_DATA_HOME@/share".to_string()),
            sysconfdir: Some("@XDG_CONFIG_HOME@".to_string()),
            localstatedir: Some("@XDG_STATE_HOME@".to_string()),
            runstatedir: Some("@XDG_RUNTIME_DIR@".to_string()),
            includedir: None,
            docdir: None,
            mandir: None,
        }
    }

    pub fn merge_root_conf(
        &mut self,
        config: Self,
    ) {
        macro_rules! update_fields {
            ($($field:tt),*) => {
                $(
                    if let Some($field) = config.$field {
                        self.$field = Some($field);
                    }
                )*
            };
        }

        update_fields!(
            prefix,
            exec_prefix,
            bindir,
            libdir,
            datarootdir,
            datadir,
            sysconfdir,
            localstatedir,
            runstatedir,
            includedir,
            docdir,
            mandir
        );
    }

    pub fn merge_user_conf(
        &mut self,
        config: Self,
    ) {
        macro_rules! update_fields {
            ($($field:tt),*) => {
                $(
                    if let Some($field) = config.$field {
                        self.$field = Some($field);
                    }
                )*
            };
        }

        update_fields!(
            bindir,
            libdir,
            datarootdir,
            datadir,
            sysconfdir,
            localstatedir,
            runstatedir
        );
    }

    pub fn replace_user_placeholders(
        &mut self,
        xdg: &BaseDirectories,
    ) -> Result<()> {
        macro_rules! replace {
            ( $var:ident, $needle:literal, $replacement:expr ) => {
                self.$var = Some(self.$var.as_ref().unwrap().replace(
                    $needle,
                    $replacement.as_os_str().to_str().with_context(|| {
                        format!("unable to convert {:?} to String", $replacement)
                    })?,
                ));
            };
        }

        replace!(datarootdir, "@XDG_DATA_HOME@", xdg.get_data_home());
        replace!(datadir, "@XDG_DATA_HOME@", xdg.get_data_home());
        replace!(sysconfdir, "@XDG_CONFIG_HOME@", xdg.get_config_home());
        replace!(
            localstatedir,
            "@XDG_STATE_HOME@",
            PathBuf::from(".local/state")
        );
        replace!(
            runstatedir,
            "@XDG_RUNTIME_DIR@",
            xdg.place_runtime_file(".").unwrap()
        );

        Ok(())
    }

    pub fn replace_root_placeholders(&mut self) {
        macro_rules! replace {
            ( $replacement:ident, $needle:literal, $($var:ident),* ) => {
                $(
                    self.$var = Some(self.$var
                        .as_ref()
                        .unwrap()
                        .replace($needle, self.$replacement.as_ref().unwrap()));
                )*
            };
        }

        replace!(
            prefix,
            "@prefix@",
            exec_prefix,
            bindir,
            libdir,
            datadir,
            datarootdir,
            sysconfdir,
            localstatedir,
            runstatedir,
            includedir,
            docdir,
            mandir
        );

        replace!(exec_prefix, "@exec_prefix@", bindir, libdir);
        replace!(localstatedir, "@localstatedir@", runstatedir);
        replace!(datarootdir, "@datarootdir@", docdir, mandir);
    }
}
