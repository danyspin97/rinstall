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

use crate::{
    icon::Icon,
    install_spec::RinstallVersion,
    install_target::FilesPolicy,
    project::{ProjectDirectories, RUST_DIRECTORIES},
    string_or_struct::string_or_struct,
};
use crate::{install_target::InstallEntry, Dirs};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnerEntry {
    #[serde(rename(deserialize = "src"))]
    pub source: Utf8PathBuf,
    #[serde(rename(deserialize = "dst"))]
    pub destination: Option<Utf8PathBuf>,
    #[serde(default, rename(deserialize = "tmpl"))]
    pub templating: bool,
}

impl InnerEntry {
    pub const fn new_with_source(source: Utf8PathBuf) -> Self {
        Self {
            source,
            destination: None,
            templating: false,
        }
    }

    fn new_entry(
        self,
        policy: FilesPolicy,
        install_dir: &Utf8Path,
        sourcepath: &dyn Fn(&Utf8Path) -> Utf8PathBuf,
    ) -> Result<InstallEntry> {
        let replace = matches!(policy, FilesPolicy::Replace);
        ensure!(
            self.source.is_relative(),
            "the source file {:?} is not relative",
            self.source
        );

        let destination =
            if self.source.is_file() || self.destination.is_some() {
                install_dir.join(if let Some(destination) = self.destination {
                    ensure!(
                        destination.is_relative(),
                        "the destination part of a file must be relative"
                    );
                    destination
                } else {
                    Utf8PathBuf::from(self.source.file_name().with_context(|| {
                        format!("unable to get file name from {:?}", self.source)
                    })?)
                })
            } else {
                install_dir.to_path_buf()
            };

        let full_source = sourcepath(&self.source);
        Ok(InstallEntry {
            source: self.source,
            full_source,
            destination,
            templating: self.templating,
            replace,
        })
    }
}

impl FromStr for InnerEntry {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new_with_source(Utf8PathBuf::from(s)))
    }
}

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
#[serde(transparent)]
struct Entry {
    #[serde(deserialize_with = "string_or_struct")]
    pub entry: InnerEntry,
}

// DataEntry is not really a good name, it is just an Entry with use_pkg_name option
#[derive(Deserialize)]
#[serde(transparent)]
struct DataEntry {
    #[serde(deserialize_with = "string_or_struct")]
    pub entry: InnerDataEntry,
}

#[derive(Deserialize)]
struct InnerDataEntry {
    #[serde(rename(deserialize = "use-pkg-name"), default = "bool_true")]
    use_pkg_name: bool,
    #[serde(flatten)]
    entry: InnerEntry,
}

impl InnerDataEntry {
    pub fn new_with_source(s: Utf8PathBuf) -> InnerDataEntry {
        InnerDataEntry {
            use_pkg_name: true,
            entry: InnerEntry::new_with_source(s),
        }
    }
}

