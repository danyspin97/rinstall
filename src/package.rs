use color_eyre::eyre::{Context, Result};
use serde::Deserialize;

use std::path::{Path, PathBuf};

use crate::install_entry::InstallEntry;
use crate::install_target::InstallTarget;
use crate::Dirs;

#[derive(Deserialize)]
pub enum Type {
    #[serde(rename(deserialize = "custom"))]
    Custom,
    #[serde(rename(deserialize = "rust"))]
    Rust,
}

#[derive(Deserialize)]
enum Completion {
    #[serde(rename(deserialize = "bash"))]
    Bash(PathBuf),
    #[serde(rename(deserialize = "fish"))]
    Fish(PathBuf),
    #[serde(rename(deserialize = "zsh"))]
    Zsh(PathBuf),
}

#[derive(Deserialize)]
pub struct Package {
    name: String,
    #[serde(rename(deserialize = "type"))]
    program_type: Type,
    #[serde(default)]
    exe: Vec<InstallEntry>,
    #[serde(default)]
    libs: Vec<InstallEntry>,
    #[serde(default)]
    man: Vec<InstallEntry>,
    #[serde(default)]
    data: Vec<InstallEntry>,
    #[serde(default)]
    docs: Vec<InstallEntry>,
    #[serde(default)]
    config: Vec<InstallEntry>,
    #[serde(default, rename(deserialize = "desktop-files"))]
    desktop_files: Vec<InstallEntry>,
    #[serde(default)]
    appdata: Vec<InstallEntry>,
    #[serde(default)]
    completions: Vec<Completion>,
}

impl Package {
    pub fn targets(
        self,
        dirs: Dirs,
    ) -> Result<Vec<InstallTarget>> {
        let mut results = Vec::new();

        let bin_local_root = match self.program_type {
            Type::Rust => Some(Path::new("target/release")),
            Type::Custom => None,
        };

        macro_rules! install_files {
            ( $files:tt, $install_dir:expr, $parent_dir:expr, $name:literal ) => {
                self.$files
                    .into_iter()
                    .map(|entry| -> Result<InstallTarget> {
                        InstallTarget::new(entry, $install_dir, $parent_dir)
                    })
                    .collect::<Result<Vec<InstallTarget>>>()
                    .with_context(|| format!("error while iterating {} files", $name))?
            };
        }

        results.extend(install_files!(exe, &dirs.bindir, bin_local_root, "exe"));
        results.extend(install_files!(libs, &dirs.libdir, None, "libs"));
        results.extend(install_files!(data, &dirs.datadir, None, "data"));
        results.extend(install_files!(config, &dirs.sysconfdir, None, "config"));
        if let Some(mandir) = &dirs.mandir {
            results.extend(install_files!(man, mandir, None, "man"));
        }

        let package_dir = self.name.to_owned();
        if let Some(docdir) = &dirs.docdir {
            results.extend(install_files!(
                docs,
                &docdir.join(Path::new(&package_dir)),
                None,
                "docs"
            ));
        }

        results.extend(install_files!(
            desktop_files,
            &dirs.datarootdir.join("applications"),
            None,
            "desktop"
        ));

        results.extend(install_files!(
            appdata,
            &dirs.datarootdir.join("appdata"),
            None,
            "appdata"
        ));

        results.extend(
            self.completions
                .into_iter()
                .map(|completion| -> Result<InstallTarget> {
                    let (entry, parent_dir) = match completion {
                        Completion::Bash(path) => (
                            InstallEntry {
                                source: path,
                                destination: None,
                            },
                            "bash-completion/completions",
                        ),
                        Completion::Fish(path) => (
                            InstallEntry {
                                source: path,
                                destination: None,
                            },
                            "fish/vendor_completions.d",
                        ),
                        Completion::Zsh(path) => (
                            InstallEntry {
                                source: path,
                                destination: None,
                            },
                            "zsh/site-functions",
                        ),
                    };
                    InstallTarget::new(entry, &dirs.datarootdir.join(parent_dir), None)
                })
                .collect::<Result<Vec<InstallTarget>>>()
                .context("error while iterating completion files")?,
        );

        Ok(results)
    }
}
