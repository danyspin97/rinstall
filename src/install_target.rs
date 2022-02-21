use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{
    eyre::{bail, ensure, Context, ContextCompat},
    Result,
};
use colored::Colorize;
use walkdir::WalkDir;

use crate::utils::write_to_file;
use crate::Dirs;
use crate::{install_entry::InstallEntry, package_info::PackageInfo};
use crate::{path_to_str, templating::Templating};
use crate::{utils::append_destdir, Config};

pub struct InstallTarget {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub templating: bool,
    pub replace: bool,
}

impl InstallTarget {
    pub fn new(
        entry: InstallEntry,
        install_dir: &Path,
        parent_dir: &Path,
        is_repleaceable: bool,
    ) -> Result<Self> {
        let full_source = parent_dir.join(&entry.source);
        let destination =
            if full_source.is_file() || entry.destination.is_some() {
                install_dir.join(if let Some(destination) = entry.destination {
                    ensure!(
                        destination.is_relative(),
                        "the destination part of a file must be relative"
                    );
                    destination
                } else {
                    PathBuf::from(entry.source.file_name().with_context(|| {
                        format!("unable to get file name from {:?}", entry.source)
                    })?)
                })
            } else {
                install_dir.to_path_buf()
            };
        ensure!(
            entry.source.is_relative(),
            "the source file {:?} is not relative",
            entry.source
        );

        Ok(Self {
            source: full_source,
            destination,
            templating: entry.templating,
            replace: is_repleaceable,
        })
    }

    pub fn generate_rpm_files(&self) -> Result<Vec<PathBuf>> {
        ensure!(self.source.exists(), "{:?} does not exist", self.source);
        Ok(if self.source.is_file() {
            vec![self.destination.clone()]
        } else {
            let mut res = Vec::new();
            WalkDir::new(&self.source)
                .into_iter()
                .try_for_each(|entry| -> Result<()> {
                    let entry = entry?;
                    if !entry.file_type().is_file() {
                        return Ok(());
                    }

                    let full_path = entry.path();
                    let relative_path =
                        full_path.strip_prefix(&self.source).with_context(|| {
                            format!(
                                "unable to strip prefix {:?} from {:?}",
                                self.source, full_path
                            )
                        })?;
                    res.push(self.destination.join(relative_path));

                    Ok(())
                })?;
            res
        })
    }

    pub fn install(
        self,
        config: &Config,
        dirs: &Dirs,
        pkg_info: &mut PackageInfo,
        pkg_already_installed: bool,
    ) -> Result<()> {
        let InstallTarget {
            source,
            destination,
            templating,
            replace,
        } = &self;
        let destination = append_destdir(destination, &config.destdir.as_deref());

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
            if config.destdir.is_none()
                && self.handle_existing_files(
                    source,
                    &destination,
                    config,
                    pkg_already_installed,
                )?
            {
                return Ok(());
            }
            println!(
                "{} {} {} {}",
                if !config.accept_changes {
                    "Would install:"
                } else {
                    "Installing"
                },
                path_to_str!(source
                    .strip_prefix(&config.package_dir.as_ref().unwrap())
                    .unwrap_or(source))
                .yellow()
                .bold(),
                "->".purple(),
                path_to_str!(destination).cyan().bold()
            );
            if config.accept_changes {
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
                let dest_wo_destdir = if let Some(destdir) = &config.destdir {
                    destination.strip_prefix(destdir).unwrap()
                } else {
                    &destination
                };
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
                    if config.destdir.is_none()
                        && self.handle_existing_files(
                            &source,
                            &destination,
                            config,
                            pkg_already_installed,
                        )?
                    {
                        return Ok(());
                    }
                    println!(
                        "{} {} {} {}",
                        if !config.accept_changes {
                            "Would install:"
                        } else {
                            "Installing"
                        },
                        path_to_str!(source
                            .strip_prefix(&config.package_dir.as_ref().unwrap())
                            .unwrap_or(&source))
                        .yellow()
                        .bold(),
                        "->".purple(),
                        path_to_str!(destination).cyan().bold()
                    );
                    if !config.accept_changes {
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
        config: &Config,
        pkg_already_installed: bool,
    ) -> Result<bool> {
        if destination.exists() && self.replace {
            if !config.force {
                if !config.accept_changes {
                    if !pkg_already_installed {
                        eprintln!(
                            "{} file {} already exists, add {} to overwrite it",
                            "WARNING:".red().italic(),
                            path_to_str!(destination).yellow().bold(),
                            "--force".bright_black().italic(),
                        );
                    }
                } else {
                    bail!(
                        "file {:?} already exists, add --force to overwrite it",
                        destination
                    );
                }
            } else if !config.accept_changes {
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
        if destination.exists() && !self.replace {
            if config.update_config {
                if !config.accept_changes {
                    if !pkg_already_installed {
                        eprintln!(
                            "{} config {} will be overwritten",
                            "WARNING:".red().italic(),
                            path_to_str!(destination)
                        );
                    }
                } else {
                    eprintln!(
                        "{} config {} is being overwritten",
                        "WARNING:".red().italic(),
                        path_to_str!(destination)
                    );
                }
            } else {
                println!(
                    "{} config {} {} {}",
                    if !config.accept_changes {
                        "Would skip"
                    } else {
                        "Skipping"
                    },
                    path_to_str!(source
                        .strip_prefix(&config.package_dir.as_ref().unwrap())
                        .unwrap_or(source))
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
