// ---------------- [ File: workspacer-rebuild-or-test/src/rebuild_or_test.rs ]
crate::ix!();

#[async_trait]
pub trait RebuildOrTest {

    type Error;

    async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error>;
}

#[async_trait]
impl RebuildOrTest for CrateHandle {
    type Error = CrateError;

    ///
    /// Runs `cargo build` and then `cargo test` in this crate's directory.
    /// Logs success or error details; returns `Ok(())` if both succeed.
    ///
    async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error> {

        info!("Running cargo build for crate at: {}", self.as_ref().display());

        let crate_path = self.as_ref();

        let mut build_cmd = Command::new("cargo");
        build_cmd.arg("build").current_dir(&crate_path);

        let output = runner.run_command(build_cmd).await??;

        if !output.status.success() {
            error!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(CrateError::from(BuildError::BuildFailed {
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }));
        }

        info!("Rebuild succeeded, running tests...");

        let mut test_cmd = Command::new("cargo");
        test_cmd.arg("test").current_dir(&crate_path);

        let test_output = runner.run_command(test_cmd).await??;

        if !test_output.status.success() {
            let stdout = Some(String::from_utf8_lossy(&test_output.stdout).to_string());
            let stderr = Some(String::from_utf8_lossy(&test_output.stderr).to_string());

            error!("Tests failed: {:#?}", stderr);
            return Err(CrateError::from(TestFailure::UnknownError { stdout, stderr }));
        }

        info!("All tests passed successfully for crate at {}.", self.as_ref().display());
        Ok(())
    }
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> RebuildOrTest for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error> {

        let workspace_path = self.as_ref();

        info!("Running cargo build...");

        let mut build_cmd = Command::new("cargo");
        build_cmd.arg("build").current_dir(&workspace_path);

        let output = runner.run_command(build_cmd).await??;

        if !output.status.success() {
            error!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
            return Err(WorkspaceError::from(BuildError::BuildFailed {
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }));
        }

        info!("Rebuild succeeded, running tests...");

        let mut test_cmd = Command::new("cargo");
        test_cmd.arg("test").current_dir(&workspace_path);

        let test_output = runner.run_command(test_cmd).await??;

        if !test_output.status.success() {
            let stdout = Some(String::from_utf8_lossy(&test_output.stdout).to_string());
            let stderr = Some(String::from_utf8_lossy(&test_output.stderr).to_string());

            error!("Tests failed: {:#?}", stderr);
            return Err(WorkspaceError::from(TestFailure::UnknownError { stdout, stderr }));
        }

        info!("Tests passed successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod test_rebuild_or_test_with_mock {
    use super::*;

    /// This test uses `create_mock_workspace` from `workspacer-mock` to create a
    /// temporary workspace on disk with one crate that should build + test successfully.
    ///
    /// Then we wrap it in a "Workspace" (or any struct implementing `RebuildOrTest`),
    /// and verify that `workspace.rebuild_or_test(...)` runs `cargo build` and `cargo test`
    /// without failure.
    #[traced_test]
    async fn test_rebuild_or_test_succeeds_with_mock() -> Result<(), WorkspaceError> {
        // 1) Create a mock workspace with one crate that compiles & passes tests:
        let crate_configs = vec![
            CrateConfig::new("working_crate")
                .with_readme()
                .with_src_files()
                .with_test_files(),
        ];
        let mock_workspace_path = create_mock_workspace(crate_configs).await?;

        // 2) Construct a real or mock "Workspace" that implements `RebuildOrTest`.
        //    For example, if you have `Workspace::<PathBuf, CrateHandle>::new(...)`:
        //    let workspace = Workspace::<PathBuf, CrateHandle>::new(&mock_workspace_path).await?;
        //
        //    Or if you prefer a minimal struct that does the same logic:
        let mock_ws = MockWorkspace { path: mock_workspace_path.clone() };

        // 3) Run rebuild_or_test using the default runner
        let runner = DefaultCommandRunner;
        let result = mock_ws.rebuild_or_test(&runner).await;

        // 4) Confirm success
        assert!(result.is_ok(), "Expected mock workspace to build & test successfully");

        Ok(())
    }

    /// This test uses `create_mock_workspace` to make a crate with *broken* Rust code,
    /// ensuring that `cargo build` fails with a `BuildError`.
    #[traced_test]
    async fn test_rebuild_or_test_build_fails_with_mock() -> Result<(), WorkspaceError> {
        // 1) Create a mock workspace with one crate, but sabotage its src so it won’t compile.
        let crate_configs = vec![
            CrateConfig::new("broken_crate")
                .with_readme()
                .with_src_files()
                .with_test_files(),
        ];
        let mock_workspace_path = create_mock_workspace(crate_configs).await?;

        // 2) Overwrite the crate's src/lib.rs with invalid code
        let lib_rs = mock_workspace_path.join("broken_crate").join("src").join("lib.rs");
        tokio::fs::write(&lib_rs, b"fn main(){ let x:DoesNotExist = 123; }").await
            .expect("Failed to write broken code to lib.rs");

        // 3) Rebuild or test => should fail on build
        let mock_ws = MockWorkspace { path: mock_workspace_path };
        let runner = DefaultCommandRunner;
        let result = mock_ws.rebuild_or_test(&runner).await;

        match result {
            Err(WorkspaceError::BuildError(BuildError::BuildFailed { stderr })) => {
                info!("Received expected build failure: stderr:\n{}", stderr);
            }
            Ok(_) => {
                panic!("Expected build to fail, but got success");
            }
            other => {
                panic!("Expected BuildError::BuildFailed, got: {:?}", other);
            }
        }

        Ok(())
    }

    /// This test creates a crate that *compiles* but has a failing test, to confirm
    /// that we get a `TestFailure::UnknownError` instead of a build error.
    #[traced_test]
    async fn test_rebuild_or_test_test_fails_with_mock() -> Result<(), WorkspaceError> {
        // 1) Create a mock workspace with a passing crate, then sabotage the test code.
        let crate_configs = vec![
            CrateConfig::new("test_fail_crate")
                .with_readme()
                .with_src_files()
                .with_test_files(),
        ];
        let mock_workspace_path = create_mock_workspace(crate_configs).await?;

        // 2) Overwrite the crate’s test file with a failing test
        let test_rs = mock_workspace_path.join("test_fail_crate").join("tests").join("test.rs");
        let failing_test = r#"
            #[test]
            fn test_fail() {
                assert_eq!(2+2, 999);
            }
        "#;
        tokio::fs::write(&test_rs, failing_test).await
            .expect("Failed to write failing test to test.rs");

        // 3) Rebuild or test => should pass build but fail tests
        let mock_ws = MockWorkspace { path: mock_workspace_path };
        let runner = DefaultCommandRunner;
        let result = mock_ws.rebuild_or_test(&runner).await;

        match result {
            Err(WorkspaceError::TestFailure(TestFailure::UnknownError { stdout, stderr })) => {
                info!("Got expected test failure:\nstdout={:?}\nstderr={:?}", stdout, stderr);
            }
            Ok(_) => panic!("Expected test failure, but got success"),
            other => panic!("Expected TestFailure::UnknownError, got: {:?}", other),
        }

        Ok(())
    }

    /// A minimal "mock workspace" struct that re-uses your RebuildOrTest logic
    /// by containing a real path on disk.
    /// We’ll implement `RebuildOrTest` in the same way as your real code does:
    #[derive(Debug)]
    struct MockWorkspace {
        path: PathBuf,
    }

    impl AsRef<std::path::Path> for MockWorkspace {
        fn as_ref(&self) -> &std::path::Path {
            &self.path
        }
    }

    #[async_trait]
    impl RebuildOrTest for MockWorkspace {
        type Error = WorkspaceError;

        async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error> {
            // Same logic as in your real `impl RebuildOrTest for Workspace<P,H>`:
            info!("Running cargo build...");
            let mut build_cmd = Command::new("cargo");
            build_cmd.arg("build").current_dir(&self.path);

            let output = runner.run_command(build_cmd).await??;
            if !output.status.success() {
                error!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
                return Err(WorkspaceError::from(BuildError::BuildFailed {
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                }));
            }

            info!("Rebuild succeeded, running tests...");
            let mut test_cmd = Command::new("cargo");
            test_cmd.arg("test").current_dir(&self.path);

            let test_output = runner.run_command(test_cmd).await??;
            if !test_output.status.success() {
                let stdout = Some(String::from_utf8_lossy(&test_output.stdout).to_string());
                let stderr = Some(String::from_utf8_lossy(&test_output.stderr).to_string());
                error!("Tests failed: {:#?}", stderr);
                return Err(WorkspaceError::from(TestFailure::UnknownError { stdout, stderr }));
            }

            info!("Tests passed successfully.");
            Ok(())
        }
    }
}
