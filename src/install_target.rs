use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use color_eyre::eyre::{bail, ensure, Context, ContextCompat, Result};
use walkdir::WalkDir;

use crate::install_entry::InstallEntry;
use crate::Dirs;

pub struct InstallTarget {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub templating: bool,
}

impl InstallTarget {
    pub fn new(
        entry: InstallEntry,
        install_dir: &Path,
        parent_dir: &Path,
    ) -> Result<Self> {
        let full_source = parent_dir.join(&entry.source);
        let destination =
            if full_source.is_file() || entry.destination.is_some() {
                install_dir.join(if let Some(destination) = entry.destination {
                    ensure!(
                        destination.is_relative(),
                        "the destination part of a file must be relative"
                    );
                    destination
                } else {
                    PathBuf::from(entry.source.file_name().with_context(|| {
                        format!("unable to get file name from {:?}", entry.source)
                    })?)
                })
            } else {
                install_dir.to_path_buf()
            };
        ensure!(
            entry.source.is_relative(),
            "the source file {:?} is not relative",
            entry.source
        );

        Ok(Self {
            source: full_source,
            destination,
            templating: entry.templating,
        })
    }

    pub fn install(
        self,
        destdir: Option<&str>,
        dry_run: bool,
        package_dir: &Path,
        dirs: &Dirs,
    ) -> Result<()> {
        let InstallTarget {
            source,
            destination,
            templating,
        } = self;
        // handle destdir
        let destination = destdir.map_or(destination.to_owned(), |destdir| {
            // join does not work when the argument (not the self) is an absolute path
            PathBuf::from({
                let mut s = destdir.to_string();
                s.push_str(destination.as_os_str().to_str().unwrap());
                s
            })
        });

        ensure!(source.exists(), "{:?} does not exists", source);

        if source.is_file() {
            let destination = if destination.as_os_str().to_str().unwrap().ends_with('/') {
                destination.join(
                    source
                        .file_name()
                        .with_context(|| format!("unable to get filename for {:?}", source))?,
                )
            } else {
                destination
            };
            println!(
                "{} {:?} -> {:?}",
                if dry_run {
                    "Would install:"
                } else {
                    "Installing"
                },
                source.strip_prefix(&package_dir).unwrap_or(&source),
                destination
            );
            if !dry_run {
                fs::create_dir_all(&destination.parent().unwrap()).with_context(|| {
                    format!("unable to create directory {:?}", destination.parent())
                })?;
                if templating {
                    let contents = fs::read_to_string(&source)
                        .with_context(|| format!("unable to read file {:?}", source))?;
                    BufWriter::new(
                        File::create(&destination)
                            .with_context(|| format!("unable to create file {:?}", destination))?,
                    )
                    .write(
                        apply_templating_to(contents, dirs)
                            .with_context(|| format!("unable to apply templating to {:?}", source))?
                            .as_bytes(),
                    )
                    .with_context(|| format!("unable to write to file {:?}", destination))?;
                } else {
                    fs::copy(&source, &destination).with_context(|| {
                        format!("unable to copy file {:?} to {:?}", source, destination)
                    })?;
                }
            }
        } else if source.is_dir() {
            WalkDir::new(&source)
                .into_iter()
                .try_for_each(|entry| -> Result<()> {
                    let entry = entry?;
                    if !entry.file_type().is_file() {
                        return Ok(());
                    }

                    let full_path = entry.path();
                    let relative_path = full_path.strip_prefix(&source).with_context(|| {
                        format!("unable to strip prefix {:?} from {:?}", source, full_path)
                    })?;
                    let source = source.join(relative_path);
                    let destination = destination.join(relative_path);
                    println!(
                        "{} {:?} -> {:?}",
                        if dry_run {
                            "Would install:"
                        } else {
                            "Installing"
                        },
                        source.strip_prefix(&package_dir).unwrap_or(&source),
                        destination
                    );
                    if dry_run {
                        return Ok(());
                    }
                    fs::create_dir_all(destination.parent().unwrap()).with_context(|| {
                        format!("unable to create directory {:?}", destination.parent())
                    })?;
                    fs::copy(full_path, &destination).with_context(|| {
                        format!("unable to copy file {:?} to {:?}", full_path, destination)
                    })?;

                    Ok(())
                })?;
        } else {
            bail!("{:?} is neither a file nor a directory", source);
        }

        Ok(())
    }
}

fn apply_templating_to(
    s: String,
    dirs: &Dirs,
) -> Result<String> {
    let mut s = s;

    macro_rules! replace_impl {
        ( $dir:expr, $needle:literal ) => {
            s = s.replace(
                $needle,
                $dir.as_os_str()
                    .to_str()
                    .with_context(|| format!("unable to convert {:?} to String", $dir))?,
            );
        };
    }

    macro_rules! replace {
        ( $dir:ident, $needle:literal ) => {
            replace_impl!(&dirs.$dir, $needle);
        };
    }

    macro_rules! replace_when_some {
        ( $dir:ident, $needle:literal ) => {
            if let Some($dir) = &dirs.$dir {
                replace_impl!($dir, $needle);
            } else {
                // TODO: Is this needed?
                ensure!(
                    !s.contains($needle),
                    "tried replacing {} when its value is none",
                    $needle
                );
            }
        };
    }

    replace_when_some!(prefix, "@prefix@");
    replace_when_some!(exec_prefix, "@exec_prefix@");
    replace!(bindir, "@bindir@");
    replace!(libdir, "@libdir@");
    replace!(datarootdir, "@datarootdir@");
    replace!(datadir, "@datadir@");
    replace!(sysconfdir, "@sysconfdir@");
    replace!(localstatedir, "@localstatedir@");
    replace!(runstatedir, "@runstatedir@");
    replace_when_some!(includedir, "@includedir@");
    replace_when_some!(docdir, "@docdir@");
    replace_when_some!(mandir, "@mandir@");
    replace_when_some!(pam_modulesdir, "@pam_moduledirs@");
    replace!(systemd_unitsdir, "@systemd_unitsdir@");

    Ok(s)
}
