// ---------------- [ File: workspacer-workspace/src/find_items.rs ]
crate::ix!();

#[async_trait]
impl<P, H> AsyncFindItems for Workspace<P, H>
where
    // Make sure P can be constructed from a PathBuf
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync,
{
    type Item = Arc<AsyncMutex<H>>;
    type Error = WorkspaceError;

    /// Asynchronously finds all the crates in the workspace
    async fn find_items(path: &Path) -> Result<Vec<Self::Item>, Self::Error> {
        let mut crates = vec![];

        let mut entries = fs::read_dir(path)
            .await
            .map_err(|e| DirectoryError::ReadDirError { io: e.into() })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DirectoryError::GetNextEntryError { io: e.into() })?
        {
            let crate_path = entry.path();

            // If there's a Cargo.toml here, we consider it a crate:
            if fs::metadata(crate_path.join("Cargo.toml"))
                .await
                .is_ok()
            {
                // Convert crate_path (PathBuf) into your generic P:
                let converted: P = crate_path.into();
                crates.push(Arc::new(AsyncMutex::new(H::new(&converted).await?)));
            }
        }

        Ok(crates)
    }
}

#[cfg(test)]
mod test_async_find_items {
    use super::*;
    
    /// We'll define a type alias for convenience
    type MyWorkspace = Workspace<PathBuf, CrateHandle>;
    
    ///
    /// Creates a minimal directory with one or more sub-crates, then calls
    /// the real `Workspace::<PathBuf,CrateHandle>::find_items(...)`.
    ///
    #[traced_test]
    async fn test_find_items_in_directory() {
        info!("Starting test_find_items_in_directory");
        let tmp = tempdir().expect("Failed to create temp dir");
        let root = tmp.path().to_path_buf();
        
        // We'll create two subdirs, each with a Cargo.toml, representing 2 crates
        let crate_a = root.join("crateA");
        fs::create_dir(&crate_a).await.unwrap();
        fs::write(crate_a.join("Cargo.toml"), b"[package]\nname=\"crateA\"").await.unwrap();

        let crate_b = root.join("crateB");
        fs::create_dir(&crate_b).await.unwrap();
        fs::write(crate_b.join("Cargo.toml"), b"[package]\nname=\"crateB\"").await.unwrap();

        // Now call the real find_items:
        let crates_found = MyWorkspace::find_items(&root).await
            .expect("Should find both crates in subdirectories");

        // We expect 2 crates
        assert_eq!(crates_found.len(), 2, "Should find exactly 2 crates");
        let mut paths = Vec::new();
        for c in crates_found.iter() {
            paths.push(c.lock().await.root_dir_path_buf());
        }
        assert!(paths.contains(&crate_a));
        assert!(paths.contains(&crate_b));
        
        info!("test_find_items_in_directory passed");
    }

    ///
    /// Tests that a directory with no sub-crates yields an empty result
    ///
    #[traced_test]
    async fn test_find_items_in_empty_directory() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().to_path_buf();

        let crates_found = MyWorkspace::find_items(&root).await
            .expect("Reading an empty dir should not error");
        assert_eq!(crates_found.len(), 0, "No subdirs => no crates found");
    }

    ///
    /// Tests that if reading the directory fails entirely, we get a `DirectoryError::ReadDirError`.
    /// We rely on a path that likely doesn't exist or is forbidden.
    ///
    #[traced_test]
    async fn test_find_items_directory_not_found() {
        let bad_path = PathBuf::from("/this/path/should/not/exist-12345");
        let result = MyWorkspace::find_items(&bad_path).await;
        assert!(result.is_err(), "Should fail on non-existent directory");
        
        match result.err().unwrap() {
            WorkspaceError::DirectoryError(DirectoryError::ReadDirError { .. }) => {
                info!("Got expected DirectoryError::ReadDirError for missing directory");
            }
            other => {
                panic!("Expected DirectoryError::ReadDirError, got: {:?}", other);
            }
        }
    }
}
