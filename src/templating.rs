use color_eyre::{
    eyre::{ensure, ContextCompat},
    Result,
};

use crate::Dirs;

pub fn apply_templating(
    contents: &[u8],
    dirs: &Dirs,
) -> Result<String> {
    macro_rules! replace_impl {
        ( $contents:tt, $dir:expr, $needle:literal ) => {
            $contents = $contents.replace(
                $needle,
                $dir.as_os_str()
                    .to_str()
                    .with_context(|| format!("unable to convert {:?} to String", $dir))?,
            );
        };
    }

    macro_rules! replace {
        ( $contents:tt, $dir:ident, $needle:literal ) => {
            replace_impl!($contents, &dirs.$dir, $needle);
        };
    }

    macro_rules! replace_when_some {
        ( $contents:tt, $dir:ident, $needle:literal ) => {
            if let Some($dir) = &dirs.$dir {
                replace_impl!($contents, $dir, $needle);
            } else {
                // TODO: Is this needed?
                ensure!(
                    !$contents.contains($needle),
                    "tried replacing {} when its value is none",
                    $needle
                );
            }
        };
    }

    let mut contents = String::from_utf8_lossy(contents).to_string();

    replace_when_some!(contents, prefix, "@prefix@");
    replace_when_some!(contents, exec_prefix, "@exec_prefix@");
    replace!(contents, bindir, "@bindir@");
    replace!(contents, libdir, "@libdir@");
    replace!(contents, datarootdir, "@datarootdir@");
    replace!(contents, datadir, "@datadir@");
    replace!(contents, sysconfdir, "@sysconfdir@");
    replace!(contents, localstatedir, "@localstatedir@");
    replace!(contents, runstatedir, "@runstatedir@");
    replace_when_some!(contents, includedir, "@includedir@");
    replace_when_some!(contents, docdir, "@docdir@");
    replace_when_some!(contents, mandir, "@mandir@");
    replace_when_some!(contents, pam_modulesdir, "@pam_moduledirs@");
    replace!(contents, systemd_unitsdir, "@systemd_unitsdir@");

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dirs::Dirs;
    use camino::Utf8PathBuf;

    fn test_dirs() -> Dirs {
        Dirs {
            prefix: Some(Utf8PathBuf::from("/usr/local")),
            exec_prefix: Some(Utf8PathBuf::from("/usr/local")),
            bindir: Utf8PathBuf::from("/usr/local/bin"),
            sbindir: None,
            libdir: Utf8PathBuf::from("/usr/local/lib"),
            libexecdir: Utf8PathBuf::from("/usr/local/libexec"),
            datarootdir: Utf8PathBuf::from("/usr/local/share"),
            datadir: Utf8PathBuf::from("/usr/local/share"),
            sysconfdir: Utf8PathBuf::from("/usr/local/etc"),
            localstatedir: Utf8PathBuf::from("/usr/local/var"),
            runstatedir: Utf8PathBuf::from("/usr/local/var/run"),
            includedir: Some(Utf8PathBuf::from("/usr/local/include")),
            docdir: Some(Utf8PathBuf::from("/usr/local/share/doc")),
            mandir: Some(Utf8PathBuf::from("/usr/local/share/man")),
            pam_modulesdir: Some(Utf8PathBuf::from("/usr/local/lib/security")),
            systemd_unitsdir: Utf8PathBuf::from("/usr/local/lib/systemd"),
        }
    }

    #[test]
    fn replaces_single_var() {
        let dirs = test_dirs();
        let result = apply_templating(b"bindir=@bindir@", &dirs).unwrap();
        assert_eq!(result, "bindir=/usr/local/bin");
    }

    #[test]
    fn replaces_multiple_vars() {
        let dirs = test_dirs();
        let result = apply_templating(b"p=@prefix@ d=@datadir@", &dirs).unwrap();
        assert_eq!(result, "p=/usr/local d=/usr/local/share");
    }

    #[test]
    fn plain_text_unchanged() {
        let dirs = test_dirs();
        let result = apply_templating(b"no placeholders here", &dirs).unwrap();
        assert_eq!(result, "no placeholders here");
    }

    #[test]
    fn optional_dir_present_is_substituted() {
        let dirs = test_dirs();
        let result = apply_templating(b"man=@mandir@", &dirs).unwrap();
        assert_eq!(result, "man=/usr/local/share/man");
    }

    #[test]
    fn optional_dir_absent_errors_when_placeholder_used() {
        let mut dirs = test_dirs();
        dirs.mandir = None;
        assert!(
            apply_templating(b"man=@mandir@", &dirs).is_err(),
            "should error when referencing an absent optional directory"
        );
    }

    #[test]
    fn optional_dir_absent_ok_when_placeholder_not_used() {
        let mut dirs = test_dirs();
        dirs.mandir = None;
        let result = apply_templating(b"no mandir ref", &dirs).unwrap();
        assert_eq!(result, "no mandir ref");
    }
}
