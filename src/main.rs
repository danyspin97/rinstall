mod config;
mod dirs;
mod install_entry;
mod install_target;
mod package;
mod project;

use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Clap;
use color_eyre::eyre::{bail, ensure, Context, ContextCompat, Result};
use walkdir::WalkDir;
use xdg::BaseDirectories;

use config::Config;
use dirs::Dirs;
use install_target::InstallTarget;
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
    let program: Package = serde_yaml::from_str(
        &fs::read_to_string(&install_spec)
            .with_context(|| format!("unable to read file {:?}", install_spec))?,
    )?;

    let project_type = program.project_type.clone();

    program
        .targets(
            &dirs,
            Project::new_from_type(project_type, package_dir.clone())?,
        )?
        .iter()
        .try_for_each(|install| -> Result<()> {
            let InstallTarget {
                source,
                destination,
                templating,
            } = install;
            // handle destdir
            let destination = destdir.as_ref().map_or(destination.to_owned(), |destdir| {
                // join does not work when the argument (not the self) is an absolute path
                PathBuf::from({
                    let mut s = destdir.clone();
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
                    source.strip_prefix(&package_dir).unwrap_or(source),
                    destination
                );
                if !dry_run {
                    fs::create_dir_all(&destination.parent().unwrap()).with_context(|| {
                        format!("unable to create directory {:?}", destination.parent())
                    })?;
                    if *templating {
                        let contents = fs::read_to_string(source)
                            .with_context(|| format!("unable to read file {:?}", source))?;
                        BufWriter::new(
                            File::create(&destination).with_context(|| {
                                format!("unable to create file {:?}", destination)
                            })?,
                        )
                        .write(
                            apply_templating_to(contents, &dirs)
                                .with_context(|| {
                                    format!("unable to apply templating to {:?}", source)
                                })?
                                .as_bytes(),
                        )
                        .with_context(|| format!("unable to write to file {:?}", destination))?;
                    } else {
                        fs::copy(source, &destination).with_context(|| {
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
                        println!("Installing {:?} to {:?}", source, destination);
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
        })?;

    Ok(())
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

    Ok(s)
}
