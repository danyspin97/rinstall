use camino::{Utf8Path, Utf8PathBuf};

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

impl InstallEntry {
    pub fn destination_for_file(&self) -> Utf8PathBuf {
        if self.destination.as_str().ends_with('/') {
            self.destination.join(self.source.file_name().unwrap())
        } else {
            self.destination.to_owned()
        }
    }

    pub fn destination_for_file_in_directory(
        &self,
        full_path: &Utf8Path,
    ) -> Utf8PathBuf {
        let relative_file_path = full_path.strip_prefix(&self.full_source).unwrap();
        self.destination.join(relative_file_path)
    }
}
