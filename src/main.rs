mod config_impl;
mod dirs;
mod icon;
mod install_entry;
mod install_spec;
mod install_target;
mod package;
mod package_info;
mod project;
mod templating;
mod uninstall_impl;
mod utils;

#[macro_use]
extern crate lazy_static;

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::{
    eyre::{bail, ensure, Context, ContextCompat},
    Result,
};
use colored::*;
use xdg::BaseDirectories;

pub use config_impl::{Config, SubCommand};
use dirs::Dirs;
use install_spec::InstallSpec;
use package::Package;
use project::Project;
pub use uninstall_impl::Uninstall;

use crate::{package_info::PackageInfo, utils::append_destdir};

lazy_static! {
    static ref XDG: BaseDirectories = BaseDirectories::new()
        .context("unable to initialize XDG Base Directories")
        .unwrap();
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let opts = Config::parse();
    let uid = unsafe { libc::getuid() };
    let dry_run = !opts.accept_changes;
    let system_install = if uid != 0 {
        ensure!(
            (!opts.system || dry_run || opts.destdir.is_some()),
            "Run rinstall as root to execute a system installation or use destdir"
        );
        opts.system
    } else {
        true
    };

    let mut config = if system_install {
        Config::new_default_root()
    } else {
        Config::new_default_user()
    };

    let config_file = if let Some(config_file) = &opts.config {
        let config_file = PathBuf::from(config_file);
        ensure!(config_file.exists(), "config file does not exist");
        config_file
    } else if system_install {
        PathBuf::from("/etc/rinstall.yml")
    } else {
        XDG.place_config_file("rinstall.yml")?
    };
    if config_file.exists() {
        let config_from_file = serde_yaml::from_str(
            &fs::read_to_string(&config_file)
                .with_context(|| format!("unable to read file {:?}", config_file))?,
        )?;
        if system_install {
            config.merge_root_conf(config_from_file);
        } else {
            config.merge_user_conf(config_from_file);
        }
    }

    if system_install {
        config.merge_root_conf(opts);
        config.replace_root_placeholders();
    } else {
        config.merge_user_conf(opts);
        config
            .replace_user_placeholders(&XDG)
            .context("unable to sanitize user directories")?;
    }

    let dirs = Dirs::new(&config).context("unable to create dirs")?;

    if let Some(subcmd) = &config.subcmd {
        match subcmd {
            SubCommand::Uninstall(uninstall) => {
                uninstall.run(Path::new(&dirs.localstatedir), dry_run)?;
                return Ok(());
            }
            SubCommand::GenerateRpmFiles => {
                ensure!(
                    system_install,
                    "rpm-files can only be used for system wide installations"
                );
            }
        }
    }

    let package_dir = PathBuf::from(&config.package_dir.as_ref().unwrap());

    // Try root/install.yml and root/.package/install.yml files
    let install_spec = {
        let install_spec = package_dir.join("install.yml");
        if install_spec.exists() {
            install_spec
        } else {
            let install_spec = package_dir.join(".package").join("install.yml");
            if install_spec.exists() {
                install_spec
            } else {
                bail!("unable to find 'install.yml' file");
            }
        }
    };
    let install_spec: InstallSpec = serde_yaml::from_str(
        &fs::read_to_string(&install_spec)
            .with_context(|| format!("unable to read file {:?}", install_spec))?,
    )?;

    let packages = install_spec
        .packages
        .into_iter()
        .filter(|(name, _)| {
            config.packages.is_empty() || config.packages.iter().any(|pkg| pkg == name)
        })
        .map(|(name, package)| {
            let mut package = package;
            package.name = Some(name);
            package
        })
        .collect::<Vec<Package>>();

    // Check if the projectdir is a release tarball instead of the
    // directory containing the source code
    let is_release_tarball = package_dir.join(".tarball").exists();

    let mut rpm_files = Vec::new();

    for package in packages {
        let pkg_name = package.name.clone().unwrap();

        let project_type = package.project_type.clone();

        let targets = package.targets(
            &dirs,
            Project::new_from_type(
                project_type,
                package_dir.to_path_buf(),
                is_release_tarball,
                config.rust_debug_target,
            )?,
            &install_spec.version,
            system_install,
        )?;
        if let Some(subcmd) = &config.subcmd {
            match subcmd {
                SubCommand::Uninstall(_) => {}
                SubCommand::GenerateRpmFiles => {
                    for target in targets {
                        rpm_files.append(&mut target.generate_rpm_files()?);
                    }
                    continue;
                }
            }
        }

        println!(
            "{} {} {}",
            ">>>".magenta(),
            "Package".bright_black(),
            pkg_name.italic().blue()
        );

        let mut pkg_info = PackageInfo::new(&pkg_name, &dirs);
        let pkg_info_path = append_destdir(&pkg_info.path, &config.destdir.as_deref());
        let pkg_already_installed = pkg_info_path.exists();
        ensure!(
            dry_run || !pkg_already_installed,
            "cannot install {} because it has already been installed",
            pkg_info.pkg_name
        );

        for target in targets {
            target.install(&config, &dirs, &mut pkg_info, pkg_already_installed)?;
        }

        if !config.disable_uninstall {
            if dry_run {
                println!(
                    "Would install installation data in {}",
                    pkg_info
                        .path
                        .to_str()
                        .with_context(|| format!(
                            "unable to convert {:?} to string",
                            pkg_info.path
                        ))?
                        .cyan()
                        .bold()
                );
            } else {
                println!(
                    "Installing installation data in {}",
                    pkg_info_path
                        .to_str()
                        .with_context(|| format!(
                            "unable to convert {:?} to string",
                            pkg_info_path
                        ))?
                        .cyan()
                        .bold()
                );
                pkg_info.install(config.destdir.as_deref())?;
            }
        }
    }

    if let Some(subcmd) = &config.subcmd {
        match subcmd {
            SubCommand::Uninstall(_) => {}
            SubCommand::GenerateRpmFiles => {
                let mut res = String::new();
                let mut owned_dir = HashSet::new();
                owned_dir.insert(dirs.bindir);
                owned_dir.insert(dirs.datadir);
                owned_dir.insert(dirs.datarootdir.clone());
                // unwrapping here is safe, because GenerateRpmFiles requires --system
                owned_dir.insert(dirs.docdir.unwrap());
                owned_dir.insert(dirs.includedir.unwrap());
                owned_dir.insert(dirs.libdir);
                owned_dir.insert(dirs.libexecdir);
                owned_dir.insert(dirs.localstatedir);
                let mandir = dirs.mandir.unwrap();
                owned_dir.insert(mandir.clone());
                for i in 1..8 {
                    owned_dir.insert(mandir.join(format!("man{i}")));
                }
                owned_dir.insert(dirs.pam_modulesdir.unwrap());
                owned_dir.insert(dirs.sbindir.unwrap());
                owned_dir.insert(dirs.sysconfdir);
                owned_dir.insert(dirs.systemd_unitsdir);
                owned_dir.insert(dirs.datarootdir.join("licenses"));
                owned_dir.insert(dirs.datarootdir.join("applications"));
                owned_dir.insert(dirs.datarootdir.join("zsh").join("site-functions"));
                owned_dir.insert(dirs.datarootdir.join("bash-completion").join("completions"));
                owned_dir.insert(dirs.datarootdir.join("fish").join("vendor_completions.d"));

                for file in rpm_files {
                    let mut parent_dirs = Vec::new();
                    let mut parent = file.parent().unwrap();
                    loop {
                        if owned_dir.contains(parent) {
                            break;
                        }
                        owned_dir.insert(parent.to_path_buf());
                        parent_dirs.push(parent);
                        parent = parent.parent().unwrap();
                    }
                    for dir in parent_dirs.iter().rev() {
                        res.push_str("%dir ");
                        res.push_str(dir.to_str().unwrap());
                        res.push('\n');
                    }
                    res.push_str(file.to_str().unwrap());
                    res.push('\n');
                }

                println!("{}", res);
            }
        }
    }

    Ok(())
}
