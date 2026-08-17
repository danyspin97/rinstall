use camino::Utf8Path;
use clap::Args;
use color_eyre::{
    eyre::{ensure, Context, ContextCompat},
    Result,
};
use colored::Colorize;
use log::info;

use crate::autodetect::generate_install_yml;

include!("generate_cmd.rs");

impl GenerateCmd {
    pub fn run(self) -> Result<()> {
        let package_dir = Utf8Path::from_path(&self.package_dir).with_context(|| {
            format!("{:?} contains invalid UTF-8 characters", self.package_dir)
        })?;
        let output = Utf8Path::from_path(&self.output)
            .with_context(|| format!("{:?} contains invalid UTF-8 characters", self.output))?;

        ensure!(
            self.force || !output.exists(),
            "{} already exists, use --force to overwrite it",
            output
        );

        let pkg_name = self
            .name
            .clone()
            .or_else(|| package_dir.file_name().map(str::to_owned))
            .unwrap_or_else(|| "package".to_owned());

        let yaml = generate_install_yml(package_dir, &pkg_name)?;

        std::fs::write(output, &yaml)
            .with_context(|| format!("unable to write file {output}"))?;

        info!(
            "{} {} {}",
            ">>>".magenta(),
            "Generated".bright_black(),
            output.as_str().cyan().bold()
        );
        info!("Review the file and adjust it before installing, then run: rinstall install");

        Ok(())
    }
}
