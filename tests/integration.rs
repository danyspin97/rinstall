/// Integration tests for rinstall.
///
/// Each test runs the compiled binary as a subprocess with `--packaging --yes
/// --destdir <tmp>`, verifying that installed files land at the right paths with
/// correct permissions and contents.  `--packaging` implies system directories
/// and skips writing pkg-info, making it safe to use with an ephemeral destdir.
///
/// Source files are relative paths resolved from the subprocess's working
/// directory, which is set to the per-test source tree (a tempdir).
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

use tempfile::TempDir;

// ── helpers ──────────────────────────────────────────────────────────────────

fn rinstall(src: &TempDir, dest: &TempDir) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rinstall"));
    cmd.current_dir(src.path())
        .args(["install", "--packaging", "--yes"])
        .arg("--destdir")
        .arg(dest.path());
    cmd
}

/// Write a file inside `dir`, creating parent directories as needed.
fn write_src(dir: &TempDir, name: &str, content: &[u8]) {
    let path = dir.path().join(name);
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(path, content).unwrap();
}

/// Resolve a system-absolute path relative to `dest`.
fn at(dest: &TempDir, system_path: &str) -> PathBuf {
    dest.path().join(system_path.trim_start_matches('/'))
}

fn mode(path: &PathBuf) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

// ── permission tests ──────────────────────────────────────────────────────────

#[test]
fn exe_gets_0755() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "mybinary", b"#!/bin/sh\necho hi");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    exe:
      - mybinary
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    let installed = at(&dest, "/usr/local/bin/mybinary");
    assert!(installed.exists(), "binary not found at {}", installed.display());
    assert_eq!(mode(&installed), 0o755, "expected 0o755, got 0o{:o}", mode(&installed));
}

#[test]
fn data_file_gets_0644() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "data.txt", b"some data");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    data:
      - data.txt
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    let installed = at(&dest, "/usr/local/share/test-pkg/data.txt");
    assert!(installed.exists());
    assert_eq!(mode(&installed), 0o644, "expected 0o644, got 0o{:o}", mode(&installed));
}

#[test]
fn config_file_gets_0644() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "myapp.conf", b"key=value");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    config:
      - myapp.conf
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    let installed = at(&dest, "/usr/local/etc/test-pkg/myapp.conf");
    assert!(installed.exists());
    assert_eq!(mode(&installed), 0o644, "expected 0o644, got 0o{:o}", mode(&installed));
}

#[test]
fn man_page_gets_0644() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "myapp.1", b".TH MYAPP 1");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    man:
      - myapp.1
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    let installed = at(&dest, "/usr/local/share/man/man1/myapp.1");
    assert!(installed.exists());
    assert_eq!(mode(&installed), 0o644, "expected 0o644, got 0o{:o}", mode(&installed));
}

// ── routing tests ─────────────────────────────────────────────────────────────

#[test]
fn man_pages_routed_to_correct_section_dir() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "foo.1", b".TH FOO 1");
    write_src(&src, "foo.5", b".TH FOO 5");
    write_src(&src, "foo.8", b".TH FOO 8");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    man:
      - foo.1
      - foo.5
      - foo.8
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    assert!(at(&dest, "/usr/local/share/man/man1/foo.1").exists(), "man1 missing");
    assert!(at(&dest, "/usr/local/share/man/man5/foo.5").exists(), "man5 missing");
    assert!(at(&dest, "/usr/local/share/man/man8/foo.8").exists(), "man8 missing");
}

#[test]
fn data_goes_into_pkgname_subdir() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "readme.txt", b"readme");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  myapp:
    data:
      - readme.txt
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());
    assert!(at(&dest, "/usr/local/share/myapp/readme.txt").exists());
}

// ── directory install ─────────────────────────────────────────────────────────

