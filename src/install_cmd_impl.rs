use std::{
    fs::{self, File},
    io::{BufReader, Cursor, Read},
};

use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;
use color_eyre::{
    eyre::{bail, ensure, Context, ContextCompat},
    Result,
};
use colored::Colorize;
use flate2::bufread::GzDecoder;
use log::{info, warn};
use walkdir::WalkDir;

use crate::{
    dirs::Dirs,
    dirs_config_impl::DirsConfig,
    install_spec::InstallSpec,
    install_target::InstallEntry,
    package::{CompletionsToInstall, Package, Type},
    package_info::PackageInfo,
    project::{RustDirectories, RUST_DIRECTORIES, RUST_DIRECTORIES_ONCE},
    templating::apply_templating,
    utils::{append_destdir, write_to_file},
    Uninstall,
};

include!("install_cmd.rs");

impl InstallCmd {
    // Returns true if we need to use the system directories
    pub fn system_dirs(&self) -> bool {
        // If it is being run as root, or --system / --packaging have been set
        (unsafe { libc::getuid() } == 0) || self.system || self.packaging
    }
    pub fn skip_pkg_info(&self) -> bool {
        self.skip_pkg_info || self.packaging
    }
    pub fn run(mut self) -> Result<()> {
        let dirs_config =
            DirsConfig::load(self.config.as_deref(), self.system_dirs(), &mut self.dirs)?;
        let dirs = Dirs::new(dirs_config, self.system_dirs()).context("unable to create dirs")?;

        let completions = if let Some(completions) = self.completions.as_ref() {
            CompletionsToInstall::parse(completions)?
        } else {
            CompletionsToInstall::all()
        };

        // Disable the experimental tarball feature
        if let Some(tarball) = self.tarball.as_ref() {
            let tarball = Utf8Path::from_path(tarball)
                .with_context(|| format!("{tarball:?} contains invalid UTF-8 characters"))?;
            ensure!(tarball.exists(), "{tarball} does not exists");

            // Decoompress the content of the archive
            let mut decoder = GzDecoder::new(BufReader::new(
                File::open(tarball).with_context(|| format!("unable to open tarball {tarball}"))?,
            ));
            let mut buf = Vec::new();
            decoder
                .read_to_end(&mut buf)
                .with_context(|| format!("unable to decompress tarball {tarball}"))?;

            let cursor = Cursor::new(buf);
            let mut archive = tar::Archive::new(cursor);

            let mut tarball_entries = archive
                .entries()
                .context("unable to create iterator over tarball archive")?;
            let mut entry = tarball_entries
                .next()
                .context("empty tarball archive")?
                .context("invalid tarball archive")?;
            ensure!(
                entry
                    .path()
                    .context("invalid path in tarball archive")?
                    .file_name()
                    .context("invalid path in tarball archive")?
                    .to_string_lossy()
                    == "install.yml",
                "the first file is not rinstall spec file"
            );
            let mut spec_file = String::new();
            // Pass the error up with the ? operator
            entry
                .read_to_string(&mut spec_file)
                .context("unable to read spec file from tarball")?;
            let install_spec = InstallSpec::new_from_string(spec_file)?;
            let version = install_spec.version.clone();

            let packages = install_spec.packages(&self.packages);
            // TODO
            // Initialize project directories (only rust for now)
            if packages.iter().any(|p| p.pkg_type == Type::Rust) {
                RUST_DIRECTORIES_ONCE.call_once(|| {
                    // We use call_once on std::once::Once, this is safe
                    unsafe {
                        RUST_DIRECTORIES = Some(
                            RustDirectories::new(
                            None,
                            self.rust_debug_target,
                            self.rust_target_triple.as_deref(),
                        )
                        // TODO
                        .unwrap(),
                        );
                    }
                });
            }

            for package in packages {
                let mut pkg_installer = PackageInstaller::new(&package, &self, &dirs)
                    .with_context(|| {
                        format!(
                            "failed to create package installer for package {:?}",
                            package.name
                        )
                    })?;
                let install_entries =
                    package.targets(&dirs, &version, self.system_dirs(), &completions)?;

                while let Some(Ok(mut tarball_entry)) = tarball_entries.next() {
                    let entry_path = tarball_entry
                        .path()
                        .context("unable to read path for tarball entry")?;
                    let path = Utf8Path::from_path(&entry_path)
                        .with_context(|| {
                            format!("invalid UTF8 path for tarball entry {:?}", entry_path)
                        })?
                        .to_path_buf();

                    // Is equal to ["rinstall-0.3.0/rinstall", "rinstall-0.3.0", ""]
                    let ancestors = path.ancestors().collect::<Vec<_>>();
                    // Get the second last
                    let prefix = ancestors[ancestors.len() - 2];
                    // and then strip it from the path, as it's just the directory
                    let path = path.strip_prefix(prefix).unwrap();

                    // Skip entries in the tarball that are not inside the rinstall spec file
                    // This is okay because the tarball entries list does not match the target list
                    // i.e. a directory in the spec file will have all the corresponding files in the entries
                    for install_entry in &install_entries {
                        let destination = if install_entry.source == path {
                            Some(install_entry.destination_for_file())
                        } else if let Ok(strip_prefix) = path.strip_prefix(&install_entry.source) {
                            Some(install_entry.destination.join(strip_prefix))
                        } else {
                            None
                        };
                        if let Some(destination) = destination {
                            let installer = |destination| -> Result<()> {
                                // unpack_in returns true if the entry was unpacked
                                // false if the entry contains '..' and thus was not packed
                                match tarball_entry.unpack_in(destination)? {
                                    true => Err(color_eyre::eyre::anyhow!("path contains '..'")),
                                    false => Ok(()),
                                }
                            };
                            pkg_installer.install_target(
                                install_entry,
                                destination,
                                self.accept_changes,
                                installer,
                            )?;
                            continue;
                        }
                    }
                }
            }
        } else {
            let packagedir = Utf8Path::from_path(&self.package_dir).with_context(|| {
                format!("{:?} contains invalid UTF-8 characters", self.package_dir)
            })?;
            let install_spec = InstallSpec::new_from_path(packagedir)?;
            let version = install_spec.version.clone();

            let packages = install_spec.packages(&self.packages);

            if packages.iter().any(|p| p.pkg_type == Type::Rust) {
                RUST_DIRECTORIES_ONCE.call_once(|| {
                    // We use call_once on std::once::Once, this is safe
                    unsafe {
                        RUST_DIRECTORIES = Some(
                            RustDirectories::new(
                            Some(packagedir.to_owned()),
                            self.rust_debug_target,
                            self.rust_target_triple.as_deref(),
                        )
                        // TODO                       
                        .unwrap(),
                        );
                    }
                });
            }

            for package in packages {
                let mut pkg_installer = PackageInstaller::new(&package, &self, &dirs)?;

                let entries = package.targets(&dirs, &version, self.system_dirs(), &completions)?;
                for install_entry in entries {
                    ensure!(
                        install_entry.full_source.exists(),
                        "File {:?} does not exist",
                        install_entry.source
                    );

                    if install_entry.full_source.is_file() {
                        let destination = install_entry.destination_for_file();
                        let installer = |destination| -> Result<()> {
                            std::fs::copy(&install_entry.full_source, &destination)
                                .with_context(|| {
                                    format!(
                                        "unable to copy file {} to {destination}",
                                        install_entry.full_source
                                    )
                                })
                                .map(|_| ())
                        };

                        pkg_installer
                            .install_target(
                                &install_entry,
                                destination,
                                self.accept_changes,
                                installer,
                            )
                            .with_context(|| {
                                format!("failed to install file {:?}", install_entry.full_source)
                            })?;
                    } else if install_entry.full_source.is_dir() {
                        WalkDir::new(&install_entry.full_source)
                            .into_iter()
                            .try_for_each(|entry| -> Result<()> {
                                let entry = entry?;
                                if !entry.file_type().is_file() {
                                    // skip directories
                                    return Ok(());
                                }
                                let full_file_path = Utf8Path::from_path(entry.path()).unwrap();

                                let destination =
                                    install_entry.destination_for_file_in_directory(full_file_path);
                                let installer = |destination| -> Result<()> {
                                    std::fs::copy(&install_entry.full_source, &destination)
                                        .with_context(|| {
                                            format!(
                                                "unable to copy file {} to {destination}",
                                                install_entry.full_source
                                            )
                                        })
                                        .map(|_| ())
                                };

                                pkg_installer
                                    .install_target(
                                        &install_entry,
                                        destination,
                                        self.accept_changes,
                                        installer,
                                    )
                                    .with_context(|| {
                                        format!(
                                            "failed to install file {:?}",
                                            install_entry.full_source
                                        )
                                    })?;

                                Ok(())
                            })?;
                    } else {
                        bail!(
                            "{:?} is neither a file nor a directory",
                            install_entry.source
                        );
                    }
                }

                pkg_installer.install_pkg_info()?;
            }
        }

        Ok(())
    }
}

