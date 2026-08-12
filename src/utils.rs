use std::{fs::OpenOptions, io::Write};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::Context, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_destdir_prepends_prefix() {
        let path = Utf8Path::new("/usr/local/bin/foo");
        let result = append_destdir(path, Some("/tmp/pkg"));
        assert_eq!(result.as_str(), "/tmp/pkg/usr/local/bin/foo");
    }

    #[test]
    fn append_destdir_none_is_identity() {
        let path = Utf8Path::new("/usr/local/bin/foo");
        let result = append_destdir(path, None);
        assert_eq!(result.as_str(), "/usr/local/bin/foo");
    }

    #[test]
    fn write_to_file_writes_all_bytes() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = Utf8Path::from_path(tmp.path()).unwrap();
        let content = b"Hello, world!\nSecond line.\n";
        write_to_file(path, content).unwrap();
        assert_eq!(std::fs::read(tmp.path()).unwrap(), content);
    }

    #[test]
    fn write_to_file_truncates_on_overwrite() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"old long content that should be gone").unwrap();
        let path = Utf8Path::from_path(tmp.path()).unwrap();
        write_to_file(path, b"new").unwrap();
        assert_eq!(std::fs::read(tmp.path()).unwrap(), b"new");
    }
}

pub fn append_destdir(
    destination: &Utf8Path,
    destdir: Option<&str>,
) -> Utf8PathBuf {
    destdir.map_or(destination.to_owned(), |destdir| {
        // join does not work when the argument (not the self) is an absolute path
        Utf8PathBuf::from({
            let mut s = destdir.to_string();
            s.push_str(destination.as_str());
            s
        })
    })
}

pub fn write_to_file(
    destination: &Utf8Path,
    contents: &[u8],
) -> Result<()> {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(destination)
        .with_context(|| format!("unable to open file {:?}", destination))?
        .write_all(contents)
        .with_context(|| format!("unable to write to file {:?}", destination))?;

    Ok(())
}
