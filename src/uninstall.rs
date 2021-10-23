use std::{fs, path::Path};

use clap::Parser;
use color_eyre::eyre::{Context, Result};
use serde::Deserialize;

#[derive(Parser, Deserialize, Clone)]
pub struct Uninstall {
    pkg_name: String,
    #[serde(skip_deserializing)]
    #[clap(
        short = 'y',
        long = "yes",
        about = "Accept the changes and perform the installation"
    )]
    accept_changes: bool,
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
        let files: Vec<String> = serde_yaml::from_str(
            &fs::read_to_string(&pkg_info)
                .with_context(|| format!("unable to read file {:?}", &pkg_info))?,
        )?;

        for file in files {
            if dry_run {
                println!("Would remove {}", file);
            } else {
                println!("Removing {}", &file);
                fs::remove_file(&file)
                    .with_context(|| format!("unable to remove file {}", file))?;
            }
        }

        Ok(())
    }
}
