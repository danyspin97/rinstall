use std::{
    env,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};

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
        is_release_tarball: bool,
        rust_debug_target: bool,
    ) -> Result<Self> {
        Ok(Self {
            outputdir: if is_release_tarball {
                projectdir.clone()
            } else {
                match project_type {
                    Type::Rust => get_target_dir_for_rust(&projectdir, rust_debug_target)?,
                    Type::Default | Type::Custom => projectdir.clone(),
                }
            },
            projectdir,
        })
    }
}

fn get_target_dir_for_rust(
    projectdir: &Path,
    rust_debug_target: bool,
) -> Result<PathBuf> {
    Ok(PathBuf::from({
        // if target directory does not exist, try reading the "target_directory"
        // from cargo metadata
        if !projectdir.join("target").exists()
            && Command::new("cargo")
                .current_dir(projectdir)
                .output()
                .map_or(false, |output| output.status.success())
        {
            json::parse(&String::from_utf8_lossy(
                &Command::new("cargo")
                    .arg("metadata")
                    .current_dir(projectdir)
                    .uid(
                        // cargo metadata only works when running as the current user that has built
                        // the project. Otherwise it will use metadata for the root user and
                        // it is almost never what we want
                        env::var("SUDO_UID")
                            .map_or(unsafe { libc::getuid() }, |uid| uid.parse::<u32>().unwrap()),
                    )
                    .gid(
                        env::var("SUDO_GID")
                            .map_or(unsafe { libc::getgid() }, |gid| gid.parse::<u32>().unwrap()),
                    )
                    .output()
                    .context("unable to run `cargo metadata`")?
                    .stdout,
            ))
            .context("unable to parse JSON from `cargo metadata` output")?["target_directory"]
                .to_string()
        } else {
            projectdir
                .join("target")
                .as_os_str()
                .to_str()
                .with_context(|| format!("unable to convert {:?} to string", projectdir))?
                .to_string()
        }
    })
    .join(if rust_debug_target {
        "debug"
    } else {
        "release"
    }))
}
