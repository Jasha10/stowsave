// use assert_cmd::Command;
// use predicates::prelude::*;
// use std::fs::{self, File};
// use std::io::Write;
// use tempfile::TempDir;
//
// #[test]
// fn test_stowsave_single_file() -> Result<(), Box<dyn std::error::Error>> {
//     let temp_dir = TempDir::new()?;
//     let home_dir = temp_dir.path().join("home");
//     let stow_dir = temp_dir.path().join("stow");
//
//     fs::create_dir_all(&home_dir)?;
//     fs::create_dir_all(&stow_dir)?;
//
//     let vimrc_path = home_dir.join(".vimrc");
//     let mut vimrc_file = File::create(&vimrc_path)?;
//     writeln!(vimrc_file, "set number")?;
//
//     let mut cmd = Command::cargo_bin("stowsave")?;
//     cmd.arg(&vimrc_path).arg(&stow_dir);
//     cmd.assert().success();
//
//     assert!(vimrc_path.with_extension("bak").exists());
//     assert!(stow_dir.join(".vimrc").exists());
//     assert!(vimrc_path.is_symlink());
//
//     Ok(())
// }
//
// #[test]
// fn test_stowsave_directory() -> Result<(), Box<dyn std::error::Error>> {
//     let temp_dir = TempDir::new()?;
//     let home_dir = temp_dir.path().join("home");
//     let stow_dir = temp_dir.path().join("stow");
//
//     fs::create_dir_all(&home_dir)?;
//     fs::create_dir_all(&stow_dir)?;
//
//     let config_dir = home_dir.join(".config");
//     fs::create_dir_all(&config_dir)?;
//
//     let nvim_dir = config_dir.join("nvim");
//     fs::create_dir_all(&nvim_dir)?;
//
//     let init_vim_path = nvim_dir.join("init.vim");
//     let mut init_vim_file = File::create(&init_vim_path)?;
//     writeln!(init_vim_file, "set relativenumber")?;
//
//     let mut cmd = Command::cargo_bin("stowsave")?;
//     cmd.arg(&config_dir).arg(&stow_dir);
//     cmd.assert().success();
//
//     assert!(config_dir.with_extension("bak").exists());
//     assert!(stow_dir
//         .join(".config")
//         .join("nvim")
//         .join("init.vim")
//         .exists());
//     assert!(config_dir.is_symlink());
//
//     Ok(())
// }
//
// #[test]
// fn test_stowsave_nonexistent_path() -> Result<(), Box<dyn std::error::Error>> {
//     let temp_dir = TempDir::new()?;
//     let home_dir = temp_dir.path().join("home");
//     let stow_dir = temp_dir.path().join("stow");
//
//     fs::create_dir_all(&home_dir)?;
//     fs::create_dir_all(&stow_dir)?;
//
//     let nonexistent_path = home_dir.join("nonexistent");
//
//     let mut cmd = Command::cargo_bin("stowsave")?;
//     cmd.arg(&nonexistent_path).arg(&stow_dir);
//     cmd.assert()
//         .failure()
//         .stderr(predicate::str::contains("Path does not exist"));
//
//     Ok(())
// }
//
// #[test]
// fn test_stowsave_invalid_stow_directory() -> Result<(), Box<dyn std::error::Error>> {
//     let temp_dir = TempDir::new()?;
//     let home_dir = temp_dir.path().join("home");
//     let invalid_stow_dir = temp_dir.path().join("invalid_stow");
//
//     fs::create_dir_all(&home_dir)?;
//
//     let vimrc_path = home_dir.join(".vimrc");
//     let mut vimrc_file = File::create(&vimrc_path)?;
//     writeln!(vimrc_file, "set number")?;
//
//     let mut cmd = Command::cargo_bin("stowsave")?;
//     cmd.arg(&vimrc_path).arg(&invalid_stow_dir);
//     cmd.assert()
//         .failure()
//         .stderr(predicate::str::contains("Invalid stow directory"));
//
//     Ok(())
// }
//
// #[test]
// fn test_stowsave_symlink() -> Result<(), Box<dyn std::error::Error>> {
//     let temp_dir = TempDir::new()?;
//     let home_dir = temp_dir.path().join("home");
//     let stow_dir = temp_dir.path().join("stow");
//
//     fs::create_dir_all(&home_dir)?;
//     fs::create_dir_all(&stow_dir)?;
//
//     let vimrc_path = home_dir.join(".vimrc");
//     let mut vimrc_file = File::create(&vimrc_path)?;
//     writeln!(vimrc_file, "set number")?;
//
//     let symlink_path = home_dir.join(".vimrc_link");
//     std::os::unix::fs::symlink(&vimrc_path, &symlink_path)?;
//
//     let mut cmd = Command::cargo_bin("stowsave")?;
//     cmd.arg(&symlink_path).arg(&stow_dir);
//     cmd.assert()
//         .failure()
//         .stderr(predicate::str::contains("Cannot save symlinks"));
//
//     Ok(())
// }
