use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use color_eyre::{
    eyre::{ensure, Context},
    Result,
};
use colored::Colorize;

use crate::{dirs::Dirs, dirs_config_impl::DirsConfig, package_info::PackageInfo};

include!("uninstall.rs");

impl Uninstall {
    pub fn run(&self) -> Result<()> {
        let mut opt_dirs = if self.system {
            DirsConfig::system_config()
        } else {
            DirsConfig::user_config()
        };
        opt_dirs.prefix = self.prefix.clone();
        opt_dirs.localstatedir = self.localstatedir.clone();
        let dirs_config = DirsConfig::load(self.config.as_deref(), self.system, &opt_dirs)?;
        let dirs = Dirs::new(dirs_config, self.system).context("unable to create dirs")?;
        let dry_run = !self.accept_changes;
        for pkg in &self.packages {
            let pkg_info = if Utf8Path::new(&pkg).is_absolute() {
                Utf8PathBuf::from(pkg)
            } else {
                dirs.localstatedir
                    .join("rinstall")
                    .join(format!("{}.pkg", &pkg))
            };
            ensure!(pkg_info.exists(), "package {} is not installed", &pkg);
            let pkg_info: PackageInfo = serde_yaml::from_str(
                &fs::read_to_string(&pkg_info)
                    .with_context(|| format!("unable to read file {:?}", &pkg_info))?,
            )?;

            for file in &pkg_info.files {
                let modified = file.has_been_modified()?;
                if dry_run {
                    if file.replace && modified {
                        eprintln!(
                            "{} file {} has been modified but it will be uninstalled anyway",
                            "WARNING:".red().italic(),
                            file.path.as_str().yellow().bold()
                        );
                    } else if !file.replace && modified {
                        eprintln!(
                        "{} file {} has been modified but it won't be removed, add {} to remove it",
                        "WARNING:".red().italic(),
                        file.path.as_str().yellow().bold(),
                        "--force".bright_black().italic(),
                    );
                    } else {
                        println!("Would remove {}", file.path.as_str().yellow().bold());
                    }
                } else if !file.replace && modified && !self.force {
                    println!("Keeping file {}", file.path.as_str().yellow().bold());
                } else if modified && (file.replace || self.force) {
                    eprintln!(
                        "{} modified file {} has been uninstalled",
                        "WARNING:".red().italic(),
                        file.path.as_str().yellow().bold(),
                    );
                    fs::remove_file(&file.path)
                        .with_context(|| format!("unable to remove file {:?}", file.path))?;
                } else {
                    println!("Removing {}", file.path.as_str().yellow().bold());
                    fs::remove_file(&file.path)
                        .with_context(|| format!("unable to remove file {:?}", file.path))?;
                }
            }

            if dry_run {
                println!("Would remove {}", pkg_info.path.as_str().yellow().bold());
            } else {
                println!("Removing {}", pkg_info.path.as_str().yellow().bold());
                fs::remove_file(&pkg_info.path)
                    .with_context(|| format!("unable to remove file {:?}", &pkg_info.path))?;
            }
        }

        Ok(())
    }
}
