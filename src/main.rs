mod config;
mod dirs;
mod icon;
mod install_entry;
mod install_spec;
mod install_target;
mod package;
mod package_info;
mod project;
mod templating;
mod uninstall;
mod utils;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::eyre::{bail, ensure, Context, Result};
use xdg::BaseDirectories;

use config::{Config, SubCommand};
use dirs::Dirs;
use install_spec::InstallSpec;
use package::Package;
use project::Project;

use crate::{package_info::PackageInfo, utils::append_destdir};

fn main() -> Result<()> {
    color_eyre::install()?;
    let opts = Config::parse();
    let subcmd = opts.subcmd.clone();
    let dry_run = !opts.accept_changes;
    let force = opts.force;
    let update_config = opts.update_config;
    let destdir = opts.destdir.clone();
    let uid = unsafe { libc::getuid() };
    let root_install = if !opts.system {
        uid == 0
    } else {
        if uid != 0 {
            ensure!(
                (!opts.system || dry_run || destdir.is_some()),
                "Run rinstall as root to execute a system installation or use destdir"
            );
        }
        opts.system
    };
    let disable_uninstall = opts.disable_uninstall;

    let mut config = if root_install {
        Config::new_default_root()
    } else {
        Config::new_default_user()
    };

    let xdg = BaseDirectories::new().context("unable to initialize XDG Base Directories")?;

    let config_file = if let Some(config_file) = &opts.config {
        let config_file = PathBuf::from(config_file);
        ensure!(config_file.exists(), "config file does not exist");
        config_file
    } else if root_install {
        PathBuf::from("/etc/rinstall.yml")
    } else {
        xdg.place_config_file("rinstall.yml")?
    };
    if config_file.exists() {
        let config_from_file = serde_yaml::from_str(
            &fs::read_to_string(&config_file)
                .with_context(|| format!("unable to read file {:?}", config_file))?,
        )?;
        if root_install {
            config.merge_root_conf(config_from_file);
        } else {
            config.merge_user_conf(config_from_file);
        }
    }

    let package_dir = opts.package_dir.clone().map_or(
        env::current_dir().context("unable to get current directory")?,
        PathBuf::from,
    );
    let pkgs_to_install = opts.packages.clone();

    if root_install {
        config.merge_root_conf(opts);
        config.replace_root_placeholders();
    } else {
        config.merge_user_conf(opts);
        config
            .replace_user_placeholders(&xdg)
            .context("unable to sanitize user directories")?;
    }

    let dirs = Dirs::new(config).context("unable to create dirs")?;

    if let Some(subcmd) = subcmd {
        match subcmd {
            SubCommand::Uninstall(uninstall) => {
                uninstall.run(Path::new(&dirs.localstatedir), dry_run)?;
                return Ok(());
            }
        }
    }

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
            pkgs_to_install.is_empty() || pkgs_to_install.iter().any(|pkg| pkg == name)
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

    for package in packages {
        let pkg_name = package.name.clone().unwrap();
        println!(">>> Package {}", pkg_name);

        let project_type = package.project_type.clone();

        let targets = package.targets(
            &dirs,
            Project::new_from_type(project_type, package_dir.clone(), is_release_tarball)?,
            &install_spec.version,
            root_install,
        )?;

        let mut pkg_info = PackageInfo::new(&pkg_name, &dirs);
        let pkg_info_path = append_destdir(&pkg_info.path, &destdir.as_deref());
        ensure!(
            !pkg_info_path.exists(),
            "cannot install {} because it has already been installed",
            pkg_info.pkg_name
        );

        for target in targets {
            target.install(
                destdir.as_deref(),
                dry_run,
                force,
                update_config,
                &package_dir,
                &dirs,
                &mut pkg_info,
            )?;
        }

        if !disable_uninstall {
            if dry_run {
                println!("Would install installation data in {:?}", pkg_info.path);
            } else {
                println!("Installing installation data in {:?}", pkg_info_path);
                pkg_info.install(destdir.as_deref())?;
            }
        }
    }

    Ok(())
}
