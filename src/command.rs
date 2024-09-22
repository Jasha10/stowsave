//! This module contains the implementions for Commands that execute side effects to modify the
//! filesytem.
use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result};
use fs_extra::dir::CopyOptions;

/// Commands to execute side effects to modify the filesystem.
#[derive(Debug)]
pub(super) enum Command {
    CreateDirIfNotExists(PathBuf),
    /// Move a file or directory into another directory.
    /// Error if `dir/dest_dir` already exists.
    MoveToDir {
        /// The file or directory to move.
        from: PathBuf,
        /// The directory into which to move.
        dest_dir: PathBuf,
    },
    CreateBackup {
        original: PathBuf,
        backup_name: String,
    },
    RunStow {
        pwd: PathBuf,
        package: String,
    },
}

pub(super) trait CommandImpl {
    fn invoke(&self, verbose: bool) -> Result<()>;
}

impl CommandImpl for Command {
    fn invoke(&self, verbose: bool) -> Result<()> {
        match self {
            Command::CreateDirIfNotExists(path) => {
                if verbose {
                    println!("Creating directory: '{}'", path.display());
                }
                fs::create_dir_all(path).context("Failed to create directory")
            }
            Command::MoveToDir { from, dest_dir } => {
                if verbose {
                    println!("Moving '{}' to '{}'", from.display(), dest_dir.display());
                }
                fs_extra::move_items(&vec![from], dest_dir, &CopyOptions::new())?;
                Ok(())
            }
            Command::CreateBackup {
                original,
                backup_name,
            } => {
                if verbose {
                    println!("Creating backup directory: '{}'", backup_name);
                }
                let backup_path = original.with_file_name(backup_name);
                if original.is_file() {
                    fs::copy(original, backup_path).context("Failed to create backup")?;
                } else if original.is_dir() {
                    fs_extra::dir::copy(
                        original,
                        backup_path,
                        &CopyOptions::new().content_only(true),
                    )?;
                } else {
                    return Err(anyhow::anyhow!("Path is not a file or directory"));
                }
                Ok(())
            }

            Command::RunStow { pwd, package } => {
                if verbose {
                    println!(
                        "Running 'stow {}' in directory '{}'",
                        package,
                        pwd.display()
                    );
                }
                let output = ProcessCommand::new("stow")
                    .arg(package)
                    .current_dir(pwd)
                    .output()
                    .context("Failed to run stow command")?;

                if verbose {
                    println!("Stow stdout: {}", String::from_utf8_lossy(&output.stdout));
                    println!("Stow stderr: {}", String::from_utf8_lossy(&output.stderr));
                }

                if output.status.success() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Failed to run 'stow {}': {}",
                        package,
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_directory");
        let nested_dir = new_dir.join("nested");
        Command::CreateDirIfNotExists(nested_dir.clone())
            .invoke(true)
            .unwrap();

        assert!(nested_dir.is_dir());
    }

    #[test]
    fn test_create_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_directory");
        let nested_dir = new_dir.join("nested");
        fs::create_dir_all(&nested_dir).unwrap();
        Command::CreateDirIfNotExists(nested_dir.clone())
            .invoke(true)
            .unwrap();
        assert!(nested_dir.is_dir());
    }

    #[test]
    fn test_move_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        // create source file
        let source = temp_path.join("source.txt");
        fs::write(&source, "test content").unwrap();
        // create dest dir
        let dest_dir = temp_path.join("dest_dir");
        fs::create_dir(&dest_dir).unwrap();
        let dest_file = dest_dir.join("source.txt");

        // move source file to dest dir
        Command::MoveToDir {
            from: source.clone(),
            dest_dir: dest_dir.clone(),
        }
        .invoke(true)
        .unwrap();

        assert!(!source.exists());
        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(dest_file).unwrap(), "test content");
    }

    #[test]
    fn test_move_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        // create source dir
        let source_dir = temp_path.join("source_dir");
        fs::create_dir(&source_dir).unwrap();
        // create file in source dir
        let file_in_source = source_dir.join("file.txt");
        fs::write(&file_in_source, "test content").unwrap();
        // create dest dir
        let destination_dir = temp_path.join("destination_dir");
        fs::create_dir(&destination_dir).unwrap(); // Create the destination directory

        // move source dir to dest dir
        Command::MoveToDir {
            from: source_dir.clone(),
            dest_dir: destination_dir.clone(),
        }
        .invoke(true)
        .unwrap();

        assert!(!source_dir.exists());
        assert!(destination_dir.exists());
        assert!(destination_dir.join("source_dir").join("file.txt").exists());
    }

    #[test]
    fn test_move_to_dir_error_if_target_already_exists() {
        // Create source.txt and dest_dir/source.txt
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let source = temp_path.join("source.txt");
        fs::write(&source, "test content").unwrap();
        let dest_dir = temp_path.join("dest_dir");
        fs::create_dir(&dest_dir).unwrap();
        let dest_file = dest_dir.join("source.txt");
        fs::write(&dest_file, "existing content").unwrap();

        // Try to move source.txt to dest_dir
        let result = Command::MoveToDir {
            from: source.clone(),
            dest_dir: dest_dir.clone(),
        }
        .invoke(true);

        // Check that the error message is correct
        assert!(result.is_err());

        // Check the source and dest file contents
        assert!(source.exists());
        assert_eq!(fs::read_to_string(source).unwrap(), "test content");
        assert!(dest_file.exists());
        assert_eq!(fs::read_to_string(dest_file).unwrap(), "existing content");
    }

    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let source = temp_path.join("source.txt");
        fs::write(&source, "test content").unwrap();

        let backup_name = "source.txt.bak";

        Command::CreateBackup {
            original: source.clone(),
            backup_name: backup_name.to_string(),
        }
        .invoke(true)
        .unwrap();

        let backup_path = source.with_file_name(backup_name);
        assert!(source.exists()); // The source file should still exist
        assert!(backup_path.exists()); // The backup file should be created

        // Check that the content is the same
        let original_content = fs::read_to_string(&source).unwrap();
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(original_content, backup_content);
    }

    #[test]
    fn test_create_backup_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create source directory and file
        let source_dir = temp_path.join("dir");
        fs::create_dir_all(&source_dir).unwrap();
        let original_file = source_dir.join("file.txt");
        fs::write(&original_file, "test content").unwrap();

        let backup_name = "dir.bak";

        Command::CreateBackup {
            original: source_dir.clone(),
            backup_name: backup_name.to_string(),
        }
        .invoke(true)
        .unwrap();

        let backup_dir = temp_path.join(backup_name);
        // Check if the backup directory exists
        assert!(backup_dir.exists());
        assert!(backup_dir.is_dir());

        // Check if the file was backed up
        let backup_file = backup_dir.join("file.txt");
        // print the contents of the backup dir
        println!("Contents of backup dir:");
        for entry in fs::read_dir(&backup_dir).unwrap() {
            let entry = entry.unwrap();
            println!("{:?}", entry.path());
        }
        assert!(backup_file.exists());

        // Check that the content is the same
        let original_content = fs::read_to_string(&original_file).unwrap();
        let backup_content = fs::read_to_string(&backup_file).unwrap();
        assert_eq!(original_content, backup_content);
    }

    #[test]
    fn test_run_stow() {
        // Check if stow is available
        if ProcessCommand::new("stow")
            .arg("--version")
            .output()
            .is_err()
        {
            println!("Stow is not available on this system. Skipping test.");
            return;
        }

        // Create a temporary directory for our test
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let parent_dir = temp_path.join("parent");
        let stow_dir = parent_dir.join("stow_dir");
        let file_in_stow_dir = stow_dir.join("tmp_file.txt");

        // Create the directory structure
        fs::create_dir_all(&stow_dir).unwrap();
        fs::write(&file_in_stow_dir, "test content").unwrap();
        println!("File in stow dir: {:?}", file_in_stow_dir);

        // Run stow command
        let run_stow_command = Command::RunStow {
            pwd: parent_dir.clone(),
            package: "stow_dir".to_string(),
        };
        run_stow_command.invoke(true).unwrap();

        // Verify that the symlink has been created
        let symlink_path = temp_path.join("tmp_file.txt");
        println!("Symlink path: {:?}", symlink_path);
        assert!(symlink_path.exists());
        assert!(symlink_path.is_symlink());

        // Verify that the symlink points to the correct file
        assert_eq!(
            symlink_path.canonicalize().unwrap(),
            file_in_stow_dir.canonicalize().unwrap()
        );
    }
}
