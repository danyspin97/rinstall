mod config;
mod dirs;
mod install;
mod package;

use std::{fs, path::PathBuf};

use clap::Clap;
use color_eyre::eyre::{ensure, Context, Result};
use walkdir::WalkDir;
use xdg::BaseDirectories;

use config::Config;
use dirs::Dirs;
use install::Install;
use package::Package;

fn main() -> Result<()> {
    color_eyre::install()?;
    let opts = Config::parse();
    let dry_run = opts.dry_run;
    let root_install = !opts.user;
    let uid = unsafe { libc::getuid() };
    ensure!(
        dry_run || !root_install || uid == 0,
        "need either root privileges or --user flag"
    );

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

    let program: Package = serde_yaml::from_str(
        &fs::read_to_string("install.yml")
            .with_context(|| format!("unable to read file {:?}", config_file))?,
    )?;

    program
        .paths(dirs)?
        .iter()
        .try_for_each(|install| -> Result<()> {
            let Install {
                source,
                destination,
            } = install;
            let destination = destination.as_ref().unwrap();

            if source.is_file() {
                println!("Installing {:?} to {:?}", source, destination);
                if !dry_run {
                    fs::create_dir_all(destination.parent().unwrap()).with_context(|| {
                        format!("unable to create directory {:?}", destination.parent())
                    })?;
                    fs::copy(source, destination).with_context(|| {
                        format!("unable to copy file {:?} to {:?}", source, destination)
                    })?;
                }
            } else if source.is_dir() {
                WalkDir::new(source)
                    .into_iter()
                    .filter_entry(|entry| entry.path().is_file())
                    .try_for_each(|entry| -> Result<()> {
                        let entry = entry?;
                        let full_path = entry.path();
                        let relative_path = full_path.strip_prefix(source).with_context(|| {
                            format!("unable to strip prefix {:?} from {:?}", source, full_path)
                        })?;
                        println!("Installing {:?} to {:?}", source, destination);
                        if dry_run {
                            return Ok(());
                        }

                        fs::create_dir_all(destination.parent().unwrap()).with_context(|| {
                            format!("unable to create directory {:?}", destination.parent())
                        })?;
                        fs::copy(full_path, destination.join(relative_path)).with_context(
                            || {
                                format!(
                                    "unable to copy file {:?} to {:?}",
                                    full_path,
                                    destination.join(relative_path)
                                )
                            },
                        )?;

                        Ok(())
                    })?;
            }

            Ok(())
        })?;

    Ok(())
}
