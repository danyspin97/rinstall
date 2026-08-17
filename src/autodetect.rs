use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::{eyre::bail, Result};
use serde::Serialize;

// Well-known top-level file name stems (case-insensitive, extension ignored) that are
// treated as documentation / licenses rather than left unclassified.
const DOC_NAMES: &[&str] = &[
    "readme",
    "changelog",
    "changes",
    "news",
    "authors",
    "contributing",
    "notice",
    "history",
    "todo",
];
const LICENSE_NAMES: &[&str] = &["license", "licence", "copying", "copying.lesser", "unlicense"];

// Scripts that are almost always part of a project's own build/release tooling rather than
// something meant to be installed, even when they carry the executable bit.
const SKIP_EXE_NAMES: &[&str] = &[
    "configure",
    "autogen.sh",
    "bootstrap.sh",
    "build.sh",
    "install.sh",
    "uninstall.sh",
    "setup.py",
    "makefile",
    "cmakelists.txt",
    "meson.build",
];
const SCRIPT_EXTENSIONS: &[&str] = &["sh", "bash", "zsh", "fish", "py", "pl", "rb", "lua", "awk"];

#[derive(Serialize, Default)]
struct DetectedCompletions {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bash: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elvish: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fish: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    zsh: Vec<String>,
}

impl DetectedCompletions {
    fn is_empty(&self) -> bool {
        self.bash.is_empty()
            && self.elvish.is_empty()
            && self.fish.is_empty()
            && self.zsh.is_empty()
    }

    fn sort(&mut self) {
        self.bash.sort();
        self.elvish.sort();
        self.fish.sort();
        self.zsh.sort();
    }
}

#[derive(Serialize, Default)]
struct DetectedPackage {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    exe: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    docs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    licenses: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    man: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "desktop-files")]
    desktop_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "appstream-metadata")]
    appstream_metadata: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "systemd-units")]
    systemd_units: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "systemd-user-units")]
    systemd_user_units: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "pkg-config")]
    pkg_config: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    config: Vec<String>,
    #[serde(skip_serializing_if = "DetectedCompletions::is_empty")]
    completions: DetectedCompletions,
}

impl DetectedPackage {
    fn is_empty(&self) -> bool {
        self.exe.is_empty()
            && self.docs.is_empty()
            && self.licenses.is_empty()
            && self.man.is_empty()
            && self.desktop_files.is_empty()
            && self.appstream_metadata.is_empty()
            && self.systemd_units.is_empty()
            && self.systemd_user_units.is_empty()
            && self.pkg_config.is_empty()
            && self.config.is_empty()
            && self.completions.is_empty()
    }

    fn sort(&mut self) {
        self.exe.sort();
        self.docs.sort();
        self.licenses.sort();
        self.man.sort();
        self.desktop_files.sort();
        self.appstream_metadata.sort();
        self.systemd_units.sort();
        self.systemd_user_units.sort();
        self.pkg_config.sort();
        self.config.sort();
        self.completions.sort();
    }
}

#[derive(Serialize)]
struct DetectedSpec {
    rinstall: &'static str,
    pkgs: BTreeMap<String, DetectedPackage>,
}

fn is_executable(path: &Utf8Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn stem(file_name: &str) -> &str {
    file_name.split('.').next().unwrap_or(file_name)
}

fn stem_matches(
    file_name: &str,
    names: &[&str],
) -> bool {
    names.contains(&stem(file_name).to_lowercase().as_str())
}

// Man pages use a single-digit-led extension (man1..man8), e.g. `foo.1`, `foo.3pm`.
fn is_man_page(file_name: &str) -> bool {
    Utf8Path::new(file_name)
        .extension()
        .and_then(|ext| ext.chars().next())
        .is_some_and(|c| ('1'..='8').contains(&c))
}

fn is_candidate_exe(
    full_path: &Utf8Path,
    file_name: &str,
) -> bool {
    let lower = file_name.to_lowercase();
    if SKIP_EXE_NAMES.contains(&lower.as_str()) {
        return false;
    }
    if !is_executable(full_path) {
        return false;
    }
    match Utf8Path::new(file_name).extension() {
        None => true,
        Some(ext) => SCRIPT_EXTENSIONS.contains(&ext.to_lowercase().as_str()),
    }
}

fn dir_entries(dir: &Utf8Path) -> Result<Vec<(String, Utf8PathBuf, std::fs::FileType)>> {
    let mut res = Vec::new();
    for entry in fs::read_dir(dir)?.filter_map(std::result::Result::ok) {
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            // Skip entries with non-UTF-8 names rather than failing the whole scan.
            continue;
        };
        let file_type = entry.file_type()?;
        let full_path = dir.join(&file_name);
        res.push((file_name, full_path, file_type));
    }
    Ok(res)
}

