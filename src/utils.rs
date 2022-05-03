use std::{
    fs::File,
    io::{BufWriter, Write},
};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::Context, Result};

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
