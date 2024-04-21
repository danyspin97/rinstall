use std::{
    fs::{self, File},
    io::Write,
};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

use crate::dirs::Dirs;

#[derive(Serialize, Deserialize)]
pub struct InstalledFile {
    pub path: Utf8PathBuf,
    pub checksum: String,
    pub replace: bool,
}

impl InstalledFile {
    pub fn has_been_modified(&self) -> Result<bool> {
        Ok(self.checksum
            != blake3::hash(
                &fs::read(&self.path)
                    .with_context(|| format!("unable to read file {:?}", self.path))?,
            )
            .to_hex()
            .to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct PackageInfo {
    #[serde(skip)]
    pub pkg_name: String,
    pub path: Utf8PathBuf,
    pub files: Vec<InstalledFile>,
}

impl PackageInfo {
    pub fn new(
        pkg_name: &str,
        dirs: &Dirs,
    ) -> Self {
        Self {
            pkg_name: pkg_name.to_string(),
            path: dirs
                .localstatedir
                .join("rinstall")
                .join(format!("{}.pkg", &pkg_name)),
            files: Vec::new(),
        }
    }

    pub fn add_file(
        &mut self,
        target_path: &Utf8Path,
        path: &Utf8Path,
        replace: bool,
    ) -> Result<()> {
        let mut hasher = blake3::Hasher::new();
        hasher.update_reader(File::open(path)?)?;
        let file = InstalledFile {
            path: Utf8Path::new("/").join(target_path),
            checksum: hasher.finalize().to_string(),
            replace,
        };

        self.files.push(file);

        Ok(())
    }

    pub fn install(&self) -> Result<()> {
        fs::create_dir_all(self.path.parent().unwrap())
            .with_context(|| format!("unable to create parent directory for {:?}", self.path))?;
        File::create(&self.path)
            .with_context(|| format!("unable to open file {:?}", self.path))?
            .write(
                serde_yaml::to_string(self)
                    .with_context(|| {
                        format!("unable to serialize installation into {:?}", self.path)
                    })?
                    .as_bytes(),
            )
            .with_context(|| format!("unable to write installation info in {:?}", self.path))?;

        Ok(())
    }
}
