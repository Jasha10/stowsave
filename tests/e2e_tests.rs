use std::fs;
use std::path::PathBuf;
use std::process::Command;

use assert_cmd::Command as TestCommand;
use predicates::prelude::*;
use tempfile::TempDir;

fn stow_available() -> bool {
    Command::new("stow").arg("--version").output().is_ok()
}

struct Layout {
    _temp: TempDir,
    root: PathBuf,
    src_proj: PathBuf,
    stow_pkg: PathBuf,
}

/// Build a valid layout in a fresh tempdir:
///   <root>/src/proj/      <- where path_to_save lives
///   <root>/dotfiles/pkg/  <- stow package (grandchild of <root>)
/// All paths returned are canonicalized so they compare cleanly against
/// what stowsave canonicalizes internally (notably /tmp -> /private/tmp on macOS).
fn setup_layout() -> Layout {
    let temp = TempDir::new().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let src_proj = root.join("src").join("proj");
    let stow_pkg = root.join("dotfiles").join("pkg");
    fs::create_dir_all(&src_proj).unwrap();
    fs::create_dir_all(&stow_pkg).unwrap();
    Layout {
        _temp: temp,
        root,
        src_proj,
        stow_pkg,
    }
}

fn stowsave() -> TestCommand {
    TestCommand::cargo_bin("stowsave").unwrap()
}

#[test]
fn stowsave_single_file_happy_path() {
    if !stow_available() {
        eprintln!("stow not available; skipping");
        return;
    }
    let layout = setup_layout();
    let file = layout.src_proj.join("file.txt");
    fs::write(&file, "hello").unwrap();

    stowsave()
        .arg(&file)
        .arg(&layout.stow_pkg)
        .assert()
        .success();

    let backup = layout.src_proj.join("file.txt.bak");
    assert!(backup.is_file(), "backup should exist");
    assert_eq!(fs::read_to_string(&backup).unwrap(), "hello");

    let target_in_pkg = layout.stow_pkg.join("src").join("proj").join("file.txt");
    assert!(target_in_pkg.is_file(), "file should be in stow pkg");
    assert_eq!(fs::read_to_string(&target_in_pkg).unwrap(), "hello");

    assert!(file.is_symlink(), "original should be a symlink");
    assert_eq!(
        file.canonicalize().unwrap(),
        target_in_pkg.canonicalize().unwrap()
    );
}

#[test]
fn stowsave_directory_happy_path() {
    if !stow_available() {
        eprintln!("stow not available; skipping");
        return;
    }
    let layout = setup_layout();
    let cfg = layout.src_proj.join("cfg");
    fs::create_dir_all(cfg.join("nested")).unwrap();
    fs::write(cfg.join("a.txt"), "A").unwrap();
    fs::write(cfg.join("nested").join("b.txt"), "B").unwrap();

    stowsave().arg(&cfg).arg(&layout.stow_pkg).assert().success();

    // CreateBackup uses content_only(true): cfg.bak/a.txt (no nested cfg/).
    let backup = layout.src_proj.join("cfg.bak");
    assert!(backup.is_dir(), "backup dir should exist");
    assert_eq!(fs::read_to_string(backup.join("a.txt")).unwrap(), "A");
    assert_eq!(
        fs::read_to_string(backup.join("nested").join("b.txt")).unwrap(),
        "B"
    );

    let target_a = layout
        .stow_pkg
        .join("src")
        .join("proj")
        .join("cfg")
        .join("a.txt");
    let target_b = layout
        .stow_pkg
        .join("src")
        .join("proj")
        .join("cfg")
        .join("nested")
        .join("b.txt");
    assert!(target_a.is_file());
    assert!(target_b.is_file());

    // Read through the original path — works whether stow folded the dir
    // into a single symlink or unfolded into per-file symlinks.
    assert_eq!(fs::read_to_string(cfg.join("a.txt")).unwrap(), "A");
    assert_eq!(
        fs::read_to_string(cfg.join("nested").join("b.txt")).unwrap(),
        "B"
    );
}

#[test]
fn dry_run_leaves_filesystem_unchanged() {
    let layout = setup_layout();
    let file = layout.src_proj.join("file.txt");
    fs::write(&file, "hello").unwrap();

    let assert = stowsave()
        .arg("--dry-run")
        .arg(&file)
        .arg(&layout.stow_pkg)
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Dry run"), "stdout: {stdout}");
    assert!(stdout.contains("Back up"), "stdout: {stdout}");
    assert!(stdout.contains("Create directory"), "stdout: {stdout}");
    assert!(stdout.contains("Move"), "stdout: {stdout}");
    assert!(stdout.contains("Run 'stow"), "stdout: {stdout}");

    assert!(file.is_file() && !file.is_symlink());
    assert_eq!(fs::read_to_string(&file).unwrap(), "hello");
    assert!(!layout.src_proj.join("file.txt.bak").exists());
    let pkg_entries: Vec<_> = fs::read_dir(&layout.stow_pkg).unwrap().collect();
    assert!(pkg_entries.is_empty(), "stow pkg should still be empty");
}

#[test]
fn error_path_to_save_does_not_exist() {
    let layout = setup_layout();
    let missing = layout.src_proj.join("missing.txt");

    stowsave()
        .arg(&missing)
        .arg(&layout.stow_pkg)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Failed to canonicalize path_to_save",
        ));
}

// NOTE: `checks::path_to_save_is_not_symlink` is unreachable from the binary —
// `main.rs` canonicalizes path_to_save first, which resolves every symlink, so
// the check always sees a non-symlink. The unit test for it in `src/checks.rs`
// covers the function in isolation; we don't add an e2e test for it.

#[test]
fn error_stow_pkg_not_grandchild_of_common_ancestor() {
    let layout = setup_layout();
    let bad_stow = layout.root.join("just_a_dir");
    fs::create_dir_all(&bad_stow).unwrap();
    let file = layout.src_proj.join("file.txt");
    fs::write(&file, "hello").unwrap();

    stowsave()
        .arg(&file)
        .arg(&bad_stow)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "grandchild of the common ancestor",
        ));
}

#[test]
fn error_target_already_exists_in_stow_pkg() {
    let layout = setup_layout();
    let file = layout.src_proj.join("file.txt");
    fs::write(&file, "hello").unwrap();
    let target_dir = layout.stow_pkg.join("src").join("proj");
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(target_dir.join("file.txt"), "other").unwrap();

    stowsave()
        .arg(&file)
        .arg(&layout.stow_pkg)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}