struct PackageInstaller<'a> {
    check_for_overwrite: bool,
    dirs: &'a Dirs,
    install_opts: &'a InstallCmd,
    pkg_info: PackageInfo,
}

impl<'a> PackageInstaller<'a> {
    pub fn new(
        package: &Package,
        install_opts: &'a InstallCmd,
        dirs: &'a Dirs,
    ) -> Result<Self> {
        let pkg_info = PackageInfo::new(package.name.as_ref().unwrap(), dirs);
        let pkg_info_path = append_destdir(&pkg_info.path, install_opts.destdir.as_deref());
        let pkg_already_installed = pkg_info_path.exists();
        info!(
            "{} {} {}",
            ">>>".magenta(),
            "Package".bright_black(),
            pkg_info.pkg_name.italic().blue()
        );
        // if this package is already installed and the user did not specify --update
        if pkg_already_installed && !install_opts.update {
            // check that we are running in dry-run mode
            ensure!(
                !install_opts.accept_changes,
                "cannot install {} because it has already been installed",
                pkg_info.pkg_name
            );

            warn!(
                "package {} is already installed",
                pkg_info.pkg_name.blue().italic(),
            )
        }
        // if this package is already installed, we need to uninstall it first
        if pkg_already_installed && install_opts.update {
            let uninstall = Uninstall {
                config: None,
                accept_changes: install_opts.accept_changes,
                force: install_opts.force,
                system: install_opts.system_dirs(),
                prefix: None,
                localstatedir: Some(dirs.localstatedir.as_str().to_owned()),
                packages: vec![pkg_info.pkg_name.clone()],
            };

            uninstall.run()?;
        }

        Ok(Self {
            check_for_overwrite: pkg_already_installed,
            dirs,
            install_opts,
            pkg_info,
        })
    }

