mod dirs;
mod dirs_config_impl;
mod icon;
mod install_cmd_impl;
mod install_spec;
mod install_target;
mod opts_impl;
mod package;
mod package_info;
mod project;
mod simple_logger;
mod string_or_struct;
mod tarball_cmd_impl;
mod templating;
mod uninstall_impl;
mod utils;

#[macro_use]
extern crate lazy_static;

use clap::Parser;
use color_eyre::Result;
use log::LevelFilter;

use dirs::Dirs;
pub use dirs_config_impl::DirsConfig;
pub use install_cmd_impl::InstallCmd;
pub use opts_impl::{Opts, SubCommand};
use package::Package;
use simple_logger::SimpleLogger;
pub use tarball_cmd_impl::TarballCmd;
pub use uninstall_impl::Uninstall;

fn main() -> Result<()> {
    color_eyre::install()?;
    let opts = Opts::parse();
    log::set_boxed_logger(Box::new(SimpleLogger { quiet: opts.quiet }))
        .map(|()| log::set_max_level(LevelFilter::Info))?;

    match opts.subcmd {
        SubCommand::Uninstall(uninstall) => {
            uninstall.run()?;
        }
        SubCommand::Install(install) => install.run()?,
        SubCommand::Tarball(tarball) => tarball.run()?,
    }

    Ok(())
}
