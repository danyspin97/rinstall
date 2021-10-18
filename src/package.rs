use std::path::{Path, PathBuf};

use color_eyre::eyre::{ensure, Context, ContextCompat, Result};
use semver::{Version, VersionReq};
use serde::Deserialize;

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

#[derive(Deserialize, Default)]
struct Completions {
    #[serde(default)]
    pub bash: Vec<Entry>,
    #[serde(default)]
    pub fish: Vec<Entry>,
    #[serde(default)]
    pub zsh: Vec<Entry>,
}

#[derive(Deserialize)]
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
}

macro_rules! entry {
    ( $x:ident ) => {
        match $x {
            Entry::InstallEntry(entry) => entry,
        }
    };
}

macro_rules! check_version {
    ( $version:ident, $req:literal ) => {
        ensure!(VersionReq::parse($req).unwrap().matches(&$version), "fail");
    };
}

impl Package {
    pub fn targets(
        self,
        dirs: &Dirs,
        project: Project,
        rinstall_version: &Version,
    ) -> Result<Vec<InstallTarget>> {
        let allowed_version = vec!["0.1.0"];
        allowed_version
            .iter()
            .map(|v| Version::parse(v).unwrap())
            .find(|v| v == rinstall_version)
            .with_context(|| format!("{} is not a valid rinstall version", rinstall_version))?;

        let package_name = self.name.unwrap().to_owned();
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
            check_version!(rinstall_version, ">=0.1.0");
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
            check_version!(rinstall_version, ">=0.1.0");
            results.extend(install_files!(man, mandir, &project.projectdir, "man"));
        }

        if let Some(docdir) = &dirs.docdir {
            check_version!(rinstall_version, ">=0.1.0");
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

        results.extend(install_files!(
            appstream_metadata,
            &dirs.datarootdir.join("metainfo"),
            &project.projectdir,
            "appstream_metadata"
        ));

        results.extend(
            self.completions
                .bash
                .into_iter()
                .map(|completion| (completion, "bash-completion/completions"))
                .chain(
                    self.completions
                        .fish
                        .into_iter()
                        .map(|completion| (completion, "fish/vendor_completions.d")),
                )
                .chain(
                    self.completions
                        .zsh
                        .into_iter()
                        .map(|completion| (completion, "zsh/site-functions")),
                )
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
            check_version!(rinstall_version, ">=0.1.0");
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

        Ok(results)
    }
}
