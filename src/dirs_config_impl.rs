use std::fs;

use camino::Utf8PathBuf;
use clap::Args;
use color_eyre::{
    eyre::{ensure, ContextCompat, WrapErr},
    Result,
};
use serde::Deserialize;
use xdg::BaseDirectories;

lazy_static! {
    static ref XDG: BaseDirectories = BaseDirectories::new()
        .context("unable to initialize XDG Base Directories")
        .unwrap();
}

include!("dirs_config.rs");

macro_rules! update_fields {
    ($update:expr, $other:expr, $($field:tt),*) => {
        $(
            if let Some($field) = $other.$field {
                $update.$field = Some($field);
            }
        )*
    };
}

impl DirsConfig {
    pub fn load(
        config: Option<&str>,
        system: bool,
        opts: &mut Self,
    ) -> Result<Self> {
        let mut dirs_config = if system {
            Self::system_config()
        } else {
            Self::user_config()
        };

        let config_file = if let Some(config_file) = config {
            let config_file = Utf8PathBuf::from(config_file);
            ensure!(config_file.exists(), "config file does not exist");
            config_file
        } else if system {
            Utf8PathBuf::from("/etc/rinstall.yml")
        } else {
            Utf8PathBuf::from_path_buf(XDG.place_config_file("rinstall.yml")?).unwrap()
        };
        if config_file.exists() {
            let config_from_file = serde_yaml::from_str(
                &fs::read_to_string(&config_file)
                    .with_context(|| format!("unable to read file {:?}", config_file))?,
            )?;
            dirs_config.merge(system, config_from_file);
        }
        opts.sanitize();
        dirs_config.merge(system, opts.clone());
        dirs_config.replace_placeholders(system)?;

        Ok(dirs_config)
    }

    #[must_use]
    pub fn system_config() -> Self {
        Self {
            prefix: Some("/usr/local/".to_string()),
            exec_prefix: Some("@prefix@".to_string()),
            bindir: Some("@exec_prefix@bin/".to_string()),
            sbindir: Some("@exec_prefix@sbin/".to_string()),
            libdir: Some("@exec_prefix@lib/".to_string()),
            libexecdir: Some("@exec_prefix@libexec/".to_string()),
            datarootdir: Some("@prefix@share/".to_string()),
            datadir: Some("@prefix@share/".to_string()),
            sysconfdir: Some("@prefix@etc/".to_string()),
            localstatedir: Some("@prefix@var/".to_string()),
            runstatedir: Some("@localstatedir@run/".to_string()),
            includedir: Some("@prefix@include/".to_string()),
            docdir: Some("@datarootdir@doc/".to_string()),
            mandir: Some("@datarootdir@man/".to_string()),
            pam_modulesdir: Some("@libdir@security/".to_string()),
            systemd_unitsdir: Some("@libdir@systemd/".to_string()),
        }
    }

    #[must_use]
    pub fn user_config() -> Self {
        Self {
            prefix: None,
            exec_prefix: None,
            bindir: Some(".local/bin/".to_string()),
            sbindir: None,
            libdir: Some(".local/lib/".to_string()),
            libexecdir: Some(".local/libexec/".to_string()),
            datarootdir: Some("@XDG_DATA_HOME@/".to_string()),
            datadir: Some("@XDG_DATA_HOME@/".to_string()),
            sysconfdir: Some("@XDG_CONFIG_HOME@/".to_string()),
            localstatedir: Some("@XDG_DATA_HOME@/".to_string()),
            runstatedir: Some("@XDG_RUNTIME_DIR@/".to_string()),
            includedir: None,
            docdir: None,
            mandir: None,
            pam_modulesdir: None,
            systemd_unitsdir: Some("@sysconfdir@/systemd/".to_string()),
        }
    }

    pub fn merge(
        &mut self,
        system: bool,
        other: Self,
    ) {
        if system {
            self.merge_root_conf(other);
        } else {
            self.merge_user_conf(other);
        }
    }

    fn merge_root_conf(
        &mut self,
        config: Self,
    ) {
        update_fields!(
            self,
            config,
            prefix,
            exec_prefix,
            bindir,
            sbindir,
            libdir,
            libexecdir,
            datarootdir,
            datadir,
            sysconfdir,
            localstatedir,
            runstatedir,
            includedir,
            docdir,
            mandir,
            pam_modulesdir,
            systemd_unitsdir
        );
    }

    fn merge_user_conf(
        &mut self,
        config: Self,
    ) {
        update_fields!(
            self,
            config,
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

    pub fn replace_placeholders(
        &mut self,
        system: bool,
    ) -> Result<()> {
        if system {
            self.replace_root_placeholders();
        } else {
            self.replace_user_placeholders(&XDG)
                .context("unable to sanitize user directories")?;
        }

        Ok(())
    }

    fn replace_root_placeholders(&mut self) {
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
            sbindir,
            libdir,
            libexecdir,
            datadir,
            datarootdir,
            sysconfdir,
            localstatedir,
            runstatedir,
            includedir,
            docdir,
            mandir,
            pam_modulesdir,
            systemd_unitsdir
        );

        replace!(
            exec_prefix,
            "@exec_prefix@",
            bindir,
            sbindir,
            libdir,
            libexecdir
        );
        replace!(localstatedir, "@localstatedir@", runstatedir);
        replace!(datarootdir, "@datarootdir@", docdir, mandir);
        replace!(libdir, "@libdir@", pam_modulesdir, systemd_unitsdir);
    }

    fn replace_user_placeholders(
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
        replace!(localstatedir, "@XDG_DATA_HOME@", xdg.get_data_home());
        let runtime_directory = xdg
            .get_runtime_directory()
            .context("insecure XDG_RUNTIME_DIR found")?;
        replace!(runstatedir, "@XDG_RUNTIME_DIR@", runtime_directory);
        replace!(systemd_unitsdir, "@XDG_CONFIG_HOME@", xdg.get_config_home());
        replace!(systemd_unitsdir, "@sysconfdir@", xdg.get_config_home());

        Ok(())
    }

    fn sanitize(&mut self) {
        macro_rules! add_ending_slash {
            ( $($var:ident),* ) => {
                $(
                    if let Some(path) = self.$var.as_mut() {
                        if !path.ends_with("/") {
                            path.push_str("/");
                        }
                    }
                )*
            };
        }

        add_ending_slash!(
            prefix,
            exec_prefix,
            bindir,
            sbindir,
            libdir,
            libexecdir,
            datarootdir,
            datadir,
            sysconfdir,
            localstatedir,
            runstatedir,
            includedir,
            docdir,
            mandir,
            pam_modulesdir,
            systemd_unitsdir
        );
    }
}
