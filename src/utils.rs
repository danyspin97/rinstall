use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
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
    contents: &[u8],
) -> Result<()> {
    BufWriter::new(
        File::create(destination)
            .with_context(|| format!("unable to create file {:?}", destination))?,
    )
    .write(contents)
    .with_context(|| format!("unable to write to file {:?}", destination))?;

    Ok(())
}

pub fn read_file(
    full_file_path: &Utf8Path,
    source: &Utf8Path,
) -> Result<Vec<u8>, color_eyre::Report> {
    let mut buf = Vec::new();
    let file = File::open(full_file_path)
        .with_context(|| format!("unable to read file path {full_file_path}"))?;
    let mut reader = BufReader::new(file);
    reader
        .read_to_end(&mut buf)
        .with_context(|| format!("unable to read file {:?}", source))?;
    Ok(buf)
}
