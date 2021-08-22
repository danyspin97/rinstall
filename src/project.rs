use std::{path::PathBuf, process::Command};

use color_eyre::eyre::{Context, Result};

// Contains data about the project that will be installed
// It doesn't refer to the system and the actual installation directories
// It is only relevant for the source part in InstallEntry
pub struct Project {
    pub outputdir: PathBuf,
    pub projectdir: PathBuf,
}

use crate::package::Type;

impl Project {
    pub fn new_from_type(
        project_type: Type,
        projectdir: PathBuf,
    ) -> Result<Self> {
        Ok(match project_type {
            Type::Rust => Self {
                outputdir: PathBuf::from(
                    json::parse(&String::from_utf8_lossy(
                        &Command::new("cargo")
                            .arg("metadata")
                            .output()
                            .context("unable to run `cargo metadata`")?
                            .stdout,
                    ))
                    .context("unable to parse JSON from `cargo metadata` output")?
                        ["target_directory"]
                        .to_string(),
                )
                .join("release"),
                projectdir,
            },
            Type::Custom => Self {
                outputdir: projectdir.clone(),
                projectdir,
            },
        })
    }
}
