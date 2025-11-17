use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Calculate the size of a directory recursively
pub fn calculate_dir_size(path: &Path) -> Result<(u64, usize)> {
    let mut total_size = 0u64;
    let mut file_count = 0usize;

    if !path.exists() {
        return Ok((0, 0));
    }

    // Handle symlinks - don't follow them to avoid loops
    if path.is_symlink() {
        return Ok((0, 0));
    }

    if path.is_file() {
        let metadata = fs::metadata(path).context("Failed to read file metadata")?;
        return Ok((metadata.len(), 1));
    }

    if path.is_dir() {
        let entries = fs::read_dir(path).context("Failed to read directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Skip symlinks
            if path.is_symlink() {
                continue;
            }

            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    total_size += metadata.len();
                    file_count += 1;
                }
            } else if path.is_dir() {
                let (size, count) = calculate_dir_size(&path)?;
                total_size += size;
                file_count += count;
            }
        }
    }

    Ok((total_size, file_count))
}

/// Remove a directory recursively with platform-specific handling
pub fn remove_dir_all(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // On Windows, we might need to handle long paths and readonly files
    #[cfg(target_os = "windows")]
    {
        // Try to remove readonly attribute if present
        if let Ok(metadata) = fs::metadata(path) {
            let mut permissions = metadata.permissions();
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
    }

    fs::remove_dir_all(path)
        .with_context(|| format!("Failed to remove directory: {}", path.display()))
}

/// Check if we have permission to delete a directory
pub fn can_delete(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    // Try to read the directory to check permissions
    if let Ok(metadata) = fs::metadata(path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            // Check if we have write permission
            return permissions.mode() & 0o200 != 0;
        }

        #[cfg(not(unix))]
        {
            // On Windows, check if it's not readonly
            return !metadata.permissions().readonly();
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_dir_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        let (size, count) = calculate_dir_size(temp_dir.path()).unwrap();
        assert_eq!(size, 13); // "Hello, World!" is 13 bytes
        assert_eq!(count, 1);
    }

    #[test]
    fn test_calculate_dir_size_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir).unwrap();

        fs::write(temp_dir.path().join("file1.txt"), "12345").unwrap();
        fs::write(nested_dir.join("file2.txt"), "67890").unwrap();

        let (size, count) = calculate_dir_size(temp_dir.path()).unwrap();
        assert_eq!(size, 10); // 5 + 5 bytes
        assert_eq!(count, 2);
    }

    #[test]
    fn test_remove_dir_all() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("to_remove");
        fs::create_dir(&test_dir).unwrap();
        fs::write(test_dir.join("file.txt"), "content").unwrap();

        assert!(test_dir.exists());
        remove_dir_all(&test_dir).unwrap();
        assert!(!test_dir.exists());
    }

    #[test]
    fn test_can_delete() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("deletable");
        fs::create_dir(&test_dir).unwrap();

        assert!(can_delete(&test_dir));
        assert!(!can_delete(&PathBuf::from("/nonexistent/path")));
    }
}
