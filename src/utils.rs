use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use color_eyre::{eyre::Context, Result};

#[macro_export]
macro_rules! path_to_str {
    ($path:expr) => {
        $path
            .to_str()
            .with_context(|| format!("unable to convert {:?} to string", $path))?
    };
}

pub fn append_destdir(
    destination: &Path,
    destdir: Option<&str>,
) -> PathBuf {
    destdir.map_or(destination.to_owned(), |destdir| {
        // join does not work when the argument (not the self) is an absolute path
        PathBuf::from({
            let mut s = destdir.to_string();
            s.push_str(destination.as_os_str().to_str().unwrap());
            s
        })
    })
}

pub fn write_to_file(
    destination: &Path,
    contents: &str,
) -> Result<()> {
    BufWriter::new(
        File::create(&destination)
            .with_context(|| format!("unable to create file {:?}", destination))?,
    )
    .write(contents.as_bytes())
    .with_context(|| format!("unable to write to file {:?}", destination))?;

    Ok(())
}
