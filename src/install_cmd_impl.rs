use std::fs;

use camino::Utf8Path;
use clap::Args;
use color_eyre::{
    eyre::{bail, ensure, Context, ContextCompat},
    Result,
};
use colored::Colorize;
use log::{info, warn};
use walkdir::WalkDir;

use crate::{
    dirs::Dirs,
    dirs_config_impl::DirsConfig,
    install_spec::InstallSpec,
    install_target::InstallTarget,
    package_info::PackageInfo,
    project::Project,
    templating::Templating,
    utils::{append_destdir, write_to_file},
    Uninstall,
};

include!("install_cmd.rs");

static PROJECTDIR_NEEDLE: &str = "$PROJECTDIR";

impl InstallCmd {
    pub fn system(&self) -> bool {
        self.system || self.packaging
    }
    pub fn skip_pkg_info(&self) -> bool {
        self.skip_pkg_info || self.packaging
    }
    pub fn run(self) -> Result<()> {
        let dirs_config = DirsConfig::load(self.config.as_deref(), self.system(), &self.dirs)?;
        let dirs = Dirs::new(dirs_config, self.system()).context("unable to create dirs")?;
        let install_spec =
            InstallSpec::new_from_path(Utf8Path::from_path(&self.package_dir).unwrap())?;

        // Check if the projectdir is a release tarball instead of the
        // directory containing the source code
        let is_release_tarball = self.package_dir.join(".tarball").exists();
        let version = install_spec.version.clone();

        let packages = install_spec.packages(&self.packages);
        for package in packages {
            let mut pkg_info = PackageInfo::new(package.name.as_ref().unwrap(), &dirs);
            let pkg_info_path = append_destdir(&pkg_info.path, self.destdir.as_deref());
            let pkg_already_installed = pkg_info_path.exists();
            info!(
                "{} {} {}",
                ">>>".magenta(),
                "Package".bright_black(),
                pkg_info.pkg_name.italic().blue()
            );
            if pkg_already_installed && !self.update {
                ensure!(
                    !self.accept_changes,
                    "cannot install {} because it has already been installed",
                    pkg_info.pkg_name
                );

                warn!(
                    "package {} is already installed",
                    pkg_info.pkg_name.blue().italic(),
                )
            }

            if pkg_already_installed && self.update {
                let uninstall = Uninstall {
                    config: None,
                    accept_changes: self.accept_changes,
                    force: self.force,
                    system: self.system(),
                    prefix: None,
                    localstatedir: Some(dirs.localstatedir.as_str().to_owned()),
                    packages: vec![pkg_info.pkg_name.clone()],
                };

                uninstall.run()?;
            }

            let project_type = package.project_type.clone();
            let project = Project::new_from_type(
                project_type,
                Utf8Path::from_path(&self.package_dir).unwrap(),
                is_release_tarball,
                self.rust_debug_target,
                self.rust_target_triple.as_deref(),
            )?;

            let targets = package.targets(&dirs, &version, self.system())?;

            for target in targets {
                self.install_target(
                    &target,
                    &dirs,
                    &mut pkg_info,
                    pkg_already_installed,
                    &project,
                )?;
            }

            if !self.skip_pkg_info() {
                if self.accept_changes {
                    info!(
                        "Installing {} -> {}",
                        "pkginfo".purple().bold(),
                        pkg_info_path.as_str().cyan().bold()
                    );
                    pkg_info.install(self.destdir.as_deref())?;
                } else {
                    info!(
                        "Would install {} -> {}",
                        "pkginfo".purple().bold(),
                        pkg_info.path.as_str().cyan().bold()
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
        project: &Project,
    ) -> Result<()> {
        let InstallTarget {
            source,
            destination,
            templating,
            replace,
        } = &install_target;
        let destination = append_destdir(destination, self.destdir.as_deref());

        // The source is using the needle to force it to be in the projectdir
        let source = if let Ok(source) = source.strip_prefix(PROJECTDIR_NEEDLE) {
            source.to_path_buf()
        } else if let Some(outputdir) = &project.outputdir {
            // In this case we are checking if the source exists inside output_dir
            // If it does we use it
            let outputdir_source = outputdir.join(source);
            if outputdir_source.exists() {
                outputdir_source
            } else {
                source.clone()
            }
        } else {
            // Otherwise we use project_dir
            source.clone()
        };

        ensure!(source.exists(), "{:?} does not exist", source);

        if source.is_file() {
            let destination = if destination.as_str().ends_with('/') {
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
                    &source,
                    &destination,
                    pkg_already_installed,
                    *replace,
                )?
            {
                return Ok(());
            }
            info!(
                "{} {} -> {}",
                if self.accept_changes {
                    "Installing"
                } else {
                    "Would install"
                },
                source
                    .strip_prefix(self.package_dir.as_path())
                    .unwrap_or(&source)
                    .as_str()
                    .purple()
                    .bold(),
                destination.as_str().cyan().bold()
            );
            if self.accept_changes {
                fs::create_dir_all(&destination.parent().unwrap()).with_context(|| {
                    format!("unable to create directory {:?}", destination.parent())
                })?;
                if *templating {
                    let mut templating = Templating::new(&source)?;
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

                    let full_path = Utf8Path::from_path(entry.path()).unwrap();
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
                    info!(
                        "{} {} -> {}",
                        if self.accept_changes {
                            "Installing"
                        } else {
                            "Would install"
                        },
                        source
                            .strip_prefix(&self.package_dir)
                            .unwrap_or(&source)
                            .as_str()
                            .purple()
                            .bold(),
                        destination.as_str().cyan().bold()
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
        source: &Utf8Path,
        destination: &Utf8Path,
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
                    warn!(
                        "file {} already exists, add {} to overwrite it",
                        destination.as_str().yellow().bold(),
                        "--force".bright_black().italic(),
                    );
                }
            } else if !self.accept_changes {
                warn!(
                    "file {} already exists, it would be overwritten",
                    destination.as_str().yellow().bold()
                );
            } else {
                warn!("file {} already exists, overwriting it", destination);
            }
        }
        if destination.exists() && !replace {
            if self.update_config {
                if self.accept_changes {
                    warn!("config {} is being overwritten", destination);
                } else if !pkg_already_installed {
                    warn!("config {} will be overwritten", destination);
                }
            } else {
                info!(
                    "{} config {} -> {}",
                    if self.accept_changes {
                        "Skipping"
                    } else {
                        "Would skip"
                    },
                    source
                        .strip_prefix(&self.package_dir)
                        .unwrap_or(source)
                        .as_str()
                        .purple()
                        .bold(),
                    destination.as_str().cyan().bold()
                );
                // Skip installation
                return Ok(true);
            }
        }

        Ok(false)
    }
}