impl FromStr for InnerDataEntry {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(InnerDataEntry::new_with_source(Utf8PathBuf::from(s)))
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
    bash: Vec<Entry>,
    #[serde(default)]
    elvish: Vec<Entry>,
    #[serde(default)]
    fish: Vec<Entry>,
    #[serde(default)]
    zsh: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: Option<String>,
    #[serde(rename(deserialize = "type"), default)]
    pub pkg_type: Type,
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
fn get_files(
    files: Vec<Entry>,
    install_dir: &Utf8Path,
    name: &str,
    replace: FilesPolicy,
    sourcepath: &dyn Fn(&Utf8Path) -> Utf8PathBuf,
) -> Result<Vec<InstallEntry>> {
    files
        .into_iter()
        .map(|entry| -> Result<InstallEntry> {
            entry.entry.new_entry(replace, install_dir, &sourcepath)
        })
        .collect::<Result<Vec<InstallEntry>>>()
        .with_context(|| format!("error while iterating {} files", name))
}
fn get_files_dataentry(
    files: Vec<DataEntry>,
    install_dir: &Utf8Path,
    name: &str,
    package_name: &str,
    replace: FilesPolicy,
    sourcepath: &dyn Fn(&Utf8Path) -> Utf8PathBuf,
) -> Result<Vec<InstallEntry>> {
    let with_pkgname = install_dir.join(package_name);
    files
        .into_iter()
        .map(|entry| -> Result<InstallEntry> {
            let entry = entry.entry;
            // Check the use_pkg_name option for this entry
            // When it is enabled (which it is by default) this entry will be installed
            // with $install_dir/<pkg-name>/ as root director for dst
            let install_dir = if entry.use_pkg_name {
                &with_pkgname
            } else {
                install_dir
            };
            entry.entry.new_entry(replace, install_dir, &sourcepath)
        })
        .collect::<Result<Vec<InstallEntry>>>()
        .with_context(|| format!("error while iterating {name} files"))
}

impl Package {
    /// Generate a vector of InstallTarget from a package defined in install.yml
    pub fn targets(
        self,
        dirs: &Dirs,
        rinstall_version: &RinstallVersion,
        system_install: bool,
    ) -> Result<Vec<InstallEntry>> {
        self.check_entries(rinstall_version)?;

        let package_name = self.name.unwrap();
        let mut results = Vec::new();

        let sourcepath = match self.pkg_type {
            Type::Default => todo!(),
            Type::Rust => unsafe { RUST_DIRECTORIES.as_ref().unwrap() },
            Type::Custom => todo!(),
        }
        .sourcepath();

        results.extend(get_files(
            self.exe,
            &dirs.bindir,
            "exe",
            FilesPolicy::Replace,
            &sourcepath,
        )?);

        if let Some(sbindir) = &dirs.sbindir {
            results.extend(get_files(
                self.admin_exe,
                sbindir,
                "admin_exe",
                FilesPolicy::Replace,
                &sourcepath,
            )?);
        }
        results.extend(get_files(
            self.libs,
            &dirs.libdir,
            "libs",
            FilesPolicy::Replace,
            &sourcepath,
        )?);
        results.extend(get_files(
            self.libexec,
            &dirs.libexecdir,
            "libexec",
            FilesPolicy::Replace,
            &sourcepath,
        )?);
        if let Some(includedir) = &dirs.includedir {
            results.extend(get_files(
                self.includes,
                includedir,
                "includes",
                FilesPolicy::Replace,
                &sourcepath,
            )?);
        }
        results.extend(get_files_dataentry(
            self.data,
            &dirs.datadir,
            "data",
            &package_name,
            FilesPolicy::Replace,
            &sourcepath,
        )?);
        results.extend(get_files_dataentry(
            self.config,
            &dirs.sysconfdir,
            "config",
            &package_name,
            FilesPolicy::NoReplace,
            &sourcepath,
        )?);

        if let Some(mandir) = &dirs.mandir {
            results.extend(
                self.man
                    .into_iter()
                    .map(|entry| -> Result<InstallEntry> {
                        let entry = entry.entry;
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
                        entry.new_entry(FilesPolicy::Replace, &install_dir, &sourcepath)
                    })
                    .collect::<Result<Vec<InstallEntry>>>()
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
                &sourcepath,
            )?);
            // use-pkg-name doesn't make any sense for user-config, so skip it
            results.extend(get_files(
                self.user_config
                    .into_iter()
                    .map(|entry| Entry {
                        entry: entry.entry.entry,
                    })
                    .collect(),
                &pkg_docs.join("user-config/"),
                "user-config",
                FilesPolicy::Replace,
                &sourcepath,
            )?);
        } else {
            results.extend(get_files(
                self.user_config
                    .into_iter()
                    .map(|entry| Entry {
                        entry: entry.entry.entry,
                    })
                    .collect(),
                &dirs.sysconfdir,
                "user-config",
                FilesPolicy::NoReplace,
                &sourcepath,
            )?);
        }

