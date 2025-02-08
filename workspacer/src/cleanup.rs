// ---------------- [ File: src/cleanup.rs ]
crate::ix!();

#[async_trait]
impl CleanupWorkspace for Workspace {

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
