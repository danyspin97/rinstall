use std::str::FromStr;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{
    eyre::{ensure, Context, ContextCompat},
    Result,
};
use colored::Colorize;
use log::warn;
use semver::{Version, VersionReq};
use serde::Deserialize;
use void::Void;

use crate::install_entry::{string_or_struct, InstallEntry};
use crate::install_target::InstallTarget;
use crate::Dirs;
use crate::{icon::Icon, install_target::FilesPolicy};

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum Type {
    #[serde(rename(deserialize = "default"))]
    Default,
    #[serde(rename(deserialize = "rust"))]
    Rust,
    #[serde(rename(deserialize = "custom"))]
    Custom,
}

impl Default for Type {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Entry {
    #[serde(deserialize_with = "string_or_struct")]
    InstallEntry(InstallEntry),
}

// DataEntry is not really a good name, it is just an Entry with use_pkg_name option
#[derive(Deserialize)]
#[serde(untagged)]
enum DataEntry {
    #[serde(deserialize_with = "string_or_struct")]
    DataInstallEntry(DataInstallEntry),
}

#[derive(Deserialize)]
struct DataInstallEntry {
    #[serde(rename(deserialize = "use-pkg-name"), default = "bool_true")]
    use_pkg_name: bool,
    #[serde(flatten)]
    entry: InstallEntry,
}

impl FromStr for DataInstallEntry {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DataInstallEntry {
            use_pkg_name: true,
            entry: InstallEntry::new_with_source(Utf8PathBuf::from(s)),
        })
    }
}

const fn bool_true() -> bool {
    true
}

