use color_eyre::eyre::{Context, Result};
use serde::Deserialize;

use std::path::Path;

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
enum Completion {
    #[serde(rename(deserialize = "bash"), deserialize_with = "string_or_struct")]
    Bash(InstallEntry),
    #[serde(rename(deserialize = "fish"), deserialize_with = "string_or_struct")]
    Fish(InstallEntry),
    #[serde(rename(deserialize = "zsh"), deserialize_with = "string_or_struct")]
    Zsh(InstallEntry),
}

#[derive(Deserialize)]
pub struct Package {
    name: String,
    #[serde(rename(deserialize = "type"))]
    pub project_type: Type,
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
        project: Project,
    ) -> Result<Vec<InstallTarget>> {
        let mut results = Vec::new();

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

        results.extend(install_files!(exe, &dirs.bindir, &project.outputdir, "exe"));
        results.extend(install_files!(
            libs,
            &dirs.libdir,
            &project.projectdir,
            "libs"
        ));
        results.extend(install_files!(
            data,
            &dirs.datadir,
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

        let package_doc_dir = self.name.to_owned();
        if let Some(docdir) = &dirs.docdir {
            results.extend(install_files!(
                docs,
                &docdir.join(Path::new(&package_doc_dir)),
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
            appdata,
            &dirs.datarootdir.join("appdata"),
            &project.projectdir,
            "appdata"
        ));

        results.extend(
            self.completions
                .into_iter()
                .map(|completion| -> Result<InstallTarget> {
                    let (entry, parent_dir) = match completion {
                        Completion::Bash(entry) => (entry, "bash-completion/completions"),
                        Completion::Fish(entry) => (entry, "fish/vendor_completions.d"),
                        Completion::Zsh(entry) => (entry, "zsh/site-functions"),
                    };
                    InstallTarget::new(
                        entry,
                        &dirs.datarootdir.join(parent_dir),
                        &project.projectdir,
                    )
                })
                .collect::<Result<Vec<InstallTarget>>>()
                .context("error while iterating completion files")?,
        );

        Ok(results)
    }
}
