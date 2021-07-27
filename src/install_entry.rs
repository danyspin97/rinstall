use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct InstallEntry {
    #[serde(rename(deserialize = "src"))]
    pub source: PathBuf,
    #[serde(rename(deserialize = "dst"))]
    pub destination: Option<PathBuf>,
}