#[derive(Deserialize)]
struct IconEntry {
    #[serde(flatten, deserialize_with = "string_or_struct")]
    icon: Icon,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Completions {
    #[serde(default)]
    pub bash: Vec<Entry>,
    #[serde(default)]
    pub elvish: Vec<Entry>,
    #[serde(default)]
    pub fish: Vec<Entry>,
    #[serde(default)]
    pub zsh: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: Option<String>,
    #[serde(rename(deserialize = "type"), default)]
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
    includes: Vec<Entry>,
    #[serde(default)]
    man: Vec<Entry>,
    #[serde(default)]
    data: Vec<DataEntry>,
    #[serde(default)]
    docs: Vec<DataEntry>,
    #[serde(default)]
    config: Vec<DataEntry>,
    #[serde(default, rename(deserialize = "user-config"))]
    user_config: Vec<DataEntry>,
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
    #[serde(default, rename(deserialize = "systemd-user-units"))]
    systemd_user_units: Vec<Entry>,
    #[serde(default)]
    icons: Vec<IconEntry>,
    #[serde(default)]
    terminfo: Vec<Entry>,
    #[serde(default)]
    licenses: Vec<DataEntry>,
    #[serde(default, rename(deserialize = "pkg-config"))]
    pkg_config: Vec<Entry>,
}

impl Package {
    // Generate a vector of InstallTarget from a package defined in install.yml
    pub fn targets(
        self,
        dirs: &Dirs,
        rinstall_version: &Version,
        system_install: bool,
    ) -> Result<Vec<InstallTarget>> {
        let allowed_version = vec!["0.1.0", "0.2.0"];
        allowed_version
            .iter()
            .map(|v| Version::parse(v).unwrap())
            .find(|v| v == rinstall_version)
            .with_context(|| format!("{} is not a valid rinstall version", rinstall_version))?;

        self.check_entries(rinstall_version)?;

        let package_name = self.name.unwrap();
        let mut results = Vec::new();

        fn get_files(
            files: Vec<Entry>,
            install_dir: &Utf8Path,
            name: &str,
            replace: FilesPolicy,
        ) -> Result<Vec<InstallTarget>> {
            files
                .into_iter()
                .map(|entry| -> Result<InstallTarget> {
                    InstallTarget::new(
                        match entry {
                            Entry::InstallEntry(entry) => entry,
                        },
                        install_dir,
                        replace,
                    )
                })
                .collect::<Result<Vec<InstallTarget>>>()
                .with_context(|| format!("error while iterating {} files", name))
        }

        fn get_files_dataentry(
            files: Vec<DataEntry>,
            install_dir: &Utf8Path,
            name: &str,
            package_name: &str,
            replace: FilesPolicy,
        ) -> Result<Vec<InstallTarget>> {
            let with_pkgname = install_dir.join(&package_name);
            files
                .into_iter()
                .map(|entry| -> Result<InstallTarget> {
                    let DataEntry::DataInstallEntry(entry) = entry;
                    // Check the use_pkg_name option for this entry
                    // When it is enabled (which it is by default) this entry will be installed
                    // with $install_dir/<pkg-name>/ as root director for dst
                    let install_dir = if entry.use_pkg_name {
                        &with_pkgname
                    } else {
                        install_dir
                    };
                    let entry = entry.entry;
                    InstallTarget::new(entry, install_dir, replace)
                })
                .collect::<Result<Vec<InstallTarget>>>()
                .with_context(|| format!("error while iterating {name} files"))
        }

        results.extend(get_files(
            self.exe,
            &dirs.bindir,
            "exe",
            FilesPolicy::Replace,
        )?);

        if let Some(sbindir) = &dirs.sbindir {
            results.extend(get_files(
                self.admin_exe,
                sbindir,
                "admin_exe",
                FilesPolicy::Replace,
            )?);
        }
        results.extend(get_files(
            self.libs,
            &dirs.libdir,
            "libs",
            FilesPolicy::Replace,
        )?);
        results.extend(get_files(
            self.libexec,
            &dirs.libexecdir,
            "libexec",
            FilesPolicy::Replace,
        )?);
        if let Some(includedir) = &dirs.includedir {
            results.extend(get_files(
                self.includes,
                includedir,
                "includes",
                FilesPolicy::Replace,
            )?);
        }
        results.extend(get_files_dataentry(
            self.data,
            &dirs.datadir,
            "data",
            &package_name,
            FilesPolicy::Replace,
        )?);
        results.extend(get_files_dataentry(
            self.config,
            &dirs.sysconfdir,
            "config",
            &package_name,
            FilesPolicy::NoReplace,
        )?);

        if let Some(mandir) = &dirs.mandir {
            results.extend(
                self.man
                    .into_iter()
                    .map(|entry| -> Result<InstallTarget> {
                        let Entry::InstallEntry(entry) = entry;
                        ensure!(
                            !entry.source.as_str().ends_with('/'),
                            "the man entry cannot be a directory"
                        );
                        let use_source_name = entry
                            .destination
                            .as_ref()
                            .map_or(true, |destination| destination.as_str().ends_with('/'));
                        let name = if use_source_name {
                            &entry.source
                        } else {
                            entry.destination.as_ref().unwrap()
                        };
                        let man_cat = name
                            .extension()
                            .with_context(|| format!("unable to get extension of file {:?}", name))?
                            .to_string();
                        ensure!(
                            man_cat.chars().next().unwrap().is_ascii_digit(),
                            "the last character should be a digit from 1 to 8"
                        );
                        let install_dir = mandir.join(format!("man{}/", &man_cat));
                        InstallTarget::new(entry, &install_dir, FilesPolicy::Replace)
                    })
                    .collect::<Result<Vec<InstallTarget>>>()
                    .context("error while iterating man pages")?,
            );
        }

        if system_install {
            let pkg_docs = &dirs.docdir.as_ref().unwrap();
            results.extend(get_files_dataentry(
                self.docs,
                pkg_docs,
                "docs",
                &package_name,
                FilesPolicy::Replace,
            )?);
            // use-pkg-name doesn't make any sense for user-config, so skip it
            results.extend(get_files(
                self.user_config
                    .into_iter()
                    .map(|entry| {
                        Entry::InstallEntry(match entry {
                            DataEntry::DataInstallEntry(entry) => entry.entry,
                        })
                    })
                    .collect(),
                &pkg_docs.join("user-config/"),
                "user-config",
                FilesPolicy::Replace,
            )?);
        } else {
            results.extend(get_files(
                self.user_config
                    .into_iter()
                    .map(|entry| {
                        Entry::InstallEntry(match entry {
                            DataEntry::DataInstallEntry(entry) => entry.entry,
                        })
                    })
                    .collect(),
                &dirs.sysconfdir,
                "user-config",
                FilesPolicy::NoReplace,
            )?);
        }

        results.extend(get_files(
            self.desktop_files,
            &dirs.datarootdir.join("applications/"),
            "desktop-files",
            FilesPolicy::Replace,
        )?);

        if system_install {
            results.extend(get_files(
                self.appstream_metadata,
                &dirs.datarootdir.join("metainfo/"),
                "appstream-metadata",
                FilesPolicy::Replace,
            )?);
        }

        let mut completions = self
            .completions
            .bash
            .into_iter()
            .map(|completion| {
                (
                    completion,
                    if system_install {
                        "bash-completion/completions/"
                    } else {
                        "bash-completion/"
                    },
                )
            })
            .collect::<Vec<(Entry, &'static str)>>();
        completions.extend(
            self.completions
                .elvish
                .into_iter()
                .map(|completion| (completion, "elvish/lib/")),
        );
        if system_install {
            completions.extend(
                self.completions
                    .fish
                    .into_iter()
                    .map(|completion| (completion, "fish/vendor_completions.d/")),
            );
            completions.extend(
                self.completions
                    .zsh
                    .into_iter()
                    .map(|completion| (completion, "zsh/site-functions/")),
            );
        }
        results.extend(
            completions
                .into_iter()
                .map(|(entry, completionsdir)| -> Result<InstallTarget> {
                    let Entry::InstallEntry(entry) = entry;
                    InstallTarget::new(
                        entry,
                        &dirs.datarootdir.join(completionsdir),
                        FilesPolicy::Replace,
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
                            let file_name = source.file_name().unwrap();
                            if file_name.starts_with("libpam_") {
                                Some(Utf8PathBuf::from(file_name.strip_prefix("lib").unwrap()))
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
                            FilesPolicy::Replace,
                        )
                    })
                    .collect::<Result<Vec<InstallTarget>>>()
                    .context("error while iterating pam-modules")?,
            );
        }

        if system_install {
            results.extend(get_files(
                self.systemd_units,
                &dirs.systemd_unitsdir.join("system/"),
                "systemd-units",
                FilesPolicy::Replace,
            )?);
        }
        results.extend(get_files(
            self.systemd_user_units,
            &dirs.systemd_unitsdir.join("user/"),
            "systemd-user-units",
            FilesPolicy::Replace,
        )?);

        results.extend(
            self.icons
                .into_iter()
                .map(|icon| -> Icon { icon.icon })
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
                        FilesPolicy::Replace,
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
                        let Entry::InstallEntry(entry) = entry;
                        ensure!(
                            !entry.source.as_str().ends_with('/'),
                            "the terminfo entry cannot be a directory"
                        );
                        let use_source_name = entry
                            .destination
                            .as_ref()
                            .map_or(true, |destination| destination.as_str().ends_with('/'));
                        let name = if use_source_name {
                            &entry.source
                        } else {
                            entry.destination.as_ref().unwrap()
                        };
                        let initial = name
                            .file_name()
                            .with_context(|| format!("unable to get filename of file {:?}", name))?
                            .chars()
                            .next()
                            .with_context(|| {
                                format!("terminfo entry {:?} contains an empty filename", name)
                            })?
                            .to_lowercase()
                            .to_string();
                        let install_dir = dirs.datarootdir.join("terminfo").join(&initial);
                        InstallTarget::new(entry, &install_dir, FilesPolicy::Replace)
                    })
                    .collect::<Result<Vec<InstallTarget>>>()
                    .context("error while iterating terminfo files")?,
            );
        }