        results.extend(get_files(
            self.desktop_files,
            &dirs.datarootdir.join("applications/"),
            "desktop-files",
            FilesPolicy::Replace,
            &sourcepath,
        )?);

        if system_install {
            results.extend(get_files(
                self.appstream_metadata,
                &dirs.datarootdir.join("metainfo/"),
                "appstream-metadata",
                FilesPolicy::Replace,
                &sourcepath,
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
                .map(|(entry, completionsdir)| -> Result<InstallEntry> {
                    let entry = entry.entry;
                    entry.new_entry(
                        FilesPolicy::Replace,
                        &dirs.datarootdir.join(completionsdir),
                        &sourcepath,
                    )
                })
                .collect::<Result<Vec<InstallEntry>>>()
                .context("error while iterating completion files")?,
        );

        if let Some(pam_modulesdir) = &dirs.pam_modulesdir {
            results.extend(
                self.pam_modules
                    .into_iter()
                    .map(|entry| {
                        let Entry {
                            entry:
                                InnerEntry {
                                    source,
                                    destination,
                                    templating,
                                },
                        } = entry;

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
                        InnerEntry {
                            source,
                            destination,
                            templating,
                        }
                        .new_entry(
                            FilesPolicy::Replace,
                            pam_modulesdir,
                            &sourcepath,
                        )
                    })
                    .collect::<Result<Vec<InstallEntry>>>()
                    .context("error while iterating pam-modules")?,
            );
        }

        if system_install {
            results.extend(get_files(
                self.systemd_units,
                &dirs.systemd_unitsdir.join("system/"),
                "systemd-units",
                FilesPolicy::Replace,
                &sourcepath,
            )?);
        }
        results.extend(get_files(
            self.systemd_user_units,
            &dirs.systemd_unitsdir.join("user/"),
            "systemd-user-units",
            FilesPolicy::Replace,
            &sourcepath,
        )?);

        results.extend(
            self.icons
                .into_iter()
                .map(|icon| -> Icon { icon.icon })
                .filter(|icon| system_install || !icon.pixmaps)
                .map(|icon| -> Result<InstallEntry> {
                    InnerEntry {
                        source: icon.source.clone(),
                        destination: Some(icon.get_destination().with_context(|| {
                            format!(
                                "unable to generate destination for icon {:?}",
                                icon.source.clone()
                            )
                        })?),
                        templating: false,
                    }
                    .new_entry(
                        FilesPolicy::Replace,
                        &dirs.datarootdir,
                        &sourcepath,
                    )
                })
                .collect::<Result<Vec<InstallEntry>>>()
                .context("error while iterating icons")?,
        );

        if system_install {
            results.extend(
                self.terminfo
                    .into_iter()
                    .map(|entry| -> Result<InstallEntry> {
                        let entry = entry.entry;
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
                        let install_dir = dirs.datarootdir.join("terminfo").join(initial);
                        entry.new_entry(FilesPolicy::Replace, &install_dir, &sourcepath)
                    })
                    .collect::<Result<Vec<InstallEntry>>>()
                    .context("error while iterating terminfo files")?,
            );
        }

        results.extend(get_files_dataentry(
            self.licenses,
            &dirs.datarootdir.join("licenses/"),
            "licenses",
            &package_name,
            FilesPolicy::Replace,
            &sourcepath,
        )?);

        if system_install {
            results.extend(get_files(
                self.pkg_config,
                &dirs.libdir.join("pkgconfig/"),
                "pkg-config",
                FilesPolicy::Replace,
                &sourcepath,
            )?);
        }

        Ok(results)
    }

    fn check_entries(
        &self,
        version: &RinstallVersion,
    ) -> Result<()> {
        let version: Version = version.into();
        macro_rules! check_version_expr {
            ( $name:literal, $type:expr, $req:literal ) => {
                let requires = VersionReq::parse($req).unwrap();
                ensure!(
                    $type.is_empty() || requires.matches(&version),
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

        if self.pkg_type == Type::Custom && VersionReq::parse(">=0.2.0").unwrap().matches(&version)
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
