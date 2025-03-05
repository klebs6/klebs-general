// ---------------- [ File: workspacer-cleanup/src/cleanup.rs ]
crate::ix!();

#[async_trait]
pub trait CleanupWorkspace {

    async fn cleanup_workspace(&self) -> Result<(), WorkspaceError>;
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> CleanupWorkspace for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    /// Asynchronously cleans up unnecessary files and directories in the workspace.
    async fn cleanup_workspace(&self) -> Result<(), WorkspaceError> {

        // Directories and files to clean up
        let dirs_to_clean  = vec![self.as_ref().join("target")];
        let files_to_clean = vec![self.as_ref().join("Cargo.lock")];

        // Remove directories
        for dir in dirs_to_clean {
            if fs::metadata(&dir).await.is_ok() {
                fs::remove_dir_all(&dir).await.map_err(|_| WorkspaceError::DirectoryRemovalError)?;
            }
        }

        // Remove files
        for file in files_to_clean {
            if fs::metadata(&file).await.is_ok() {
                fs::remove_file(&file).await.map_err(|_| WorkspaceError::FileRemovalError)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_cleanup_workspace {
    use super::*;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;
    use tokio::fs;
    use workspacer_3p::async_trait; 
    // or however you're importing async_trait, etc.

    // -----------------------------------------------------------------------
    // 1) A minimal MockWorkspace that implements AsRef<Path> and references a
    //    temp directory. We'll also implement the CleanupWorkspace trait if needed.
    // -----------------------------------------------------------------------

    #[derive(Debug)]
    struct MockWorkspace {
        root_dir: PathBuf,
    }

    impl MockWorkspace {
        fn new(path: PathBuf) -> Self {
            Self { root_dir: path }
        }
    }

    impl AsRef<Path> for MockWorkspace {
        fn as_ref(&self) -> &Path {
            &self.root_dir
        }
    }

    // We'll also implement the CleanupWorkspace trait if your code
    // requires the workspace itself to implement it:
    #[async_trait]
    impl CleanupWorkspace for MockWorkspace {
        async fn cleanup_workspace(&self) -> Result<(), WorkspaceError> {
            // We'll copy the logic from your snippet or call the same function:
            // In real usage, if you have a workspace type that references the same code,
            // you can simply delegate or replicate the snippet:

            let dirs_to_clean = vec![self.as_ref().join("target")];
            let files_to_clean = vec![self.as_ref().join("Cargo.lock")];

            // Remove directories
            for dir in dirs_to_clean {
                if fs::metadata(&dir).await.is_ok() {
                    fs::remove_dir_all(&dir)
                        .await
                        .map_err(|_| WorkspaceError::DirectoryRemovalError)?;
                }
            }

            // Remove files
            for file in files_to_clean {
                if fs::metadata(&file).await.is_ok() {
                    fs::remove_file(&file)
                        .await
                        .map_err(|_| WorkspaceError::FileRemovalError)?;
                }
            }

            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // 2) Test Cases
    // -----------------------------------------------------------------------

    /// 1) If neither `target/` nor `Cargo.lock` exist, cleanup_workspace should succeed with no errors.
    #[tokio::test]
    async fn test_cleanup_with_no_target_or_cargo_lock() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let workspace = MockWorkspace::new(tmp_dir.path().to_path_buf());

        // The directory is empty => no target/, no Cargo.lock
        // Call cleanup
        let result = workspace.cleanup_workspace().await;
        assert!(result.is_ok(), "Cleanup should succeed if nothing to remove");
    }

    /// 2) If `target/` exists, it should be removed by cleanup.
    #[tokio::test]
    async fn test_cleanup_removes_target_directory() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let workspace = MockWorkspace::new(tmp_dir.path().to_path_buf());

        // Create `target` dir
        let target_dir = workspace.as_ref().join("target");
        fs::create_dir_all(&target_dir).await.expect("Failed to create target dir");
        // Optionally put some sub-files in there
        let sub_file = target_dir.join("some_file.txt");
        fs::write(&sub_file, b"dummy").await.expect("Failed to write sub file");

        // Confirm it exists
        let meta = fs::metadata(&target_dir).await;
        assert!(meta.is_ok(), "target/ directory should exist before cleanup");

        // Now run cleanup
        workspace.cleanup_workspace().await.expect("Cleanup should succeed");

        // Confirm it's removed
        let meta_after = fs::metadata(&target_dir).await;
        assert!(meta_after.is_err(), "target/ should be removed by cleanup");
    }

    /// 3) If `Cargo.lock` exists, it should be removed.
    #[tokio::test]
    async fn test_cleanup_removes_cargo_lock_file() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let workspace = MockWorkspace::new(tmp_dir.path().to_path_buf());

        // Create Cargo.lock
        let lock_path = workspace.as_ref().join("Cargo.lock");
        fs::write(&lock_path, b"dummy lock content").await.expect("Failed to write Cargo.lock");

        // Confirm it exists
        let meta = fs::metadata(&lock_path).await;
        assert!(meta.is_ok(), "Cargo.lock should exist");

        // cleanup
        workspace.cleanup_workspace().await.expect("Cleanup should succeed");

        // confirm removed
        let meta_after = fs::metadata(&lock_path).await;
        assert!(meta_after.is_err(), "Cargo.lock should be removed");
    }

    /// 4) If both `target/` and `Cargo.lock` exist, we remove both.
    #[tokio::test]
    async fn test_cleanup_removes_both_target_and_cargo_lock() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let workspace = MockWorkspace::new(tmp_dir.path().to_path_buf());

        // create target
        let target_dir = workspace.as_ref().join("target");
        fs::create_dir_all(&target_dir).await.expect("create target dir");
        // create Cargo.lock
        let lock_path = workspace.as_ref().join("Cargo.lock");
        fs::write(&lock_path, b"dummy").await.expect("write cargo.lock");

        // call cleanup
        workspace.cleanup_workspace().await.expect("cleanup ok");

        // confirm both removed
        let target_meta = fs::metadata(&target_dir).await;
        let lock_meta = fs::metadata(&lock_path).await;
        assert!(target_meta.is_err(), "target/ removed");
        assert!(lock_meta.is_err(), "Cargo.lock removed");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_cleanup_failure_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let tmp_dir = tempdir().expect("create tempdir");
        let workspace = MockWorkspace::new(tmp_dir.path().to_path_buf());

        // 1) Create Cargo.lock
        let lock_path = workspace.as_ref().join("Cargo.lock");
        fs::write(&lock_path, b"some content").await.unwrap();

        // 2) Remove *directory* write permission so we can't remove files inside it
        let mut perms = std::fs::metadata(tmp_dir.path()).unwrap().permissions();
        // Typically, `0o500` => user read & execute, no write
        // (or you can do 0o555 for read+execute for user/group/other)
        perms.set_mode(0o500);
        std::fs::set_permissions(tmp_dir.path(), perms).expect("set perms on directory");

        // 3) Now attempt cleanup; removing Cargo.lock should fail due to no write permission
        let result = workspace.cleanup_workspace().await;
        assert!(result.is_err(), "Should fail if we can't remove Cargo.lock");
        match result {
            Err(WorkspaceError::FileRemovalError) => {
                // expected
            }
            other => panic!("Expected FileRemovalError, got {:?}", other),
        }
    }

    /// 6) If we call cleanup multiple times, subsequent calls should succeed even if there’s nothing to remove.
    #[tokio::test]
    async fn test_cleanup_idempotent() {
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let workspace = MockWorkspace::new(tmp_dir.path().to_path_buf());

        // Create target + cargo.lock
        fs::create_dir_all(workspace.as_ref().join("target")).await.unwrap();
        fs::write(workspace.as_ref().join("Cargo.lock"), b"xyz").await.unwrap();

        // First cleanup
        workspace.cleanup_workspace().await.unwrap();
        // Everything removed

        // Second cleanup => no target/ or Cargo.lock => should not fail
        workspace.cleanup_workspace().await.unwrap();
    }
}