    fn install_target<F>(
        &mut self,
        target: &InstallEntry,
        destination: Utf8PathBuf,
        accept_changes: bool,
        mut installer: F,
    ) -> Result<()>
    where
        F: FnMut(Utf8PathBuf) -> Result<()>,
    {
        // if we are not installing to a custom destdir and the file already exists
        if self.install_opts.destdir.is_none() && self.handle_existing_file(target, &destination)? {
            return Ok(());
        }

        let destination_destdir =
            append_destdir(&destination, self.install_opts.destdir.as_deref());

        if accept_changes {
            info!(
                "Installing {} -> {}",
                target.source.as_str().purple().bold(),
                destination_destdir.as_str().cyan().bold()
            );

            fs::create_dir_all(destination_destdir.parent().unwrap()).with_context(|| {
                format!(
                    "unable to create directory {:?}",
                    destination_destdir.parent()
                )
            })?;

            installer(destination_destdir.clone())?;

            self.apply_templating(target)?;

            self.pkg_info
                .add_file(&destination, &destination_destdir, target.replace)?;
        } else {
            info!(
                "Would install {} -> {}",
                target.source.as_str().purple().bold(),
                destination.as_str().cyan().bold()
            );
        }

        Ok(())
    }

    fn apply_templating(
        &self,
        entry: &InstallEntry,
    ) -> Result<()> {
        if entry.templating {
            let mut file = File::open(&entry.destination)
                .with_context(|| format!("unable to open file {:?}", entry.destination))?;
            let mut file_contents = Vec::new();
            file.read_to_end(&mut file_contents)
                .with_context(|| format!("unable to read file {:?}", entry.destination))?;
            let contents = apply_templating(&file_contents, self.dirs)
                .with_context(|| format!("unable to apply templating to {:?}", entry.source))?;
            write_to_file(&entry.destination, contents.as_bytes())?;
        }

        Ok(())
    }

    /// Returns true if the file already exists and we should skip installation
    fn handle_existing_file(
        &self,
        target: &InstallEntry,
        destination: &Utf8Path,
    ) -> Result<bool> {
        let accept_changes = self.install_opts.accept_changes;
        if destination.exists() && target.replace {
            if !self.install_opts.force {
                if accept_changes {
                    bail!(
                        "file {:?} already exists, add --force to overwrite it",
                        destination
                    );
                } else if !self.check_for_overwrite {
                    warn!(
                        "file {} already exists, add {} to overwrite it",
                        destination.as_str().yellow().bold(),
                        "--force".bright_black().italic(),
                    );
                }
            } else if !accept_changes {
                warn!(
                    "file {} already exists, it would be overwritten",
                    destination.as_str().yellow().bold()
                );
            } else {
                warn!("file {} already exists, overwriting it", destination);
            }
        }
        if destination.exists() && !target.replace {
            if self.install_opts.update_config {
                if accept_changes {
                    warn!("config {} is being overwritten", destination);
                } else if !self.check_for_overwrite {
                    warn!("config {} will be overwritten", target.destination);
                }
            } else {
                info!(
                    "{} config {} -> {}",
                    if accept_changes {
                        "Skipping"
                    } else {
                        "Would skip"
                    }, // TODO
                    target.source.as_str().purple().bold(),
                    destination.as_str().cyan().bold()
                );
                // Skip installation
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn install_pkg_info(&self) -> Result<()> {
        if !self.install_opts.skip_pkg_info() {
            if self.install_opts.accept_changes {
                info!(
                    "Installing {} -> {}",
                    "pkginfo".purple().bold(),
                    self.pkg_info.path.as_str().cyan().bold()
                );
                self.pkg_info.install()?;
            } else {
                info!(
                    "Would install {} -> {}",
                    "pkginfo".purple().bold(),
                    self.pkg_info.path.as_str().cyan().bold()
                );
            }
        }

        Ok(())
    }
}
