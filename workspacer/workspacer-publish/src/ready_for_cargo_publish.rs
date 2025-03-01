// ---------------- [ File: src/ready_for_cargo_publish.rs ]
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

        let toml = self.cargo_toml();

        toml.ready_for_cargo_publish().await?;

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

#[cfg(test)]
mod test_ready_for_cargo_publish {
    use super::*;

    #[traced_test]
    async fn test_all_crates_ready() {
        // Create two crates in a mock workspace, both fully configured (license, src, README, etc.)
        let crate_configs = vec![
            CrateConfig::new("crateA")
                .with_readme()
                .with_src_files()
                .with_test_files(),
            CrateConfig::new("crateB")
                .with_readme()
                .with_src_files()
                .with_test_files(),
        ];
        let mock_ws_dir = create_mock_workspace(crate_configs)
            .await
            .expect("failed to create mock workspace");

        // Build a real Workspace<..., CrateHandle> from the newly created directory
        let ws = Workspace::<PathBuf, CrateHandle>::new(&mock_ws_dir)
            .await
            .expect("failed to parse workspace from mock dir");

        // Now call ready_for_cargo_publish on that workspace
        let result = ws.ready_for_cargo_publish().await;
        assert!(result.is_ok(), "All crates are ready => Ok(())");
    }

    #[traced_test]
    async fn test_some_crates_not_ready() {
        // One crate is missing a README, so it won't be "ready" for publishing.
        // Another crate is fully ready.
        let crate_configs = vec![
            CrateConfig::new("crateA")
                // No readme => fails
                .with_src_files()
                .with_test_files(),
            CrateConfig::new("crateB")
                // fully ok
                .with_readme()
                .with_src_files()
                .with_test_files(),
        ];
        let mock_ws_dir = create_mock_workspace(crate_configs)
            .await
            .expect("failed to create mock workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&mock_ws_dir)
            .await
            .expect("failed to parse workspace from mock dir");

        let result = ws.ready_for_cargo_publish().await;
        match result {
            // We expect a MultipleErrors with 1 failing crate
            Err(WorkspaceError::MultipleErrors(errors)) => {
                assert_eq!(errors.len(), 1, "Only one crate fails => 1 error returned");
                println!("Got expected multiple errors: {:?}", errors);
            }
            other => panic!("Expected MultipleErrors, got {:?}", other),
        }
    }

    #[traced_test]
    async fn test_no_crates_means_ok() {
        // An empty workspace => trivially "no crates => no errors"
        let mock_ws_dir = create_mock_workspace(vec![])
            .await
            .expect("failed to create mock workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&mock_ws_dir)
            .await
            .expect("failed to parse workspace from empty dir");

        let result = ws.ready_for_cargo_publish().await;
        assert!(result.is_ok(), "No crates => no failures => Ok(())");
    }

    /// This test checks a single crate that should pass all publishing checks:
    /// - Has Cargo.toml with required fields,
    /// - Has README.md,
    /// - Has src files,
    /// - Valid version field, etc.
    #[traced_test]
    async fn test_ready_for_cargo_publish_ok() {
        let crate_configs = vec![
            CrateConfig::new("publishable_crate")
                .with_readme()
                .with_src_files()
                .with_test_files(),
        ];
        let mock_ws_dir = create_mock_workspace(crate_configs)
            .await
            .expect("failed to create mock workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&mock_ws_dir)
            .await
            .expect("failed to parse workspace");

        // We should pass all checks
        let result = ws.ready_for_cargo_publish().await;
        assert!(result.is_ok(), "Expected crate to be ready for publish");
    }

    /// This test simulates a crate that lacks a README, so `ready_for_cargo_publish` should fail.
    #[traced_test]
    async fn test_ready_for_cargo_publish_fails_missing_readme() {
        // We'll create everything except a README for this single crate
        let crate_configs = vec![
            CrateConfig::new("not_publishable_crate")
                .with_src_files()
                .with_test_files(),
        ];
        let mock_ws_dir = create_mock_workspace(crate_configs)
            .await
            .expect("failed to create mock workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&mock_ws_dir)
            .await
            .expect("failed to parse workspace");

        let result = ws.ready_for_cargo_publish().await;
        assert!(result.is_err(), "Expected an error due to missing README.md");
        match result {
            Err(WorkspaceError::MultipleErrors(mut errs)) if !errs.is_empty() => {
                // Typically you'll see CrateError::FileNotFound { missing_file: ... } or similar.
                let first_err = errs.remove(0);
                println!("Got expected error: {:?}", first_err);
                // If you want to specifically check for a missing README, you can match it:
                if let WorkspaceError::CrateError(CrateError::FileNotFound { missing_file }) = first_err {
                    assert!(
                        missing_file.to_string_lossy().contains("README.md"),
                        "Should mention missing README.md in the error"
                    );
                } else {
                    panic!("Expected FileNotFound for missing README, got: {:?}", first_err);
                }
            }
            other => panic!("Expected multiple errors for missing readme, got: {:?}", other),
        }
    }
}
