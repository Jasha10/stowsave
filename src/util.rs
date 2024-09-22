use std::path::{Path, PathBuf};

/// Finds the common ancestor path between two absolute paths.
pub(super) fn find_common_ancestor(path1: &Path, path2: &Path) -> PathBuf {
    assert!(path1.is_absolute(), "Path1 must be absolute");
    assert!(path2.is_absolute(), "Path2 must be absolute");
    let mut ancestor = PathBuf::new();
    let components1: Vec<_> = path1.components().collect();
    let components2: Vec<_> = path2.components().collect();

    for (c1, c2) in components1.iter().zip(components2.iter()) {
        if c1 == c2 {
            ancestor.push(c1);
        } else {
            break;
        }
    }

    if ancestor.as_os_str().is_empty() {
        PathBuf::from("/")
    } else {
        ancestor
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_find_common_ancestor_same_path() {
        let path = Path::new("/home/user/documents");
        assert_eq!(
            find_common_ancestor(path, path),
            Path::new("/home/user/documents")
        );
    }

    #[test]
    fn test_find_common_ancestor_subpath() {
        let path1 = Path::new("/home/user/documents");
        let path2 = Path::new("/home/user/documents/subfolder");
        assert_eq!(
            find_common_ancestor(path1, path2),
            Path::new("/home/user/documents")
        );
    }

    #[test]
    fn test_find_common_ancestor_different_paths() {
        let path1 = Path::new("/home/user1/documents");
        let path2 = Path::new("/home/user2/pictures");
        assert_eq!(find_common_ancestor(path1, path2), Path::new("/home"));
    }

    #[test]
    fn test_find_common_ancestor_root() {
        let path1 = Path::new("/home/user/documents");
        let path2 = Path::new("/var/log");
        assert_eq!(find_common_ancestor(path1, path2), Path::new("/"));
    }
}
