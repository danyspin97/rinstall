use clap::Parser;
use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};
use xdg::BaseDirectories;

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
    pub fn system_config() -> Self {
        Self {
            prefix: Some("/usr/local".to_string()),
            exec_prefix: Some("@prefix@".to_string()),
            bindir: Some("@exec_prefix@/bin".to_string()),
            sbindir: Some("@exec_prefix@/sbin".to_string()),
            libdir: Some("@exec_prefix@/lib".to_string()),
            libexecdir: Some("@exec_prefix@/libexec".to_string()),
            datarootdir: Some("@prefix@/share".to_string()),
            datadir: Some("@prefix@/share".to_string()),
            sysconfdir: Some("@prefix@/etc".to_string()),
            localstatedir: Some("@prefix@/var".to_string()),
            runstatedir: Some("@localstatedir@/run".to_string()),
            includedir: Some("@prefix@/include".to_string()),
            docdir: Some("@datarootdir@/doc".to_string()),
            mandir: Some("@datarootdir@/man".to_string()),
            pam_modulesdir: Some("@libdir@/security".to_string()),
            systemd_unitsdir: Some("@libdir@/systemd".to_string()),
        }
    }

    pub fn user_config() -> Self {
        Self {
            prefix: None,
            exec_prefix: None,
            bindir: Some(".local/bin".to_string()),
            sbindir: None,
            libdir: Some(".local/lib".to_string()),
            libexecdir: Some(".local/libexec".to_string()),
            datarootdir: Some("@XDG_DATA_HOME@".to_string()),
            datadir: Some("@XDG_DATA_HOME@".to_string()),
            sysconfdir: Some("@XDG_CONFIG_HOME@".to_string()),
            localstatedir: Some("@XDG_DATA_HOME@".to_string()),
            runstatedir: Some("@XDG_RUNTIME_DIR@".to_string()),
            includedir: None,
            docdir: None,
            mandir: None,
            pam_modulesdir: None,
            systemd_unitsdir: Some("@sysconfdir@/systemd".to_string()),
        }
    }

    pub fn merge_root_conf(
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

    pub fn merge_user_conf(
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
        replace!(localstatedir, "@XDG_DATA_HOME@", xdg.get_data_home());
        let runtime_directory = xdg
            .get_runtime_directory()
            .context("insecure XDG_RUNTIME_DIR found")?;
        replace!(runstatedir, "@XDG_RUNTIME_DIR@", runtime_directory);
        replace!(systemd_unitsdir, "@XDG_CONFIG_HOME@", xdg.get_config_home());
        replace!(systemd_unitsdir, "@sysconfdir@", xdg.get_config_home());

        Ok(())
    }
}
