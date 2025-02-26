// ---------------- [ File: src/find_items.rs ]
crate::ix!();

#[async_trait]
impl<P, H> AsyncFindItems for Workspace<P, H>
where
    // Make sure P can be constructed from a PathBuf
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync,
{
    type Item = H;
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
                crates.push(H::new(&converted).await?);
            }
        }

        Ok(crates)
    }
}

#[cfg(test)]
#[disable]
mod test_async_find_items {
    use super::*;
    use workspacer_3p::tokio::fs;
    use workspacer_3p::tokio;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    // ----------------------------------------------------------------------
    // 1) A mock H that implements `CrateHandleInterface<P>` minimally
    // ----------------------------------------------------------------------
    #[derive(Debug, Clone)]
    struct MockCrateHandle {
        path: PathBuf,
        fail_on_new: bool, // if we want to simulate a crate handle constructor error
    }

    // We'll define the trait bounds for P => AsRef<Path>, etc.
    #[async_trait::async_trait]
    impl<P> CrateHandleInterface<P> for MockCrateHandle
    where
        for<'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait
    {
        // Provide stubs for any required trait methods if needed.
    }

    // We'll define `new` for the `H::new(&converted).await?` usage in `find_items`:
    #[async_trait::async_trait]
    impl<P> AsyncTryFrom<P> for MockCrateHandle
    where
        for<'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait,
    {
        type Error = WorkspaceError;

        async fn new(path: &P) -> Result<Self, Self::Error> {
            // If we want to simulate an error, read from something, or do a fail_on_new check
            let path_buf = path.as_ref().to_path_buf();
            // For demonstration, we might store a special marker file or a naming convention
            // to check if we want to fail:
            if path_buf.to_string_lossy().contains("fail-crate") {
                // simulate an error
                return Err(WorkspaceError::InvalidWorkspace {
                    invalid_workspace_path: path_buf,
                });
            }

            Ok(Self {
                path: path_buf,
                fail_on_new: false,
            })
        }
    }

    // ----------------------------------------------------------------------
    // 2) Use the real `find_items` from your code, but with `MockCrateHandle`
    // ----------------------------------------------------------------------
    // We'll define a type alias or minimal "Workspace-like" approach:
    type MyItem = MockCrateHandle;
    type MyError = WorkspaceError;

    // Re-export the trait for clarity:
    use crate::AsyncFindItems; // or wherever you define `AsyncFindItems`

    // We'll test the static method: `Workspace<P,H>::find_items(path: &Path) -> ...`
    // but it’s actually the trait's default: `find_items`

    // ----------------------------------------------------------------------
    // 3) Tests
    // ----------------------------------------------------------------------

    /// Scenario A: The directory doesn't exist => read_dir error => `DirectoryError::ReadDirError`
    #[tokio::test]
    async fn test_directory_not_found() {
        let missing_path = PathBuf::from("/this/path/should/not/exist-12345");
        let result = Workspace::<PathBuf, MyItem>::find_items(&missing_path).await;
        match result {
            Err(WorkspaceError::DirectoryError(dir_err)) => {
                match dir_err {
                    DirectoryError::ReadDirError { io } => {
                        println!("Got expected read_dir error: {:?}", io);
                    }
                    other => panic!("Expected ReadDirError, got {:?}", other),
                }
            }
            other => panic!("Expected DirectoryError, got {:?}", other),
        }
    }

    /// Scenario B: The directory is empty => no crates found => Ok(empty_vec).
    #[tokio::test]
    async fn test_empty_directory() {
        let tmp = tempdir().expect("Failed to create temp dir");
        let path = tmp.path();

        let crates = Workspace::<PathBuf, MyItem>::find_items(path).await
            .expect("Should succeed reading an empty directory");
        assert!(crates.is_empty(), "No subdirectories => no crates found");
    }

    /// Scenario C: Single subdirectory with a `Cargo.toml`.
    /// => we expect exactly one item returned.
    #[tokio::test]
    async fn test_single_crate() {
        let tmp = tempdir().unwrap();
        let path = tmp.path();

        // Make a subdir with a Cargo.toml
        let crate_dir = path.join("mycrate");
        fs::create_dir(&crate_dir).await.unwrap();
        fs::write(crate_dir.join("Cargo.toml"), b"[package]\nname=\"test\"").await.unwrap();

        // Now call find_items
        let crates = Workspace::<PathBuf, MyItem>::find_items(path).await
            .expect("Should find one crate");
        assert_eq!(crates.len(), 1, "One subdir with Cargo.toml => 1 crate");
        assert_eq!(crates[0].path, crate_dir, "Crate path should match subdir");
    }

