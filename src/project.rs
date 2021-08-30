use std::{env, os::unix::process::CommandExt, path::PathBuf, process::Command};

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
                // cargo metadata only works when running as the current user that has built
                // the project. Otherwise it will use metadata for the root user and
                // it is almost never what we want
                outputdir: PathBuf::from({
                    if Command::new("cargo")
                        .output()
                        .map_or(false, |output| output.status.success())
                    {
                        json::parse(&String::from_utf8_lossy(
                            &Command::new("cargo")
                                .arg("metadata")
                                .uid(
                                    env::var("SUDO_UID").map_or(unsafe { libc::getuid() }, |uid| {
                                        uid.parse::<u32>().unwrap()
                                    }),
                                )
                                .gid(
                                    env::var("SUDO_GID").map_or(unsafe { libc::getgid() }, |gid| {
                                        gid.parse::<u32>().unwrap()
                                    }),
                                )
                                .output()
                                .context("unable to run `cargo metadata`")?
                                .stdout,
                        ))
                        .context("unable to parse JSON from `cargo metadata` output")?
                            ["target_directory"]
                            .to_string()
                    } else {
                        "target".to_string()
                    }
                })
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
