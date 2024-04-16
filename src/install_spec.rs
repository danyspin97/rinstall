use std::{collections::HashMap, fs};

use camino::Utf8Path;
use color_eyre::{
    eyre::{bail, Context},
    Result,
};
use semver::Version;
use serde::Deserialize;

use crate::Package;

#[derive(Deserialize, Clone)]
pub enum RinstallVersion {
    #[serde(rename = "0.1.0")]
    V0_1_0,
    #[serde(rename = "0.2.0")]
    V0_2_0,
}

impl From<&RinstallVersion> for Version {
    fn from(val: &RinstallVersion) -> Self {
        match val {
            RinstallVersion::V0_1_0 => Version::new(0, 1, 0),
            RinstallVersion::V0_2_0 => Version::new(0, 2, 0),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallSpec {
    #[serde(rename(deserialize = "rinstall"))]
    pub version: RinstallVersion,
    #[serde(rename(deserialize = "pkgs"))]
    pub packages: HashMap<String, Package>,
}

impl InstallSpec {
    pub fn new_from_path(package_dir: &Utf8Path) -> Result<Self> {
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

    pub fn new_from_string(spec_file: String) -> Result<Self> {
        Ok(serde_yaml::from_str(&spec_file)?)
    }

    pub fn packages(
        self,
        selected: &[String],
    ) -> Vec<Package> {
        // Only return packages that are selected by the user
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
