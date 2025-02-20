// ---------------- [ File: src/rebuild_or_test.rs ]
crate::ix!();

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
