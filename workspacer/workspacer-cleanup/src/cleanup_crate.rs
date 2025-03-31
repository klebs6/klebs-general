// ---------------- [ File: workspacer-cleanup/src/cleanup_crate.rs ]
crate::ix!();

// 1) Introduce a new trait for cleaning up a single crate:
#[async_trait]
pub trait CleanupCrate {
    async fn cleanup_crate(&self) -> Result<(), WorkspaceError>;
}

#[async_trait]
impl CleanupCrate for CrateHandle {
    async fn cleanup_crate(&self) -> Result<(), WorkspaceError> {
        let crate_path = self.root_dir_path_buf();
        let target_dir = crate_path.join("target");
        let lock_file  = crate_path.join("Cargo.lock");

        info!("cleanup_crate for '{}'", self.name());
        // Remove target/
        if fs::metadata(&target_dir).await.is_ok() {
            fs::remove_dir_all(&target_dir)
                .await
                .map_err(|_| WorkspaceError::DirectoryRemovalError)?;
            info!("Removed '{:?}'", target_dir);
        }

        // Remove Cargo.lock
        if fs::metadata(&lock_file).await.is_ok() {
            fs::remove_file(&lock_file)
                .await
                .map_err(|_| WorkspaceError::FileRemovalError)?;
            info!("Removed '{:?}'", lock_file);
        }

        Ok(())
    }
}
