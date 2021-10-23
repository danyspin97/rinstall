mod config;
mod dirs;
mod install_entry;
mod install_spec;
mod install_target;
mod package;
mod project;

use std::{env, fs, path::PathBuf};

use clap::Clap;
use color_eyre::eyre::{bail, ensure, Context, Result};
use xdg::BaseDirectories;

use config::Config;
use dirs::Dirs;
use install_spec::InstallSpec;
use package::Package;
use project::Project;

fn main() -> Result<()> {
    color_eyre::install()?;
    let opts = Config::parse();
    let dry_run = !opts.accept_changes;
    let uid = unsafe { libc::getuid() };
    let root_install = if !opts.system {
        uid == 0
    } else {
        if uid != 0 {
            ensure!(
                (!opts.system || dry_run),
                "Run rinstall as root to execute a system installation"
            );
        }
        opts.system
    };

    let mut config = if root_install {
        Config::new_default_root()
    } else {
        Config::new_default_user()
    };

    let xdg = BaseDirectories::new().context("unable to initialize XDG Base Directories")?;

    let config_file = if let Some(config_file) = &opts.config {
        let config_file = PathBuf::from(config_file);
        ensure!(config_file.exists(), "config file does not exists");
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
    let destdir = opts.destdir.clone();
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

    for package in packages {
        println!(">>> Package {}", package.name.as_ref().unwrap());

        let project_type = package.project_type.clone();

        for target in package.targets(
            &dirs,
            Project::new_from_type(project_type, package_dir.clone())?,
            &install_spec.version,
        )? {
            target.install(destdir.as_deref(), dry_run, &package_dir, &dirs)?;
        }
    }

    Ok(())
}
