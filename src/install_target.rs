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
}