    /// Scenario D: Multiple subdirectories, some with Cargo.toml, some without => we only get crates from those with Cargo.toml.
    #[tokio::test]
    async fn test_multiple_subdirs_some_crates() {
        let tmp = tempdir().unwrap();
        let path = tmp.path();

        // Subdir 1 => has Cargo.toml
        let crate_a = path.join("crateA");
        fs::create_dir(&crate_a).await.unwrap();
        fs::write(crate_a.join("Cargo.toml"), b"[package]\nname=\"crateA\"").await.unwrap();

        // Subdir 2 => no Cargo.toml => not a crate
        let crate_b = path.join("not_a_crate");
        fs::create_dir(&crate_b).await.unwrap();
        // (no Cargo.toml here)

        // Subdir 3 => has Cargo.toml
        let crate_c = path.join("crateC");
        fs::create_dir(&crate_c).await.unwrap();
        fs::write(crate_c.join("Cargo.toml"), b"[package]\nname=\"crateC\"").await.unwrap();

        let crates = Workspace::<PathBuf, MyItem>::find_items(path).await
            .expect("Should succeed scanning directories");
        assert_eq!(crates.len(), 2, "We have 2 subdirs with Cargo.toml");
        
        // We can check the specific paths in any order
        let found_paths: Vec<_> = crates.into_iter().map(|c| c.path).collect();
        assert!(found_paths.contains(&crate_a));
        assert!(found_paths.contains(&crate_c));
    }

    /// Scenario E: One subdirectory triggers `H::new(...)` to fail => entire find_items fails with that error.
    #[tokio::test]
    async fn test_crate_handle_construction_failure() {
        let tmp = tempdir().unwrap();
        let path = tmp.path();

        // We'll create a subdir named "fail-crate" that triggers our mock handle to fail
        let fail_dir = path.join("fail-crate");
        fs::create_dir(&fail_dir).await.unwrap();
        fs::write(fail_dir.join("Cargo.toml"), b"[package]\nname=\"fail-crate\"").await.unwrap();

        let result = Workspace::<PathBuf, MyItem>::find_items(path).await;
        match result {
            Err(WorkspaceError::InvalidWorkspace { invalid_workspace_path }) => {
                assert_eq!(invalid_workspace_path, fail_dir, "Should fail on that subdir path");
            }
            other => panic!("Expected InvalidWorkspace error, got {:?}", other),
        }
    }

    /// Scenario F: If `next_entry()` fails mid-iteration => we get `DirectoryError::GetNextEntryError`.
    /// Hard to simulate on normal OS, but we can define a test on restricted permissions, or a mock approach.
    #[tokio::test]
    async fn test_directory_next_entry_error() {
        // Realistically, you'd need something like special permissions or a failing FS to see it,
        // or you can do a manual mocking approach. We'll demonstrate the real approach is OS-specific. 
        // We'll just show the structure:
        let tmp = tempdir().unwrap();
        let path = tmp.path();

        // create a subdir with restricted perms or something
        // On Unix, we might do:
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let restricted = path.join("restricted_dir");
            fs::create_dir(&restricted).await.unwrap();
            let mut perms = tokio::fs::metadata(&restricted).await.unwrap().permissions();
            perms.set_mode(0o000); // no permissions
            tokio::fs::set_permissions(&restricted, perms).await.unwrap();

            let result = Workspace::<PathBuf, MyItem>::find_items(path).await;
            match result {
                Err(WorkspaceError::DirectoryError(DirectoryError::GetNextEntryError { io })) => {
                    println!("Got expected error: can't read restricted dir entry: {:?}", io);
                }
                other => panic!("Expected GetNextEntryError, got {:?}", other),
            }

            // reset perms so we can clean up
            let mut perms = tokio::fs::metadata(&restricted).await.unwrap().permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&restricted, perms).await.unwrap();
        }
        #[cfg(not(unix))]
        {
            // We can't easily do it on Windows in a stable manner, so we might skip or do a mock approach.
            println!("Skipping restricted dir test on non-Unix OS");
        }
    }
}
