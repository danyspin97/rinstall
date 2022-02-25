use std::{collections::HashMap, fs, path::Path};

use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use semver::Version;
use serde::Deserialize;

use crate::Package;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallSpec {
    #[serde(rename(deserialize = "rinstall"))]
    pub version: Version,
    #[serde(rename(deserialize = "pkgs"))]
    pub packages: HashMap<String, Package>,
}

impl InstallSpec {
    pub fn new_from_path(package_dir: &Path) -> Result<Self> {
        // Try root/install.yml and root/.package/install.yml files
        let install_spec = {
            let install_spec = package_dir.join("install.yml");
            if install_spec.exists() {
                install_spec
            } else {
                let install_spec = package_dir.join(".package").join("install.yml");
                if install_spec.exists() {
                    install_spec
                } else {
                    bail!("unable to find 'install.yml' file");
                }
            }
        };
        Ok(serde_yaml::from_str(
            &fs::read_to_string(&install_spec)
                .with_context(|| format!("unable to read file {:?}", install_spec))?,
        )?)
    }

    pub fn packages(
        self,
        selected: &[String],
    ) -> Vec<Package> {
        self.packages
            .into_iter()
            .filter(|(name, _)| selected.is_empty() || selected.iter().any(|pkg| pkg == name))
            .map(|(name, package)| {
                let mut package = package;
                package.name = Some(name);
                package
            })
            .collect::<Vec<Package>>()
    }
}
