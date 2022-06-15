use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{
    eyre::{ensure, ContextCompat},
    Result,
};

use crate::install_entry::InstallEntry;

pub struct InstallTarget {
    pub source: Utf8PathBuf,
    pub destination: Utf8PathBuf,
    pub templating: bool,
    pub replace: bool,
}

#[derive(Clone, Copy)]
pub enum FilesPolicy {
    Replace,
    NoReplace,
}

impl InstallTarget {
    pub fn new(
        entry: InstallEntry,
        install_dir: &Utf8Path,
        policy: FilesPolicy,
    ) -> Result<Self> {
        let replace = matches!(policy, FilesPolicy::Replace);
        ensure!(
            entry.source.is_relative(),
            "the source file {:?} is not relative",
            entry.source
        );

        let destination =
            if entry.source.is_file() || entry.destination.is_some() {
                install_dir.join(if let Some(destination) = entry.destination {
                    ensure!(
                        destination.is_relative(),
                        "the destination part of a file must be relative"
                    );
                    destination
                } else {
                    Utf8PathBuf::from(entry.source.file_name().with_context(|| {
                        format!("unable to get file name from {:?}", entry.source)
                    })?)
                })
            } else {
                install_dir.to_path_buf()
            };

        Ok(Self {
            source: entry.source,
            destination,
            templating: entry.templating,
            replace,
        })
    }
}
