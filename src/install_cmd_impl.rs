use std::{fs, path::Path};

use clap::Args;
use color_eyre::{
    eyre::{bail, ensure, Context, ContextCompat},
    Result,
};
use colored::Colorize;
use walkdir::WalkDir;

use crate::{
    dirs::Dirs,
    dirs_config_impl::DirsConfig,
    install_spec::InstallSpec,
    install_target::InstallTarget,
    package_info::PackageInfo,
    path_to_str,
    project::Project,
    templating::Templating,
    utils::{append_destdir, write_to_file},
};

include!("install_cmd.rs");

impl InstallCmd {
    pub fn run(self) -> Result<()> {
        let dirs_config = DirsConfig::load(self.config.as_deref(), self.system, &self.dirs)?;
        let dirs = Dirs::new(dirs_config, self.system).context("unable to create dirs")?;
        let install_spec = InstallSpec::new_from_path(&self.package_dir)?;

        // Check if the projectdir is a release tarball instead of the
        // directory containing the source code
        let is_release_tarball = self.package_dir.join(".tarball").exists();
        let version = install_spec.version.clone();

        let packages = install_spec.packages(&self.packages);
        for package in packages {
            let mut pkg_info = PackageInfo::new(package.name.as_ref().unwrap(), &dirs);
            let pkg_info_path = append_destdir(&pkg_info.path, self.destdir.as_deref());
            let pkg_already_installed = pkg_info_path.exists();
            println!(
                "{} {} {}",
                ">>>".magenta(),
                "Package".bright_black(),
                pkg_info.pkg_name.italic().blue()
            );
            ensure!(
                !self.accept_changes || !pkg_already_installed,
                "cannot install {} because it has already been installed",
                pkg_info.pkg_name
            );

            let project_type = package.project_type.clone();
            let project = Project::new_from_type(
                project_type,
                &self.package_dir,
                is_release_tarball,
                self.rust_debug_target,
            )?;

            let targets = package.targets(&dirs, &project, &version, self.system)?;

            for target in targets {
                self.install_target(&target, &dirs, &mut pkg_info, pkg_already_installed)?;
            }

            if !self.skip_pkg_info {
                if self.accept_changes {
                    println!(
                        "Installing {} in {}",
                        "pkginfo".yellow().bold(),
                        pkg_info_path
                            .to_str()
                            .with_context(|| format!(
                                "unable to convert {:?} to string",
                                pkg_info_path
                            ))?
                            .cyan()
                            .bold()
                    );
                    pkg_info.install(self.destdir.as_deref())?;
                } else {
                    println!(
                        "Would install {} in {}",
                        "pkginfo".yellow().bold(),
                        pkg_info
                            .path
                            .to_str()
                            .with_context(|| format!(
                                "unable to convert {:?} to string",
                                pkg_info.path
                            ))?
                            .cyan()
                            .bold()
                    );
                }
            }
        }

        Ok(())
    }
    pub fn install_target(
        &self,
        install_target: &InstallTarget,
        dirs: &Dirs,
        pkg_info: &mut PackageInfo,
        pkg_already_installed: bool,
    ) -> Result<()> {
        let InstallTarget {
            source,
            destination,
            templating,
            replace,
        } = &install_target;
        let destination = append_destdir(destination, self.destdir.as_deref());

        ensure!(source.exists(), "{:?} does not exist", source);

        if source.is_file() {
            let destination = if path_to_str!(destination).ends_with('/') {
                destination.join(
                    source
                        .file_name()
                        .with_context(|| format!("unable to get filename for {:?}", source))?,
                )
            } else {
                destination
            };
            // destdir conflicts with force and update-config
            if self.destdir.is_none()
                && self.handle_existing_files(
                    source,
                    &destination,
                    pkg_already_installed,
                    *replace,
                )?
            {
                return Ok(());
            }
            println!(
                "{} {} {} {}",
                if self.accept_changes {
                    "Installing"
                } else {
                    "Would install:"
                },
                path_to_str!(source
                    .strip_prefix(self.package_dir.as_path())
                    .unwrap_or(source))
                .yellow()
                .bold(),
                "->".purple(),
                path_to_str!(destination).cyan().bold()
            );
            if self.accept_changes {
                fs::create_dir_all(&destination.parent().unwrap()).with_context(|| {
                    format!("unable to create directory {:?}", destination.parent())
                })?;
                if *templating {
                    let mut templating = Templating::new(source)?;
                    templating
                        .apply(dirs)
                        .with_context(|| format!("unable to apply templating to {:?}", source))?;
                    write_to_file(&destination, &templating.contents)?;
                } else {
                    fs::copy(&source, &destination).with_context(|| {
                        format!("unable to copy file {:?} to {:?}", source, destination)
                    })?;
                }
                let dest_wo_destdir = &self
                    .destdir
                    .as_ref()
                    .map_or(destination.as_path(), |destdir| {
                        destination.strip_prefix(&destdir).unwrap()
                    });
                pkg_info.add_file(&destination, dest_wo_destdir, *replace)?;
            }
        } else if source.is_dir() {
            WalkDir::new(&source)
                .into_iter()
                .try_for_each(|entry| -> Result<()> {
                    let entry = entry?;
                    if !entry.file_type().is_file() {
                        return Ok(());
                    }

                    let full_path = entry.path();
                    let relative_path = full_path.strip_prefix(&source).with_context(|| {
                        format!("unable to strip prefix {:?} from {:?}", source, full_path)
                    })?;
                    let source = source.join(relative_path);
                    let destination = destination.join(relative_path);
                    // destdir conflicts with force and update-config
                    if self.destdir.is_none()
                        && self.handle_existing_files(
                            &source,
                            &destination,
                            pkg_already_installed,
                            *replace,
                        )?
                    {
                        return Ok(());
                    }
                    println!(
                        "{} {} {} {}",
                        if self.accept_changes {
                            "Installing"
                        } else {
                            "Would install:"
                        },
                        path_to_str!(source.strip_prefix(&self.package_dir).unwrap_or(&source))
                            .yellow()
                            .bold(),
                        "->".purple(),
                        path_to_str!(destination).cyan().bold()
                    );
                    if !self.accept_changes {
                        return Ok(());
                    }
                    fs::create_dir_all(destination.parent().unwrap()).with_context(|| {
                        format!("unable to create directory {:?}", destination.parent())
                    })?;
                    if *templating {
                        let mut templating = Templating::new(&source)?;
                        templating.apply(dirs).with_context(|| {
                            format!("unable to apply templating to {:?}", source)
                        })?;
                        write_to_file(&destination, &templating.contents)?;
                    } else {
                        fs::copy(&source, &destination).with_context(|| {
                            format!("unable to copy file {:?} to {:?}", source, destination)
                        })?;
                    }

                    Ok(())
                })?;
        } else {
            bail!("{:?} is neither a file nor a directory", source);
        }

        Ok(())
    }

