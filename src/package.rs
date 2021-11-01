use std::path::{Path, PathBuf};

use color_eyre::eyre::{ensure, Context, ContextCompat, Result};
use semver::{Version, VersionReq};
use serde::Deserialize;

use crate::icon::Icon;
use crate::install_entry::{string_or_struct, InstallEntry};
use crate::install_target::InstallTarget;
use crate::project::Project;
use crate::Dirs;

#[derive(Deserialize, Clone)]
pub enum Type {
    #[serde(rename(deserialize = "custom"))]
    Custom,
    #[serde(rename(deserialize = "rust"))]
    Rust,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Entry {
    #[serde(deserialize_with = "string_or_struct")]
    InstallEntry(InstallEntry),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum IconEntry {
    #[serde(deserialize_with = "string_or_struct")]
    Icon(Icon),
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Completions {
    #[serde(default)]
    pub bash: Vec<Entry>,
    #[serde(default)]
    pub fish: Vec<Entry>,
    #[serde(default)]
    pub zsh: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub project_type: Type,
    #[serde(default)]
    exe: Vec<Entry>,
    #[serde(default, rename(deserialize = "admin-exe"))]
    admin_exe: Vec<Entry>,
    #[serde(default)]
    libs: Vec<Entry>,
    #[serde(default)]
    libexec: Vec<Entry>,
    #[serde(default)]
    man: Vec<Entry>,
    #[serde(default)]
    data: Vec<Entry>,
    #[serde(default)]
    docs: Vec<Entry>,
    #[serde(default)]
    config: Vec<Entry>,
    #[serde(default, rename(deserialize = "desktop-files"))]
    desktop_files: Vec<Entry>,
    #[serde(default, rename(deserialize = "appstream-metadata"))]
    appstream_metadata: Vec<Entry>,
    #[serde(default)]
    completions: Completions,
    #[serde(default, rename(deserialize = "pam-modules"))]
    pam_modules: Vec<Entry>,
    #[serde(default, rename(deserialize = "systemd-units"))]
    systemd_units: Vec<Entry>,
    #[serde(default)]
    icons: Vec<IconEntry>,
    #[serde(default)]
    terminfo: Vec<Entry>,
}

macro_rules! entry {
    ( $x:ident ) => {
        match $x {
            Entry::InstallEntry(entry) => entry,
        }
    };
}

impl Package {
    pub fn targets(
        self,
        dirs: &Dirs,
        project: Project,
        rinstall_version: &Version,
        system_install: bool,
    ) -> Result<Vec<InstallTarget>> {
        let allowed_version = vec!["0.1.0"];
        allowed_version
            .iter()
            .map(|v| Version::parse(v).unwrap())
            .find(|v| v == rinstall_version)
            .with_context(|| format!("{} is not a valid rinstall version", rinstall_version))?;

        self.check_entries(rinstall_version)?;

        let package_name = self.name.unwrap();
        let mut results = Vec::new();

        macro_rules! install_files {
            ( $files:tt, $install_dir:expr, $parent_dir:expr, $name:literal ) => {
                self.$files
                    .into_iter()
                    .map(|entry| -> Result<InstallTarget> {
                        InstallTarget::new(entry!(entry), $install_dir, $parent_dir)
                    })
                    .collect::<Result<Vec<InstallTarget>>>()
                    .with_context(|| format!("error while iterating {} files", $name))?
            };
        }

        results.extend(install_files!(exe, &dirs.bindir, &project.outputdir, "exe"));
        if let Some(sbindir) = &dirs.sbindir {
            results.extend(install_files!(
                admin_exe,
                sbindir,
                &project.outputdir,
                "admin_exe"
            ));
        }
        results.extend(install_files!(
            libs,
            &dirs.libdir,
            &project.outputdir,
            "libs"
        ));
        results.extend(install_files!(
            libexec,
            &dirs.libexecdir,
            &project.outputdir,
            "libexec"
        ));
        results.extend(install_files!(
            data,
            &dirs.datadir.join(&package_name),
            &project.projectdir,
            "data"
        ));
        results.extend(install_files!(
            config,
            &dirs.sysconfdir,
            &project.projectdir,
            "config"
        ));
        if let Some(mandir) = &dirs.mandir {
            results.extend(install_files!(man, mandir, &project.projectdir, "man"));
        }

        if let Some(docdir) = &dirs.docdir {
            results.extend(install_files!(
                docs,
                &docdir.join(Path::new(&package_name)),
                &project.projectdir,
                "docs"
            ));
        }

        results.extend(install_files!(
            desktop_files,
            &dirs.datarootdir.join("applications"),
            &project.projectdir,
            "desktop"
        ));

        if system_install {
            results.extend(install_files!(
                appstream_metadata,
                &dirs.datarootdir.join("metainfo"),
                &project.projectdir,
                "appstream_metadata"
            ));
        }

        let mut completions = self
            .completions
            .bash
            .into_iter()
            .map(|completion| {
                (
                    completion,
                    if system_install {
                        "bash-completion/completions"
                    } else {
                        "bash-completion"
                    },
                )
            })
            .collect::<Vec<(Entry, &'static str)>>();
        if system_install {
            completions.extend(
                self.completions
                    .fish
                    .into_iter()
                    .map(|completion| (completion, "fish/vendor_completions.d")),
            );
            completions.extend(
                self.completions
                    .zsh
                    .into_iter()
                    .map(|completion| (completion, "zsh/site-functions")),
            );
        }
        results.extend(
            completions
                .into_iter()
                .map(|(entry, completionsdir)| -> Result<InstallTarget> {
                    InstallTarget::new(
                        entry!(entry),
                        &dirs.datarootdir.join(completionsdir),
                        &project.projectdir,
                    )
                })
                .collect::<Result<Vec<InstallTarget>>>()
                .context("error while iterating completion files")?,
        );

        if let Some(pam_modulesdir) = &dirs.pam_modulesdir {
            results.extend(
                self.pam_modules
                    .into_iter()
                    .map(|entry| {
                        let Entry::InstallEntry(InstallEntry {
                            source,
                            destination,
                            templating,
                        }) = entry;

                        let destination = if destination.is_some() {
                            destination
                        } else {
                            let file_name = source
                                .file_name()
                                .with_context(|| {
                                    format!("unable to get file name of file {:?}", source)
                                })?
                                .to_str()
                                .unwrap();
                            if file_name.starts_with("libpam_") {
                                Some(PathBuf::from(file_name.strip_prefix("lib").unwrap()))
                            } else {
                                None
                            }
                        };

                        InstallTarget::new(
                            InstallEntry {
                                source,
                                destination,
                                templating,
                            },
                            pam_modulesdir,
                            &project.outputdir,
                        )
                    })
                    .collect::<Result<Vec<InstallTarget>>>()
                    .context("error while iterating pam-modules")?,
            );
        }

        results.extend(install_files!(
            systemd_units,
            &dirs.systemd_unitsdir,
            &project.projectdir,
            "systemd_units"
        ));

        results.extend(
            self.icons
                .into_iter()
                .map(|icon| -> Icon {
                    match icon {
                        IconEntry::Icon(icon) => icon,
                    }
                })
                .filter(|icon| system_install || !icon.pixmaps)
                .map(|icon| -> Result<InstallTarget> {
                    InstallTarget::new(
                        InstallEntry {
                            source: icon.source.clone(),
                            destination: Some(icon.get_destination().with_context(|| {
                                format!(
                                    "unable to generate destination for icon {:?}",
                                    icon.source.clone()
                                )
                            })?),
                            templating: false,
                        },
                        &dirs.datarootdir,
                        &project.projectdir,
                    )
                })
                .collect::<Result<Vec<InstallTarget>>>()
                .context("error while iterating icons")?,
        );

        if system_install {
            results.extend(
                self.terminfo
                    .into_iter()
                    .map(|entry| -> Result<InstallTarget> {
                        let entry = entry!(entry);
                        ensure!(
                            !entry
                                .source
                                .as_os_str()
                                .to_str()
                                .with_context(|| format!(
                                    "unable to convert {:?} to string",
                                    entry.source
                                ))?
                                .ends_with("/"),
                            "the terminfo entry cannot be a directory"
                        );
                        let use_source_name = if let Some(destination) = &entry.destination {
                            if !destination
                                .as_os_str()
                                .to_str()
                                .with_context(|| {
                                    format!("unable to convert {:?} to string", entry.source)
                                })?
                                .ends_with("/")
                            {
                                false
                            } else {
                                true
                            }
                        } else {
                            true
                        };
                        let name = if use_source_name {
                            &entry.source
                        } else {
                            entry.destination.as_ref().unwrap()
                        };
                        let initial = name
                            .file_name()
                            .with_context(|| format!("unable to get filename of file {:?}", name))?
                            .to_str()
                            .with_context(|| format!("unable to convert {:?} to string", name))?
                            .chars()
                            .next()
                            .with_context(|| {
                                format!("terminfo entry {:?} contains an empty filename", name)
                            })?
                            .to_lowercase()
                            .to_string();
                        let install_dir = dirs.datarootdir.join("terminfo").join(&initial);
                        InstallTarget::new(entry, &install_dir, &project.projectdir)
                    })
                    .collect::<Result<Vec<InstallTarget>>>()
                    .context("error while iterating terminfo files")?,
            );
        }

        Ok(results)
    }

    fn check_entries(
        &self,
        rinstall_version: &Version,
    ) -> Result<()> {
        macro_rules! check_version_expr {
            ( $version:ident, $name:literal, $type:expr, $req:literal ) => {
                let requires = VersionReq::parse($req).unwrap();
                ensure!(
                    $type.is_empty() || requires.matches(&$version),
                    "{} requires version {}",
                    $name,
                    requires
                );
            };
        }
        macro_rules! check_version {
            ( $version:ident, $name:literal, $type:ident, $req:literal ) => {
                check_version_expr!($version, $name, self.$type, $req);
            };
        }

        check_version!(rinstall_version, "exe", exe, ">=0.1.0");
        check_version!(rinstall_version, "admin_exe", admin_exe, ">=0.1.0");
        check_version!(rinstall_version, "libs", libs, ">=0.1.0");
        check_version!(rinstall_version, "libexec", libexec, ">=0.1.0");
        check_version!(rinstall_version, "man", man, ">=0.1.0");
        check_version!(rinstall_version, "data", data, ">=0.1.0");
        check_version!(rinstall_version, "docs", docs, ">=0.1.0");
        check_version!(rinstall_version, "config", config, ">=0.1.0");
        check_version!(rinstall_version, "desktop_files", desktop_files, ">=0.1.0");
        check_version!(
            rinstall_version,
            "appstream_metadata",
            appstream_metadata,
            ">=0.1.0"
        );
        check_version_expr!(
            rinstall_version,
            "pam_moduless",
            self.completions.bash,
            ">=0.1.0"
        );
        check_version_expr!(
            rinstall_version,
            "pam_moduless",
            self.completions.fish,
            ">=0.1.0"
        );
        check_version_expr!(
            rinstall_version,
            "pam_moduless",
            self.completions.zsh,
            ">=0.1.0"
        );
        check_version!(rinstall_version, "pam_modules", pam_modules, ">=0.1.0");
        check_version!(rinstall_version, "systemd_units", systemd_units, ">=0.1.0");
        check_version!(rinstall_version, "icons", icons, ">=0.1.0");
        check_version!(rinstall_version, "terminfo", terminfo, ">=0.1.0");

        Ok(())
    }
}
