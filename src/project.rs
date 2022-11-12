use std::{env, os::unix::process::CommandExt, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::Context, Result};

// Contains data about the project that will be installed
// It doesn't refer to the system and the actual installation directories
// It is only relevant for the source part in InstallEntry
pub struct Project {
    pub outputdir: Option<Utf8PathBuf>,
    pub projectdir: Utf8PathBuf,
}

use crate::package::Type;

impl Project {
    pub fn new_from_type(
        project_type: Type,
        projectdir: &Utf8Path,
        is_release_tarball: bool,
        rust_debug_target: bool,
        rust_target_triple: Option<&str>,
    ) -> Result<Self> {
        Ok(Self {
            outputdir: if is_release_tarball {
                None
            } else {
                match project_type {
                    Type::Rust => Some(get_target_dir_for_rust(projectdir, rust_debug_target, rust_target_triple)?),
                    Type::Default | Type::Custom => None,
                }
            },
            projectdir: projectdir.to_path_buf(),
        })
    }
}

fn get_target_dir_for_rust(
    projectdir: &Utf8Path,
    rust_debug_target: bool,
    rust_target_triple: Option<&str>,
) -> Result<Utf8PathBuf> {
    Ok(
        // if target directory does not exist, try reading the "target_directory"
        // from cargo metadata
        if !projectdir.join("target").exists()
            && Command::new("cargo")
                .current_dir(projectdir)
                .output()
                .map_or(false, |output| output.status.success())
        {
            Utf8PathBuf::from(
                json::parse(&String::from_utf8_lossy(
                    &Command::new("cargo")
                        .arg("metadata")
                        .current_dir(projectdir)
                        .uid(
                            // cargo metadata only works when running as the current user that has built
                            // the project. Otherwise it will use metadata for the root user and
                            // it is almost never what we want
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
                .context("unable to parse JSON from `cargo metadata` output")?["target_directory"]
                    .to_string(),
            )
        } else {
            projectdir.join("target")
        }
        .join(if let Some(triple) = rust_target_triple {
            triple
        } else {
            ""
        })
        .join(if rust_debug_target {
            "debug"
        } else {
            "release"
        }),
    )
}
