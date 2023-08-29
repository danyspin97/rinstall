use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use camino::{Utf8Path, Utf8PathBuf};
use clap::Args;
use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use colored::Colorize;
use log::info;
use xz::write::XzEncoder;

use crate::{dirs::Dirs, install_spec::InstallSpec, project::Project, DirsConfig};

include!("tarball_cmd.rs");

impl TarballCmd {
    pub fn run(self) -> Result<()> {
        let dirs_config = DirsConfig::load(None, true, &mut DirsConfig::system_config())?;
        let dirs = Dirs::new(dirs_config, true).context("unable to create dirs")?;
        let install_spec =
            InstallSpec::new_from_path(Utf8Path::from_path(&self.package_dir).unwrap())?;

        let version = install_spec.version.clone();

        let package_dir = Utf8Path::from_path(&self.package_dir)
            .context("Package directory contains invalid UTF-8 character")?;
        let pkg_name = package_dir
            .file_name()
            .context("invalid package directory")?;
        // For the filename of the tarball, use the name of the directory where
        // the project resides as fallback
        let filename = self
            .tarball_name
            .unwrap_or_else(|| format!("{pkg_name}-{version}.tar.xz"));
        info!("Creating tarball {}", filename.italic().yellow());

        let archive_buf: Vec<u8> = Vec::new();
        let mut archive = tar::Builder::new(archive_buf);
        archive.follow_symlinks(false);

        // Add install.yml
        info!("Adding install.yml");
        archive
            .append_path_with_name(
                package_dir.join("install.yml"),
                format!("{pkg_name}-{version}/install.yml",),
            )
            .context("Unable to append file install.yml to tarball")?;

        let packages = install_spec.packages(&self.packages);
        for package in packages {
            info!(
                "{} {} {}",
                ">>>".magenta(),
                "Package".bright_black(),
                package.name.as_ref().unwrap().italic().blue()
            );
            let project = Project::new_from_type(
                package.project_type.clone(),
                Utf8Path::from_path(&self.package_dir).unwrap(),
                false,
                self.rust_debug_target,
                self.rust_target_triple.as_deref(),
            )?;

            let targets = package.targets(&dirs, &version, true)?;

            for target in &targets {
                // Discard the destination part, we'll copy the directory structure as it is
                let source = &target.get_source(&project);
                // Source is an absolute path, we need the path relative to
                // package_dir to put inside the tarball
                // It can be either inside package_dir or project.outputdir, try
                // to remove the prefix starting from the latter
                let relative_path = if let Some(outputdir) = project.outputdir.as_ref() {
                    if let Ok(res) = source.strip_prefix(outputdir) {
                        res
                    } else {
                        if let Ok(stripped_path) = source.strip_prefix(&package_dir) {
                            stripped_path
                        } else {
                            source
                        }
                    }
                } else {
                    if let Ok(stripped_path) = source.strip_prefix(&package_dir) {
                        stripped_path
                    } else {
                        source
                    }
                };
                // Print each file added
                info!(
                    "Adding {}",
                    source
                        .strip_prefix(std::env::current_dir().unwrap())
                        .unwrap_or(source)
                        .as_str()
                        .bold()
                        .magenta()
                );
                archive
                    .append_path_with_name(
                        // usually entries are relative to the package_dir
                        // they are absolute only for files contained in the outputdir folder
                        if source.is_relative() {
                            package_dir.join(source)
                        } else {
                            source.to_path_buf()
                        },
                        format!("{pkg_name}-{version}/{relative_path}",),
                    )
                    .with_context(|| {
                        format!("Unable to append file {} to tarball", target.source)
                    })?;
            }
        }

        let compressor = XzEncoder::new(
            archive
                .into_inner()
                .context("unable to create the uncompressed tarball archive")?,
            9,
        );
        if Utf8Path::new(&filename).exists() {
            fs::remove_file(&filename)
                .with_context(|| format!("unable to remove file {filename}"))?;
        }
        let file =
            File::create(&filename).with_context(|| format!("Unable to create file {filename}"))?;
        BufWriter::new(file)
            .write(&compressor.finish().context("Unable to compress tarball")?)
            .with_context(|| format!("Unable to write compressed tarball into filesystem"))?;

        Ok(())
    }
}
