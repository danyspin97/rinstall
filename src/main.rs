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

use clap::Parser;
use color_eyre::Result;

pub use config_impl::{Config, SubCommand};
use dirs::Dirs;
pub use dirs_config_impl::DirsConfig;
pub use generate_rpm_files_impl::GenerateRpmFiles;
pub use install_cmd_impl::InstallCmd;
use package::Package;
pub use uninstall_impl::Uninstall;

fn main() -> Result<()> {
    color_eyre::install()?;
    let opts = Config::parse();
    // let uid = unsafe { libc::getuid() };
    // let system_install = if uid == 0 { true } else { opts.system };

    match opts.subcmd {
        SubCommand::Uninstall(uninstall) => {
            uninstall.run()?;
        }
        SubCommand::GenerateRpmFiles(generate_rpm) => {
            generate_rpm.run()?;
        }
        SubCommand::Install(install) => install.run()?,
    }

    Ok(())
}
