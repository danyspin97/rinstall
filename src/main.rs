mod config_impl;
mod dirs;
mod dirs_config_impl;
mod generate_rpm_files_impl;
mod icon;
mod install_cmd_impl;
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
    env, fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::{
    eyre::{ensure, Context},
    Result,
};
use xdg::BaseDirectories;

pub use config_impl::{Config, SubCommand};
use dirs::Dirs;
pub use dirs_config_impl::DirsConfig;
pub use generate_rpm_files_impl::GenerateRpmFiles;
pub use install_cmd_impl::InstallCmd;
use install_spec::InstallSpec;
use package::Package;
pub use uninstall_impl::Uninstall;

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
    let system_install = if uid == 0 { true } else { opts.system };

    let mut dirs_config = if system_install {
        DirsConfig::system_config()
    } else {
        DirsConfig::user_config()
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
            dirs_config.merge_root_conf(config_from_file);
        } else {
            dirs_config.merge_user_conf(config_from_file);
        }
    }

    if system_install {
        dirs_config.merge_root_conf(opts.dirs.clone());
        dirs_config.replace_root_placeholders();
    } else {
        dirs_config.merge_user_conf(opts.dirs.clone());
        dirs_config
            .replace_user_placeholders(&XDG)
            .context("unable to sanitize user directories")?;
    }

    let dirs = Dirs::new(dirs_config, system_install).context("unable to create dirs")?;

    let current_dir = env::current_dir()
        .context("unable to get current directory")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let package_dir = PathBuf::from(&opts.package_dir.as_ref().unwrap_or(&current_dir));
    let install_spec = InstallSpec::new_from_path(&package_dir)?;

    match opts.subcmd {
        SubCommand::Uninstall(uninstall) => {
            uninstall.run(Path::new(&dirs.localstatedir), dry_run)?;
        }
        SubCommand::GenerateRpmFiles(generate_rpm) => {
            generate_rpm.run(&package_dir, install_spec, dirs)?;
        }
        SubCommand::Install(install) => install.run(&package_dir, install_spec, &dirs)?,
    }

    Ok(())
}
