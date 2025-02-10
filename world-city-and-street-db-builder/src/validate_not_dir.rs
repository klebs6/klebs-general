// ---------------- [ File: src/validate_not_dir.rs ]
// ---------------- [ File: src/validate_not_dir.rs ]
crate::ix!();

/// Ensures the path does not point to an existing directory.
pub fn validate_not_dir(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        let msg = format!("Refusing to create file at {:?}, path is a directory", path);
        error!("{}", msg);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg));
    }
    Ok(())
}

#[cfg(test)]
#[disable]
mod test_validate_not_dir {
    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tracing::error; // adjust if needed

    #[traced_test]
    fn test_directory_returns_error() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let dir_path = temp_dir.path();

        let result = validate_not_dir(dir_path);
        assert!(result.is_err(), "Passing a directory => should return an error");

        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Other, "Kind => Other");
        assert!(
            err.to_string().contains("path is a directory"),
            "Should mention 'path is a directory'. Got: {}",
            err
        );
    }

    #[traced_test]
    fn test_file_path_returns_ok() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test_file.txt");
        
        // Create an empty file
        File::create(&file_path).expect("Failed to create file");
        
        let result = validate_not_dir(&file_path);
        assert!(result.is_ok(), "A file path => should be Ok(())");
    }

    #[traced_test]
    fn test_nonexistent_path_returns_ok() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let nonexistent = temp_dir.path().join("does_not_exist");

        // We do NOT create anything at 'nonexistent'
        let result = validate_not_dir(&nonexistent);
        assert!(result.is_ok(), "A nonexistent path => Ok(())");
    }

    #[traced_test]
    fn test_symlink_to_dir_might_be_error_if_it_is_a_dir() {
        // Depending on your OS and how you handle symlinks, 
        // `path.is_dir()` might evaluate the symlink target. 
        // We'll do a simple approach on Unix. If you're on Windows, adapt.
        #[cfg(unix)]
        {
            let temp_dir = TempDir::new().expect("Failed to create temporary directory");
            let real_dir = temp_dir.path().join("real_dir");
            fs::create_dir(&real_dir).expect("Failed to create real_dir");

            let symlink_path = temp_dir.path().join("dir_link");
            std::os::unix::fs::symlink(&real_dir, &symlink_path).expect("Failed to create symlink");

            // If path.is_dir() follows the symlink => error
            let result = validate_not_dir(&symlink_path);
            assert!(result.is_err(), "Symlink to directory => error if path.is_dir() is true");
        }
    }

    #[traced_test]
    fn test_symlink_to_file_should_ok_or_err_depending_on_target() {
        // Similarly, if the symlink points to a file, the function might treat it as a file => Ok(()),
        // or if it leads to a directory => error. We'll do a file scenario here:
        #[cfg(unix)]
        {
            let temp_dir = TempDir::new().expect("Failed to create temporary directory");
            let real_file = temp_dir.path().join("some_file.txt");
            let _ = File::create(&real_file).expect("Failed to create file");

            let symlink_path = temp_dir.path().join("file_link");
            std::os::unix::fs::symlink(&real_file, &symlink_path)
                .expect("Failed to create symlink to file");
            
            let result = validate_not_dir(&symlink_path);
            assert!(result.is_ok(), "Symlink => path.is_dir() = false => Ok(())");
        }
    }
}
