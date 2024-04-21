use std::{
    fs::{self, File},
    io::Write,
};

use camino::Utf8Path;
use clap::Args;
use color_eyre::{
    eyre::{Context, ContextCompat},
    owo_colors::OwoColorize,
    Result,
};
use colored::Colorize;
use flate2::{write::GzEncoder, Compression};
use log::info;
use walkdir::WalkDir;

use crate::{
    dirs::Dirs,
    install_spec::InstallSpec,
    package::{CompletionsToInstall, Type},
    project::{RustDirectories, RUST_DIRECTORIES, RUST_DIRECTORIES_ONCE},
    DirsConfig,
};

include!("tarball_cmd.rs");

impl TarballCmd {
    pub fn run(self) -> Result<()> {
        let dirs_config = DirsConfig::load(None, true, &mut DirsConfig::system_config())?;
        let dirs = Dirs::new(dirs_config, true).context("unable to create dirs")?;
        let install_spec =
            InstallSpec::new_from_path(Utf8Path::from_path(&self.package_dir).unwrap())?;

        let package_dir = Utf8Path::from_path(&self.package_dir)
            .context("Package directory contains invalid UTF-8 character")?;
        // For the filename of the tarball append the suffix .tar.gz
        let filename = format!("{}.tar.gz", self.tarball_name);

        info!("Creating tarball {}", filename.italic().yellow());

        let archive_buf: Vec<u8> = Vec::new();
        let mut archive = tar::Builder::new(archive_buf);
        archive.follow_symlinks(false);

        let directory_name = self.directory_name.as_deref().unwrap_or(&self.tarball_name);

        // Add install.yml
        info!("Adding install.yml");
        archive
            .append_path_with_name(
                package_dir.join("install.yml"),
                format!("{directory_name}/install.yml",),
            )
            .context("Unable to append file install.yml to tarball")?;

        let rinstall_version = install_spec.version.clone();
        let packages = install_spec.packages(&self.packages);

        if packages.iter().any(|p| p.pkg_type == Type::Rust) {
            RUST_DIRECTORIES_ONCE.call_once(|| {
                // We use call_once on std::once::Once, this is safe
                unsafe {
                    RUST_DIRECTORIES = Some(
                        RustDirectories::new(
                                    Some(package_dir.to_path_buf())
                            ,
                            self.rust_debug_target,
                            self.rust_target_triple.as_deref(),
                        )
                        // TODO
                        .unwrap(),
                    );
                }
            });
        }

        for package in packages {
            info!(
                "{} {} {}",
                ">>>".magenta(),
                "Package".bright_black(),
                package.name.as_ref().unwrap().italic().blue()
            );

            let targets =
                package.targets(&dirs, &rinstall_version, true, &CompletionsToInstall::all())?;

            for install_entry in &targets {
                // Print each file/directory added
                info!("Adding {}", install_entry.source.as_str().bold().magenta());
                archive
                    .append_path_with_name(
                        &install_entry.full_source,
                        format!("{directory_name}/{}", install_entry.source),
                    )
                    .with_context(|| {
                        format!("Unable to append path {} to tarball", install_entry.source)
                    })?;
                if install_entry.full_source.is_dir() {
                    WalkDir::new(&install_entry.full_source)
                        .into_iter()
                        .try_for_each(|entry| -> Result<()> {
                            let entry = entry?;
                            if !entry.file_type().is_file() {
                                // skip directories
                                return Ok(());
                            }
                            let full_file_path = Utf8Path::from_path(entry.path()).unwrap();
                            // unwrap here is unsafe
                            let relative_file_path = full_file_path
                                .strip_prefix(&install_entry.full_source)
                                .unwrap();
                            let source = install_entry.source.join(relative_file_path);

                            info!("Adding {}", source.as_str().bold().magenta());
                            archive
                                .append_path_with_name(
                                    full_file_path,
                                    format!("{directory_name}/{source}",),
                                )
                                .with_context(|| {
                                    format!("Unable to append path {source} to tarball")
                                })?;

                            Ok(())
                        })?;
                }
            }
        }

        if Utf8Path::new(&filename).exists() {
            fs::remove_file(&filename)
                .with_context(|| format!("unable to remove file {filename}"))?;
        }
        let file =
            File::create(&filename).with_context(|| format!("Unable to create file {filename}"))?;

        let mut encoder = GzEncoder::new(file, Compression::default());
        //.context("unable to compress tarball")?;

        encoder
            .write_all(&archive.into_inner().context("unable to create tarball")?)
            .context("Unable to write compressed tarball into filesystem")?;

        Ok(())
    }
}