#[test]
fn directory_install_copies_all_files() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "assets/icon.png", b"\x89PNG");
    write_src(&src, "assets/style.css", b"body {}");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    data:
      - assets
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    // directory prefix is stripped; files land directly in the pkg data dir
    let icon = at(&dest, "/usr/local/share/test-pkg/icon.png");
    let css = at(&dest, "/usr/local/share/test-pkg/style.css");
    assert!(icon.exists(), "icon.png not found");
    assert!(css.exists(), "style.css not found");
    assert_eq!(fs::read(&icon).unwrap(), b"\x89PNG");
    assert_eq!(fs::read(&css).unwrap(), b"body {}");
    assert_eq!(mode(&icon), 0o644);
    assert_eq!(mode(&css), 0o644);
}

// ── content correctness ───────────────────────────────────────────────────────

#[test]
fn file_content_preserved_exactly() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    // Arbitrary bytes including nulls and high bytes
    let content: Vec<u8> = (0u8..=255).collect();
    write_src(&src, "data.bin", &content);
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    data:
      - data.bin
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    let installed = fs::read(at(&dest, "/usr/local/share/test-pkg/data.bin")).unwrap();
    assert_eq!(installed, content, "content must survive the copy byte-for-byte");
}

// ── templating ────────────────────────────────────────────────────────────────

#[test]
fn templating_substitutes_placeholders() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "myapp.pc", b"prefix=@prefix@\nbindir=@bindir@\ndatadir=@datadir@");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    data:
      - src: myapp.pc
        tmpl: true
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    let content = fs::read_to_string(at(&dest, "/usr/local/share/test-pkg/myapp.pc")).unwrap();
    assert!(!content.contains('@'), "all @placeholders@ should be replaced, got:\n{content}");
    assert!(content.contains("/usr/local"), "resolved paths should appear, got:\n{content}");
}

#[test]
fn templating_applied_to_file_installed_under_destdir() {
    // Regression: apply_templating must read the file from destination_destdir,
    // not from the bare destination path (which doesn't exist when destdir is set).
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "cfg.conf", b"bin=@bindir@");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    config:
      - src: cfg.conf
        tmpl: true
"#,
    );

    // Must succeed (previously would fail trying to open the non-destdir path)
    assert!(rinstall(&src, &dest).status().unwrap().success());

    let content = fs::read_to_string(at(&dest, "/usr/local/etc/test-pkg/cfg.conf")).unwrap();
    assert!(!content.contains("@bindir@"), "placeholder must be resolved");
}

// ── multi-type package ────────────────────────────────────────────────────────

#[test]
fn multiple_file_types_in_one_package() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    write_src(&src, "myapp", b"#!/bin/sh");
    write_src(&src, "myapp.1", b".TH MYAPP 1");
    write_src(&src, "myapp.conf", b"key=value");
    write_src(&src, "readme.txt", b"docs");
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  myapp:
    exe:
      - myapp
    man:
      - myapp.1
    config:
      - myapp.conf
    data:
      - readme.txt
"#,
    );

    assert!(rinstall(&src, &dest).status().unwrap().success());

    let exe = at(&dest, "/usr/local/bin/myapp");
    let man = at(&dest, "/usr/local/share/man/man1/myapp.1");
    let cfg = at(&dest, "/usr/local/etc/myapp/myapp.conf");
    let dat = at(&dest, "/usr/local/share/myapp/readme.txt");

    assert!(exe.exists(), "exe missing");
    assert!(man.exists(), "man page missing");
    assert!(cfg.exists(), "config missing");
    assert!(dat.exists(), "data file missing");

    assert_eq!(mode(&exe), 0o755, "exe must be 0o755");
    assert_eq!(mode(&man), 0o644, "man must be 0o644");
    assert_eq!(mode(&cfg), 0o644, "config must be 0o644");
    assert_eq!(mode(&dat), 0o644, "data must be 0o644");
}

// ── invalid inputs ────────────────────────────────────────────────────────────

#[test]
fn missing_source_file_exits_nonzero() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    // install.yml references a file that does not exist
    write_src(
        &src,
        "install.yml",
        br#"rinstall: "0.2.0"
pkgs:
  test-pkg:
    exe:
      - nonexistent_binary
"#,
    );

    let status = rinstall(&src, &dest).status().unwrap();
    assert!(!status.success(), "should fail when source file is missing");
}
