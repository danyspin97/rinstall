use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{
    eyre::{ensure, ContextCompat},
    Result,
};

use crate::{install_entry::InstallEntry, project::Project};

static PROJECTDIR_NEEDLE: &str = "$PROJECTDIR";

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

    pub fn get_source(
        &self,
        project: &Project,
    ) -> Utf8PathBuf {
        // The source is using the needle to force it to be in the projectdir
        if let Ok(source) = self.source.strip_prefix(PROJECTDIR_NEEDLE) {
            source.to_path_buf()
        } else if let Some(outputdir) = &project.outputdir {
            // In this case we are checking if the source exists inside output_dir
            // If it does we use it
            let outputdir_source = outputdir.join(self.source.clone());
            if outputdir_source.exists() {
                outputdir_source
            } else {
                self.source.to_path_buf()
            }
        } else {
            // Otherwise we use project_dir
            self.source.to_path_buf()
        }
    }
}
