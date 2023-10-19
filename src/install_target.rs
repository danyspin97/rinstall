use camino::Utf8PathBuf;

// SpecFile entries
pub struct InstallEntry {
    // The source as written in the spec file
    pub source: Utf8PathBuf,
    // The full path to the source (either file or directory)
    pub full_source: Utf8PathBuf,
    pub destination: Utf8PathBuf,
    // Do we apply templating here?
    pub templating: bool,
    // Do this file replace the contents by default?
    // i.e. in config it's not replaceable
    pub replace: bool,
}

#[derive(Clone, Copy)]
pub enum FilesPolicy {
    Replace,
    NoReplace,
}
