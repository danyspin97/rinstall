use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::io::Error;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use serde::Deserialize;

include!("src/dirs_config.rs");
include!("src/install_cmd.rs");
include!("src/uninstall.rs");
include!("src/tarball_cmd.rs");
include!("src/opts.rs");

fn build_shell_completion(outdir: &Path) -> Result<(), Error> {
    let mut opts = Opts::command();
    let shells = Shell::value_variants();

    for shell in shells {
        generate_to(*shell, &mut opts, "rinstall", outdir)?;
    }

    Ok(())
}

fn build_manpages(outdir: &Path) -> Result<(), Error> {
    let opts = Opts::command();

    let file = Path::new(&outdir).join("rinstall.1");
    let mut file = File::create(file)?;

    Man::new(opts).render(&mut file)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=man");

    let outdir = PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .ancestors()
        .nth(3)
        .unwrap()
        .to_path_buf();

    let comp_path = outdir.join("completions");
    let man_path = outdir.join("man");
    std::fs::create_dir_all(&comp_path)?;
    std::fs::create_dir_all(&man_path)?;

    build_shell_completion(&comp_path)?;
    build_manpages(&man_path)?;

    Ok(())
}
