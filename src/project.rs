use std::{env, os::unix::process::CommandExt, process::Command, sync::Once};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::Context, Result};

pub static RUST_DIRECTORIES_ONCE: Once = Once::new();
pub static mut RUST_DIRECTORIES: Option<RustDirectories> = None;

// Contains data about the project that will be installed
// It doesn't refer to the system and the actual installation directories
// It is only relevant for the source part in InstallEntry
pub struct RustDirectories {
    pub packagedir: Option<Utf8PathBuf>,
    pub outputdir: Option<Utf8PathBuf>,
}

pub struct DefaultProjectDirectories;

pub trait ProjectDirectories {
    fn sourcepath(
        &'static self,
        source: &Utf8Path,
    ) -> Utf8PathBuf;
}

impl RustDirectories {
    pub fn new(
        package_dir: Option<Utf8PathBuf>,
        rust_debug_target: bool,
        rust_target_triple: Option<&str>,
    ) -> Result<Self> {
        Ok(Self {
            packagedir: package_dir.as_ref().map(|p| p.to_owned()),
            // package_dir is empty when we are extracting from a tarball
            outputdir: if let Some(package_dir) = package_dir {
                Some(Self::target_dir(
                    &package_dir,
                    rust_debug_target,
                    rust_target_triple,
                )?)
            } else {
                None
            },
        })
    }

    fn target_dir(
        package_dir: &Utf8Path,
        rust_debug_target: bool,
        rust_target_triple: Option<&str>,
    ) -> Result<Utf8PathBuf> {
        let env_target_dir = std::env::var("CARGO_TARGET_DIR");
        // if CARGO_TARGET_DIR and target directory do not exist,
        // try reading the "target_directory" from cargo metadata
        let res = if env_target_dir.is_err()
            || !package_dir.join("target").exists()
                // cargo is installed?
                && Command::new("cargo")
                    .current_dir(package_dir)
                    .output()
                    .map_or(false, |output| output.status.success())
        {
            Utf8PathBuf::from(
                json::parse(&String::from_utf8_lossy(
                    &Command::new("cargo")
                        .arg("metadata")
                        .current_dir(package_dir)
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
        } else if let Ok(env_target_dir) = env_target_dir {
            Utf8PathBuf::from(env_target_dir)
        } else {
            package_dir.join("target")
        };

        // Append the target triple and debug/release
        let res = res
            .join(rust_target_triple.unwrap_or_default())
            .join(if rust_debug_target {
                "debug"
            } else {
                "release"
            });

        Ok(res)
    }
}

impl ProjectDirectories for RustDirectories {
    fn sourcepath(
        &'static self,
        source: &Utf8Path,
    ) -> Utf8PathBuf {
        // Packagedir is set when we are installing from a directory
        // It is not set when installing from a tarball
        match (&self.packagedir, &self.outputdir) {
            (Some(packagedir), Some(outputdir)) => {
                // In this case we are checking if the source exists inside output_dir
                // If it does we use it
                let outputdir_source = outputdir.join(source);
                if outputdir_source.exists() {
                    outputdir_source
                } else {
                    // Otherwise we use packagedir
                    packagedir.join(source)
                }
            }
            (None, None) => source.to_owned(),
            // Both are always set or not set
            _ => unreachable!(),
        }
    }
}

impl ProjectDirectories for DefaultProjectDirectories {
    fn sourcepath(
        &'static self,
        source: &Utf8Path,
    ) -> Utf8PathBuf {
        source.to_path_buf()
    }
}