        results.extend(get_files_dataentry(
            self.licenses,
            &dirs.datarootdir.join("licenses/"),
            "licenses",
            &package_name,
            FilesPolicy::Replace,
        )?);

        if system_install {
            results.extend(get_files(
                self.pkg_config,
                &dirs.libdir.join("pkgconfig/"),
                "pkg-config",
                FilesPolicy::Replace,
            )?);
        }

        Ok(results)
    }

    fn check_entries(
        &self,
        rinstall_version: &Version,
    ) -> Result<()> {
        macro_rules! check_version_expr {
            ( $name:literal, $type:expr, $req:literal ) => {
                let requires = VersionReq::parse($req).unwrap();
                ensure!(
                    $type.is_empty() || requires.matches(&rinstall_version),
                    "{} requires version {}",
                    $name,
                    requires
                );
            };
        }
        macro_rules! check_version {
            ( $name:literal, $type:ident, $req:literal ) => {
                check_version_expr!($name, self.$type, $req);
            };
        }

        if self.project_type == Type::Custom
            && VersionReq::parse(">=0.2.0")
                .unwrap()
                .matches(rinstall_version)
        {
            warn!(
                "type '{}' has been deprecated, use '{}' or leave it empty",
                "custom".bright_black(),
                "default".bright_black(),
            );
        }
        check_version!("exe", exe, ">=0.1.0");
        check_version!("admin_exe", admin_exe, ">=0.1.0");
        check_version!("libs", libs, ">=0.1.0");
        check_version!("libexec", libexec, ">=0.1.0");
        check_version!("includes", includes, ">=0.1.0");
        check_version!("man", man, ">=0.1.0");
        check_version!("data", data, ">=0.1.0");
        check_version!("docs", docs, ">=0.1.0");
        check_version!("config", config, ">=0.1.0");
        check_version!("user-config", user_config, ">=0.1.0");
        check_version!("desktop-files", desktop_files, ">=0.1.0");
        check_version!("appstream-metadata", appstream_metadata, ">=0.1.0");
        check_version_expr!("completions:bash", self.completions.bash, ">=0.1.0");
        check_version_expr!("completions:elvish", self.completions.elvish, ">=0.2.0");
        check_version_expr!("completions:fish", self.completions.fish, ">=0.1.0");
        check_version_expr!("completions:zsh", self.completions.zsh, ">=0.1.0");
        check_version!("pam-modules", pam_modules, ">=0.1.0");
        check_version!("systemd-units", systemd_units, ">=0.1.0");
        check_version!("systemd-user-units", systemd_user_units, ">=0.2.0");
        check_version!("icons", icons, ">=0.1.0");
        check_version!("terminfo", terminfo, ">=0.1.0");
        check_version!("licenses", licenses, ">=0.1.0");
        check_version!("pkg-config", pkg_config, ">=0.1.0");

        Ok(())
    }
}
