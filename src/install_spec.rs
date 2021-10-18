use std::collections::HashMap;

use semver::Version;
use serde::Deserialize;

use crate::Package;

#[derive(Deserialize)]
pub struct InstallSpec {
    #[serde(rename(deserialize = "rinstall"))]
    pub version: Version,
    #[serde(rename(deserialize = "pkgs"))]
    pub packages: HashMap<String, Package>,
}
