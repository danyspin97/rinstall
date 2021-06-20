use std::env;

use color_eyre::eyre::{ensure, Context, ContextCompat, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct Install {
    #[serde(rename(deserialize = "src"))]
    pub source: PathBuf,
    #[serde(rename(deserialize = "dst"))]
    pub destination: Option<PathBuf>,
}

impl Install {
    pub fn sanitize(
        self,
        install_dir: &Path,
        parent_dir: Option<&Path>,
    ) -> Result<Self> {
        let parent_dir = parent_dir
            .map(PathBuf::from)
            .unwrap_or(env::current_dir().context("unable to get current directory")?);
        let destination =
            Some(
                install_dir.join(if let Some(destination) = self.destination {
                    ensure!(
                        destination.is_relative(),
                        "the destination part of a file must be relative"
                    );
                    destination
                } else {
                    PathBuf::from(self.source.file_name().with_context(|| {
                        format!("unable to get file name from {:?}", self.source)
                    })?)
                }),
            );
        ensure!(
            self.source.is_relative(),
            "the source file {:?} is not relative",
            self.source
        );
        let source = parent_dir.join(self.source);
        ensure!(source.exists(), "source file does not exists");

        Ok(Install {
            source,
            destination,
        })
    }
}
