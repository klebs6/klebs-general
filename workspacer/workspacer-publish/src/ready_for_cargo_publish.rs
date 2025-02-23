crate::ix!();

/// Trait for checking if a component is ready for Cargo publishing
#[async_trait]
pub trait ReadyForCargoPublish {

    type Error;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error>;
}

#[async_trait]
impl ReadyForCargoPublish for CrateHandle {

    type Error = CrateError;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error> {

        self.cargo_toml().ready_for_cargo_publish().await?;

        self.check_readme_exists()?;
        self.check_src_directory_contains_valid_files()?;

        Ok(())
    }
}

#[async_trait]
impl<P,H:CrateHandleInterface<P> + ReadyForCargoPublish<Error=CrateError>> ReadyForCargoPublish for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    /// Ensures all crates in the workspace are ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), WorkspaceError> {
        let mut errors = vec![];

        for crate_handle in self {
            if let Err(e) = crate_handle.ready_for_cargo_publish().await {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let errors: Vec<WorkspaceError> = errors.into_iter().map(|x| x.into()).collect();
            Err(WorkspaceError::MultipleErrors(errors))
        }
    }
}

// =============================================
// Test Module #3: ReadyForCargoPublish
// =============================================
#[cfg(test)]
#[disable]
mod test_ready_for_cargo_publish {
    use super::*;
    use workspacer_3p::tokio;

    #[tokio::test]
    async fn test_all_crates_ready() {
        let c1 = MockCrateHandle { crate_path: PathBuf::from("crateA"), publish_ready: true };
        let c2 = MockCrateHandle { crate_path: PathBuf::from("crateB"), publish_ready: true };
        let ws = MockWorkspace::new(MockPath(PathBuf::from("/all_ready")), vec![c1, c2]);

        let result = ws.ready_for_cargo_publish().await;
        assert!(result.is_ok(), "All crates are ready => Ok(())");
    }

    #[tokio::test]
    async fn test_some_crates_not_ready() {
        let c1 = MockCrateHandle { crate_path: PathBuf::from("crateA"), publish_ready: false };
        let c2 = MockCrateHandle { crate_path: PathBuf::from("crateB"), publish_ready: true };
        let ws = MockWorkspace::new(MockPath(PathBuf::from("/some_not_ready")), vec![c1, c2]);

        let result = ws.ready_for_cargo_publish().await;
        match result {
            Err(WorkspaceError::MultipleErrors(errors)) => {
                assert_eq!(errors.len(), 1, "Only c1 fails => 1 error returned");
                println!("Got expected multiple errors: {:?}", errors);
            },
            other => panic!("Expected MultipleErrors, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_no_crates_means_ok() {
        // Edge case: empty workspace => trivially "ready"?
        let ws = MockWorkspace::new(MockPath(PathBuf::from("/empty_ws")), vec![]);
        let result = ws.ready_for_cargo_publish().await;
        assert!(result.is_ok(), "No crates => no failures => Ok(())");
    }

    /// Test ready_for_cargo_publish => success with a minimal crate that has src/main.rs or lib.rs, plus README, plus required fields in Cargo.toml.
    #[tokio::test]
    async fn test_ready_for_cargo_publish_ok() {
        let handle = create_crate_handle_in_temp(
            "publishable_crate",
            "0.1.0",
            true,
            false,
            true,  // create readme
            Some("main"),
        )
        .await;

        // We have: 
        //   [package] with name/version/authors/license
        //   src/main.rs
        //   README.md
        // That should pass the checks
        let result = handle.ready_for_cargo_publish().await;
        assert!(result.is_ok(), "Expected crate to be ready for publish");
    }

    /// Test ready_for_cargo_publish => fails if the required fields in Cargo.toml are missing or if the README is missing, etc.
    #[tokio::test]
    async fn test_ready_for_cargo_publish_fails_missing_readme() {
        // We'll create everything except README.md
        let handle = create_crate_handle_in_temp(
            "not_publishable_crate",
            "0.1.0",
            true,
            false,
            false, // missing readme
            Some("main"),
        )
        .await;

        let result = handle.ready_for_cargo_publish().await;
        assert!(result.is_err(), "Expected an error, missing README.md");
        match result {
            Err(CrateError::FileNotFound { missing_file }) => {
                let missing = missing_file.to_string_lossy();
                assert!(
                    missing.contains("README.md"),
                    "Should mention missing README.md"
                );
            }
            other => panic!("Expected CrateError::FileNotFound, got {other:?}"),
        }
    }
}