    // return true if the file should be skipped
    fn handle_existing_files(
        &self,
        source: &Path,
        destination: &Path,
        pkg_already_installed: bool,
        replace: bool,
    ) -> Result<bool> {
        if destination.exists() && replace {
            if !self.force {
                if self.accept_changes {
                    bail!(
                        "file {:?} already exists, add --force to overwrite it",
                        destination
                    );
                } else if !pkg_already_installed {
                    eprintln!(
                        "{} file {} already exists, add {} to overwrite it",
                        "WARNING:".red().italic(),
                        path_to_str!(destination).yellow().bold(),
                        "--force".bright_black().italic(),
                    );
                }
            } else if !self.accept_changes {
                eprintln!(
                    "{} file {} already exists, it would be overwritten",
                    "WARNING:".red().italic(),
                    path_to_str!(destination).yellow().bold()
                );
            } else {
                eprintln!(
                    "{} file {} already exists, overwriting it",
                    "WARNING:".red().italic(),
                    path_to_str!(destination)
                );
            }
        }
        if destination.exists() && !replace {
            if self.update_config {
                if self.accept_changes {
                    eprintln!(
                        "{} config {} is being overwritten",
                        "WARNING:".red().italic(),
                        path_to_str!(destination)
                    );
                } else if !pkg_already_installed {
                    eprintln!(
                        "{} config {} will be overwritten",
                        "WARNING:".red().italic(),
                        path_to_str!(destination)
                    );
                }
            } else {
                println!(
                    "{} config {} {} {}",
                    if self.accept_changes {
                        "Skipping"
                    } else {
                        "Would skip"
                    },
                    path_to_str!(source.strip_prefix(&self.package_dir).unwrap_or(source))
                        .yellow()
                        .bold(),
                    "->".purple(),
                    path_to_str!(destination).cyan().bold()
                );
                // Skip installation
                return Ok(true);
            }
        }

        Ok(false)
    }
}