// Collect the (one level, non-recursive) files of `dir` whose file name matches `filter`,
// pushing their path relative to the project root (`rel_prefix/file_name`) into `dst`.
fn collect_dir_files<F>(
    dir: &Utf8Path,
    rel_prefix: &Utf8Path,
    filter: F,
    dst: &mut Vec<String>,
) -> Result<()>
where
    F: Fn(&str) -> bool,
{
    for (file_name, _full_path, file_type) in dir_entries(dir)? {
        if !file_type.is_file() || !filter(&file_name) {
            continue;
        }
        dst.push(rel_prefix.join(&file_name).into_string());
    }
    Ok(())
}

/// Scan a project directory for common file layout conventions and produce a best-effort
/// `install.yml` package definition (as YAML text) for a package without a build system.
/// Only well-known conventions are recognized; anything ambiguous is left out rather than
/// guessed, so the result should still be reviewed before installing.
pub fn generate_install_yml(
    package_dir: &Utf8Path,
    pkg_name: &str,
) -> Result<String> {
    let mut pkg = DetectedPackage::default();

    for (file_name, full_path, file_type) in dir_entries(package_dir)? {
        if file_name.starts_with('.') {
            continue;
        }

        if file_type.is_dir() {
            match file_name.as_str() {
                "man" => collect_dir_files(&full_path, Utf8Path::new("man"), is_man_page, &mut pkg.man)?,
                "etc" => collect_dir_files(&full_path, Utf8Path::new("etc"), |_| true, &mut pkg.config)?,
                "systemd" => {
                    let system_dir = full_path.join("system");
                    let user_dir = full_path.join("user");
                    let is_service = |name: &str| name.to_lowercase().ends_with(".service");
                    if system_dir.is_dir() {
                        collect_dir_files(
                            &system_dir,
                            Utf8Path::new("systemd/system"),
                            is_service,
                            &mut pkg.systemd_units,
                        )?;
                    }
                    if user_dir.is_dir() {
                        collect_dir_files(
                            &user_dir,
                            Utf8Path::new("systemd/user"),
                            is_service,
                            &mut pkg.systemd_user_units,
                        )?;
                    }
                    if !system_dir.is_dir() && !user_dir.is_dir() {
                        collect_dir_files(
                            &full_path,
                            Utf8Path::new("systemd"),
                            is_service,
                            &mut pkg.systemd_units,
                        )?;
                    }
                }
                "completions" => {
                    let shell_dirs = [
                        ("bash", &mut pkg.completions.bash),
                        ("elvish", &mut pkg.completions.elvish),
                        ("fish", &mut pkg.completions.fish),
                        ("zsh", &mut pkg.completions.zsh),
                    ];
                    let mut any_shell_dir = false;
                    for (shell, dst) in shell_dirs {
                        let shell_dir = full_path.join(shell);
                        if shell_dir.is_dir() {
                            any_shell_dir = true;
                            collect_dir_files(
                                &shell_dir,
                                &Utf8PathBuf::from(format!("completions/{shell}")),
                                |_| true,
                                dst,
                            )?;
                        }
                    }
                    // Fall back to a flat completions/ dir classified by extension,
                    // e.g. completions/foo.bash, completions/foo.fish
                    if !any_shell_dir {
                        collect_dir_files(&full_path, Utf8Path::new("completions"), |n| {
                            n.ends_with(".bash")
                        }, &mut pkg.completions.bash)?;
                        collect_dir_files(&full_path, Utf8Path::new("completions"), |n| {
                            n.ends_with(".zsh")
                        }, &mut pkg.completions.zsh)?;
                        collect_dir_files(&full_path, Utf8Path::new("completions"), |n| {
                            n.ends_with(".fish")
                        }, &mut pkg.completions.fish)?;
                        collect_dir_files(&full_path, Utf8Path::new("completions"), |n| {
                            n.ends_with(".elv") || n.ends_with(".elvish")
                        }, &mut pkg.completions.elvish)?;
                    }
                }
                _ => {}
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let lower = file_name.to_lowercase();

        if stem_matches(&file_name, LICENSE_NAMES) {
            pkg.licenses.push(file_name);
        } else if stem_matches(&file_name, DOC_NAMES) {
            pkg.docs.push(file_name);
        } else if is_man_page(&file_name) {
            pkg.man.push(file_name);
        } else if lower.ends_with(".desktop") {
            pkg.desktop_files.push(file_name);
        } else if lower.ends_with(".appdata.xml") || lower.ends_with(".metainfo.xml") {
            pkg.appstream_metadata.push(file_name);
        } else if lower.ends_with(".pc") {
            pkg.pkg_config.push(file_name);
        } else if lower.ends_with(".service") {
            pkg.systemd_units.push(file_name);
        } else if is_candidate_exe(&full_path, &file_name) {
            pkg.exe.push(file_name);
        }
    }

    pkg.sort();

    if pkg.is_empty() {
        bail!(
            "unable to auto-detect any installable files in {package_dir}; write an install.yml manually"
        );
    }

    let mut pkgs = BTreeMap::new();
    pkgs.insert(pkg_name.to_owned(), pkg);

    Ok(serde_yaml::to_string(&DetectedSpec {
        rinstall: "0.2.0",
        pkgs,
    })?)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use camino::Utf8PathBuf;
    use tempfile::tempdir;

    use super::generate_install_yml;

    fn make_exe(path: &std::path::Path) {
        std::fs::write(path, "#!/bin/sh\necho hi\n").unwrap();
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
    }

    #[test]
    fn detects_readme_license_and_exe() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "hello").unwrap();
        std::fs::write(dir.path().join("LICENSE"), "MIT").unwrap();
        make_exe(&dir.path().join("myprog"));

        let package_dir = Utf8PathBuf::from_path_buf(dir.path().to_owned()).unwrap();
        let yaml = generate_install_yml(&package_dir, "myprog").unwrap();

        assert!(yaml.contains("exe:"));
        assert!(yaml.contains("- myprog"));
        assert!(yaml.contains("docs:"));
        assert!(yaml.contains("- README.md"));
        assert!(yaml.contains("licenses:"));
        assert!(yaml.contains("- LICENSE"));
    }

    #[test]
    fn skips_non_executable_scripts_and_build_scripts() {
        let dir = tempdir().unwrap();
        // not executable -> skipped
        std::fs::write(dir.path().join("helper.sh"), "#!/bin/sh\n").unwrap();
        // executable but a known build script -> skipped
        make_exe(&dir.path().join("configure"));
        std::fs::write(dir.path().join("README"), "hello").unwrap();

        let package_dir = Utf8PathBuf::from_path_buf(dir.path().to_owned()).unwrap();
        let yaml = generate_install_yml(&package_dir, "pkg").unwrap();

        assert!(!yaml.contains("exe:"));
        assert!(yaml.contains("docs:"));
    }

    #[test]
    fn detects_man_pages_desktop_and_systemd_units() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("man")).unwrap();
        std::fs::write(dir.path().join("man").join("foo.1"), "man page").unwrap();
        std::fs::write(dir.path().join("foo.desktop"), "[Desktop Entry]").unwrap();
        std::fs::create_dir_all(dir.path().join("systemd").join("system")).unwrap();
        std::fs::write(
            dir.path().join("systemd").join("system").join("foo.service"),
            "[Unit]",
        )
        .unwrap();

        let package_dir = Utf8PathBuf::from_path_buf(dir.path().to_owned()).unwrap();
        let yaml = generate_install_yml(&package_dir, "foo").unwrap();

        assert!(yaml.contains("man:"));
        assert!(yaml.contains("- man/foo.1"));
        assert!(yaml.contains("desktop-files:"));
        assert!(yaml.contains("- foo.desktop"));
        assert!(yaml.contains("systemd-units:"));
        assert!(yaml.contains("- systemd/system/foo.service"));
    }

    #[test]
    fn detects_shell_completions() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("completions").join("bash")).unwrap();
        std::fs::write(
            dir.path().join("completions").join("bash").join("foo.bash"),
            "complete",
        )
        .unwrap();

        let package_dir = Utf8PathBuf::from_path_buf(dir.path().to_owned()).unwrap();
        let yaml = generate_install_yml(&package_dir, "foo").unwrap();

        assert!(yaml.contains("bash:"));
        assert!(yaml.contains("- completions/bash/foo.bash"));
    }

    #[test]
    fn errors_when_nothing_detected() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("random.txt"), "nothing to see here").unwrap();

        let package_dir = Utf8PathBuf::from_path_buf(dir.path().to_owned()).unwrap();
        assert!(generate_install_yml(&package_dir, "pkg").is_err());
    }
}
