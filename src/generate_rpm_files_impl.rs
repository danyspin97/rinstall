use std::collections::HashSet;

use clap::Parser;
use color_eyre::{eyre::WrapErr, Result};

use crate::{
    dirs::Dirs, dirs_config_impl::DirsConfig, install_spec::InstallSpec, package::Type,
    project::Project,
};

include!("generate_rpm_files.rs");

impl GenerateRpmFiles {
    pub fn run(self) -> Result<()> {
        let dirs_config = DirsConfig::load(self.config.as_deref(), self.system, &self.dirs)?;
        let dirs = Dirs::new(dirs_config, self.system).context("unable to create dirs")?;
        let install_spec = InstallSpec::new_from_path(&self.package_dir)?;
        let version = install_spec.version.clone();
        let packages = install_spec.packages(&self.packages);
        let mut rpm_files = Vec::new();

        for package in packages {
            let targets = package.targets(
                &dirs,
                &Project::new_from_type(
                    Type::Default,
                    &self.package_dir,
                    false,
                    false,
                    // opts.rust_debug_target,
                )?,
                &version,
                true,
            )?;
            for target in targets {
                rpm_files.append(&mut target.generate_rpm_files()?);
            }
        }

        let mut res = String::new();
        let mut owned_dir = HashSet::new();
        owned_dir.insert(dirs.bindir);
        owned_dir.insert(dirs.datadir);
        owned_dir.insert(dirs.datarootdir.clone());
        // unwrapping here is safe, because GenerateRpmFiles requires --system
        owned_dir.insert(dirs.docdir.unwrap());
        owned_dir.insert(dirs.includedir.unwrap());
        owned_dir.insert(dirs.libdir);
        owned_dir.insert(dirs.libexecdir);
        owned_dir.insert(dirs.localstatedir);
        let mandir = dirs.mandir.unwrap();
        owned_dir.insert(mandir.clone());
        for i in 1..8 {
            owned_dir.insert(mandir.join(format!("man{i}")));
        }
        owned_dir.insert(dirs.pam_modulesdir.unwrap());
        owned_dir.insert(dirs.sbindir.unwrap());
        owned_dir.insert(dirs.sysconfdir);
        owned_dir.insert(dirs.systemd_unitsdir);
        owned_dir.insert(dirs.datarootdir.join("licenses"));
        owned_dir.insert(dirs.datarootdir.join("applications"));
        owned_dir.insert(dirs.datarootdir.join("zsh").join("site-functions"));
        owned_dir.insert(dirs.datarootdir.join("bash-completion").join("completions"));
        owned_dir.insert(dirs.datarootdir.join("fish").join("vendor_completions.d"));

        for file in rpm_files {
            let mut parent_dirs = Vec::new();
            let mut parent = file.parent().unwrap();
            loop {
                if owned_dir.contains(parent) {
                    break;
                }
                owned_dir.insert(parent.to_path_buf());
                parent_dirs.push(parent);
                parent = parent.parent().unwrap();
            }
            for dir in parent_dirs.iter().rev() {
                res.push_str("%dir ");
                res.push_str(dir.to_str().unwrap());
                res.push('\n');
            }
            res.push_str(file.to_str().unwrap());
            res.push('\n');
        }

        println!("{}", res);

        Ok(())
    }
}
