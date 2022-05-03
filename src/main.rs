#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

mod dirs;
mod dirs_config_impl;
mod icon;
mod install_cmd_impl;
mod install_entry;
mod install_spec;
mod install_target;
mod opts_impl;
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

use dirs::Dirs;
pub use dirs_config_impl::DirsConfig;
pub use install_cmd_impl::InstallCmd;
pub use opts_impl::{Opts, SubCommand};
use package::Package;
pub use uninstall_impl::Uninstall;

fn main() -> Result<()> {
    color_eyre::install()?;
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Uninstall(uninstall) => {
            uninstall.run()?;
        }
        SubCommand::Install(install) => install.run()?,
    }

    Ok(())
}
