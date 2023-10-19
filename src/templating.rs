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
