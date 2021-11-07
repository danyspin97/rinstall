use std::{fs, path::Path};

use clap::Parser;
use color_eyre::eyre::{Context, Result};
use serde::Deserialize;

use crate::package_info::PackageInfo;

#[derive(Parser, Deserialize, Clone)]
pub struct Uninstall {
    pkg_name: String,
    #[serde(skip_deserializing)]
    #[clap(
        short = 'y',
        long = "yes",
        about = "Accept the changes and perform the uninstallation"
    )]
    accept_changes: bool,
    #[clap(short = 'f', long = "force", about = "Force the uninstallation")]
    force: bool,
}

impl Uninstall {
    pub fn run(
        self,
        localstatedir: &Path,
        dry_run: bool,
    ) -> Result<()> {
        // Both rinstall and uninstall subcmd have -y argument
        // if this flag has been enabled at least one time, then accept the changes
        // into the system. I.e. it's a dry run only when both flags are disabled
        let dry_run = dry_run && !self.accept_changes;
        let pkg_info = &localstatedir
            .join("rinstall")
            .join(format!("{}.pkg", &self.pkg_name));
        let pkg_info: PackageInfo = serde_yaml::from_str(
            &fs::read_to_string(&pkg_info)
                .with_context(|| format!("unable to read file {:?}", &pkg_info))?,
        )?;

        for file in &pkg_info.files {
            let modified = file.has_been_modified()?;
            if dry_run {
                if file.replace && modified {
                    eprintln!(
                        "WARNING: file {:?} has been modified but it will be uninstalled anyway",
                        file.path
                    );
                } else if !file.replace && modified {
                    eprintln!(
                        "WARNING: file {:?} has been modified but it won't be removed with using --force",
                        file.path
                    );
                } else {
                    println!("Would remove {:?}", file.path);
                }
            } else if !file.replace && modified && !self.force {
                println!("Keeping file {:?}", &file.path);
            } else if modified && (file.replace || self.force) {
                eprintln!(
                    "WARNING: modified file {:?} has been uninstalled",
                    &file.path
                );
                fs::remove_file(&file.path)
                    .with_context(|| format!("unable to remove file {:?}", file.path))?;
            } else {
                println!("Removing {:?}", &file.path);
                fs::remove_file(&file.path)
                    .with_context(|| format!("unable to remove file {:?}", file.path))?;
            }
        }

        if dry_run {
            println!("Would remove {:?}", pkg_info.path);
        } else {
            println!("Removing {:?}", &pkg_info.path);
            fs::remove_file(&pkg_info.path)
                .with_context(|| format!("unable to remove file {:?}", &pkg_info.path))?;
        }

        Ok(())
    }
}
