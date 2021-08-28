use color_eyre::eyre::{ensure, ContextCompat, Result};
use std::path::{Path, PathBuf};

use crate::install_entry::InstallEntry;

pub struct InstallTarget {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub templating: bool,
}

impl InstallTarget {
    pub fn new(
        entry: InstallEntry,
        install_dir: &Path,
        parent_dir: &Path,
    ) -> Result<Self> {
        let destination = install_dir.join(if let Some(destination) = entry.destination {
            ensure!(
                destination.is_relative(),
                "the destination part of a file must be relative"
            );
            destination
        } else {
            PathBuf::from(
                entry
                    .source
                    .file_name()
                    .with_context(|| format!("unable to get file name from {:?}", entry.source))?,
            )
        });
        ensure!(
            entry.source.is_relative(),
            "the source file {:?} is not relative",
            entry.source
        );
        let source = parent_dir.join(entry.source);

        Ok(Self {
            source,
            destination,
            templating: entry.templating,
        })
    }
}
