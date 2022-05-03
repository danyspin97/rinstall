use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    dirs::Dirs,
    utils::{append_destdir, write_to_file},
};

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
        path: &Utf8Path,
        installed_path: &Utf8Path,
        replace: bool,
    ) -> Result<()> {
        let file = InstalledFile {
            path: Utf8Path::new("/").join(installed_path),
            checksum: blake3::hash(
                &fs::read(path).with_context(|| format!("unable to read file {:?}", path))?,
            )
            .to_hex()
            .to_string(),
            replace,
        };

        self.files.push(file);

        Ok(())
    }

    pub fn install(
        &self,
        destdir: Option<&str>,
    ) -> Result<()> {
        let path = append_destdir(&self.path, destdir);
        fs::create_dir_all(path.parent().unwrap())
            .with_context(|| format!("unable to create parent directory for {:?}", path))?;
        write_to_file(
            &path,
            &serde_yaml::to_string(self)
                .with_context(|| format!("unable to serialize installation into {:?}", path))?,
        )
        .with_context(|| format!("unable to write installation info in {:?}", path))?;

        Ok(())
    }
}
