use std::path::{Path, PathBuf};

use color_eyre::{
    eyre::{ensure, Context, ContextCompat},
    Result,
};
use walkdir::WalkDir;

use crate::install_entry::InstallEntry;

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
}
