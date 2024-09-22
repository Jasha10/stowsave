use std::path::Path;

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
enum StowSaveError {
    #[error("Path '{0}' does not exist")]
    PathDoesNotExist(String),
    #[error("Path '{0}' is a symlink. Cannot save symlinks.")]
    PathIsSymlink(String),
    #[error("Directory '{0}' does not exist or is not a directory")]
    InvalidStowDirectory(String),
    #[error("Path '{0}' already exists in the stow directory")]
    PathAlreadyExists(String),
}

pub(super) fn path_to_save_exists(path_to_save: &Path) -> Result<()> {
    if !path_to_save.exists() {
        return Err(
            StowSaveError::PathDoesNotExist(path_to_save.to_string_lossy().into_owned()).into(),
        );
    }
    Ok(())
}
pub(super) fn path_to_save_is_not_symlink(path_to_save: &Path) -> Result<()> {
    if path_to_save.is_symlink() {
        return Err(
            StowSaveError::PathIsSymlink(path_to_save.to_string_lossy().into_owned()).into(),
        );
    }
    Ok(())
}
pub(super) fn stow_directory_exists(stow_dir: &Path) -> Result<()> {
    if !stow_dir.is_dir() {
        return Err(
            StowSaveError::InvalidStowDirectory(stow_dir.to_string_lossy().into_owned()).into(),
        );
    }
    Ok(())
}
pub(super) fn target_path_does_not_exist(target_path: &Path) -> Result<()> {
    if target_path.exists() {
        return Err(
            StowSaveError::PathAlreadyExists(target_path.to_string_lossy().into_owned()).into(),
        );
    }
    Ok(())
}

/// The stow directory should be precisely two generators below the common ancestor
pub(super) fn stow_directory_is_grandchild_of_common_ancestor(
    stow_dir: &Path,
    common_ancestor: &Path,
) -> Result<()> {
    if stow_dir.strip_prefix(common_ancestor)?.ancestors().count() != 3 {
        return Err(anyhow::anyhow!(
            "The stow directory must be a grandchild of the common ancestor"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::os::unix::fs as unix_fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_stow_directory_is_grandchild_of_common_ancestor() {
        let common_ancestor = Path::new("/home/user");

        // bad case
        let stow_dir = Path::new("/home/user/stow");
        assert!(
            stow_directory_is_grandchild_of_common_ancestor(stow_dir, common_ancestor).is_err()
        );

        // good case
        let stow_dir = Path::new("/home/user/dotfiles/stow");
        assert!(stow_directory_is_grandchild_of_common_ancestor(stow_dir, common_ancestor).is_ok());
    }

    #[test]
    fn test_path_to_save_exists() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("existing_file");
        File::create(&existing_file).unwrap();

        assert!(path_to_save_exists(&existing_file).is_ok());
        assert!(path_to_save_exists(&temp_dir.path().join("non_existent_file")).is_err());
    }

    #[test]
    fn test_path_to_save_is_not_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let regular_file = temp_dir.path().join("regular_file");
        File::create(&regular_file).unwrap();

        let symlink = temp_dir.path().join("symlink");
        unix_fs::symlink(&regular_file, &symlink).unwrap();

        assert!(path_to_save_is_not_symlink(&regular_file).is_ok());
        assert!(path_to_save_is_not_symlink(&symlink).is_err());
    }

    #[test]
    fn test_stow_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_dir = temp_dir.path().join("non_existent_dir");
        let file_path = temp_dir.path().join("file");
        File::create(&file_path).unwrap();

        assert!(stow_directory_exists(temp_dir.path()).is_ok());
        assert!(stow_directory_exists(&non_existent_dir).is_err());
        assert!(stow_directory_exists(&file_path).is_err());
    }

    #[test]
    fn test_target_path_does_not_exist() {
        let temp_dir = TempDir::new().unwrap();
        let existing_file = temp_dir.path().join("existing_file");
        File::create(&existing_file).unwrap();

        let non_existent_file = temp_dir.path().join("non_existent_file");

        assert!(target_path_does_not_exist(&non_existent_file).is_ok());
        assert!(target_path_does_not_exist(&existing_file).is_err());
    }
}
